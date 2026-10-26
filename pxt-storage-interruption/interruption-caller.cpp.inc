// SPDX-License-Identifier: MIT
// Explicit operation-boundary interruption roles; no reference firmware.
#ifndef BW_INTERRUPTION_ROLE
#error Define BW_INTERRUPTION_ROLE to producer(0), writer(1), or reader(2)
#endif
namespace settings {
int _set(String, Buffer); bool _exists(String);
Buffer _get(String); RefCollection *_list(String);
}
extern "C" {
bool bw_storage_try_mount(); void bw_storage_format();
unsigned bw_storage_base(); unsigned bw_storage_size();
volatile unsigned storage_stage=0, storage_checks=0, storage_result=0;
volatile unsigned interruption_data=0x1234abcd, interruption_bss=0;
volatile unsigned storage_canary[4] __attribute__((used, section(".storage_canary")))={0x13579bdf,0x2468ace0,0x5a5a5a5a,0xc3c3c3c3};
}
static void storage_require(bool ok,unsigned reason) {
    if (!ok) target_panic(reason);
    ++storage_checks;
}
static unsigned char old_byte(unsigned i) { return (i*17u+29u)%251u; }
static unsigned char new_byte(unsigned i) { return (i*37u+71u)%251u; }
static void check_value(Buffer value,unsigned length,bool newer,unsigned reason) {
    storage_require(value && unsigned(value->length)==length,reason);
    for(unsigned i=0;i<length;++i)
        storage_require(value->data[i]==(newer?new_byte(i):old_byte(i)),reason);
}
static void storage_run() {
    storage_require(interruption_data==0x1234abcd && interruption_bss==0,0xe00);
    auto key=mkString("atomic.value",-1);registerGCObj(key);
    auto keep=mkString("atomic.keep",-1);registerGCObj(keep);
    auto prefix=mkString("atomic.",-1);registerGCObj(prefix);
    storage_require(bw_storage_base()==0x78000 && bw_storage_size()==32768,0xe01);
    bool mounted=bw_storage_try_mount();
#if BW_INTERRUPTION_ROLE == 0
    storage_require(!mounted,0xe02);
    bw_storage_format();
    storage_require(bw_storage_try_mount(),0xe03);
    auto old=mkBuffer(nullptr,97);registerGCObj(old);
    auto neighbour=mkBuffer(nullptr,19);registerGCObj(neighbour);
    for(unsigned i=0;i<97;++i)old->data[i]=old_byte(i);
    for(unsigned i=0;i<19;++i)neighbour->data[i]=old_byte(i);
    storage_require(settings::_set(key,old)==0,0xe04);
    storage_require(settings::_set(keep,neighbour)==0,0xe05);
    check_value(settings::_get(key),97,false,0xe06);
    check_value(settings::_get(keep),19,false,0xe07);
    storage_stage=1;
    for(;;) asm volatile("nop");
#elif BW_INTERRUPTION_ROLE == 1
    storage_require(mounted,0xe10); // No formatting branch in this role.
    check_value(settings::_get(key),97,false,0xe11);
    check_value(settings::_get(keep),19,false,0xe12);
    auto next=mkBuffer(nullptr,257);registerGCObj(next);
    for(unsigned i=0;i<257;++i)next->data[i]=new_byte(i);
    storage_stage=2;
    storage_require(settings::_set(key,next)==0,0xe13);
    storage_stage=3;
    for(;;) asm volatile("nop");
#elif BW_INTERRUPTION_ROLE == 2
    storage_require(mounted,0xe20); // Recovery must mount before any format/write.
    auto saved=settings::_get(key);registerGCObj(saved);
    storage_require(saved!=nullptr,0xe21);
    if(saved->length==97) {
        check_value(saved,97,false,0xe22);
        storage_result=1;
    } else {
        check_value(saved,257,true,0xe23);
        storage_result=2;
    }
    check_value(settings::_get(keep),19,false,0xe24);
    storage_require(settings::_exists(key) && settings::_exists(keep),0xe25);
    auto list=settings::_list(prefix);registerGCObj(list);
    storage_require(list && list->length()==2,0xe26);
    unsigned names=0;
    for(int i=0;i<list->length();++i) {
        auto name=reinterpret_cast<String>(list->getAt(i));
        storage_require(name!=nullptr,0xe27);
        if(strcmp(name->getUTF8Data(),"atomic.value")==0)names|=1;
        else if(strcmp(name->getUTF8Data(),"atomic.keep")==0)names|=2;
        else storage_require(false,0xe28);
    }
    storage_require(names==3,0xe29);
    storage_stage=4;
#else
#error Unknown interruption role
#endif
}
