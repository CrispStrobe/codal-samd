// SPDX-License-Identifier: MIT
namespace settings {
int _set(String, Buffer); int _remove(String); bool _exists(String);
Buffer _get(String); RefCollection *_list(String);
}
extern "C" {
bool bw_storage_try_mount(); void bw_storage_format();
unsigned bw_storage_base(); unsigned bw_storage_size();
volatile unsigned storage_stage=0, storage_checks=0;
volatile unsigned reset_data=0x1234abcd, reset_bss=0;
volatile unsigned storage_canary[4] __attribute__((used, section(".storage_canary")))={0x13579bdf,0x2468ace0,0x5a5a5a5a,0xc3c3c3c3};
}
static void storage_require(bool ok, unsigned reason) {
    if (!ok) target_panic(reason);
    ++storage_checks;
}
static unsigned char pattern(unsigned i) { return (i*37u+11u)&255u; }
static void check_buffer(Buffer value,unsigned size,unsigned reason) {
    storage_require(value && unsigned(value->length)==size,reason);
    for(unsigned i=0;i<size;++i) storage_require(value->data[i]==pattern(i),reason);
}
static void check_list(RefCollection *list,const char *name,unsigned reason) {
    storage_require(list && list->length()==1,reason);
    auto key=reinterpret_cast<String>(list->getAt(0));
    storage_require(key && strcmp(key->getUTF8Data(),name)==0,reason);
}
static void storage_run() {
    storage_require(reset_data==0x1234abcd && reset_bss==0,0xd01);
    auto key=mkString("user.alpha",-1);registerGCObj(key);
    auto gone=mkString("user.gone",-1);registerGCObj(gone);
    auto marker=mkString("#keep",-1);registerGCObj(marker);
    auto prefix=mkString("user.",-1);registerGCObj(prefix);
    auto iprefix=mkString("#",-1);registerGCObj(iprefix);
    storage_require(bw_storage_base()==0x78000 && bw_storage_size()==32768,0xd02);
    bool mounted=bw_storage_try_mount();
    if(!mounted) {
        // Only the producer may format. A lost-backing reader stops at the
        // actual failed mount return before this branch can format.
        bw_storage_format();
        storage_require(bw_storage_try_mount(),0xd03);
        auto value=mkBuffer(nullptr,257);registerGCObj(value);
        auto small=mkBuffer(nullptr,19);registerGCObj(small);
        for(unsigned i=0;i<257;++i)value->data[i]=pattern(i);
        for(unsigned i=0;i<19;++i)small->data[i]=pattern(i);
        storage_require(settings::_set(key,value)==0,0xd04);
        storage_require(settings::_set(gone,small)==0,0xd05);
        storage_require(settings::_set(marker,small)==0,0xd06);
        storage_require(settings::_remove(gone)==0,0xd07);
        auto saved=settings::_get(key);registerGCObj(saved);
        check_buffer(saved,257,0xd08);
        storage_require(!settings::_exists(gone),0xd09);
        auto list=settings::_list(prefix);registerGCObj(list);
        check_list(list,"user.alpha",0xd0a);unregisterGCObj(list);
        list=settings::_list(iprefix);registerGCObj(list);
        check_list(list,"#keep",0xd0b);unregisterGCObj(list);
        // All original operations returned. Quiesce for exact backing export.
        Mask::restore(1);
        reset_data=0xdeadbeef;reset_bss=0xa5a55a5a;
        storage_stage=1;
        for(;;) asm volatile("nop");
    }
    storage_require(settings::_exists(marker),0xd10);
    auto saved=settings::_get(key);registerGCObj(saved);
    auto internal=settings::_get(marker);registerGCObj(internal);
    check_buffer(saved,257,0xd11);
    check_buffer(internal,19,0xd12);
    storage_require(!settings::_exists(gone),0xd13);
    storage_require(settings::_get(gone)==nullptr,0xd14);
    auto list=settings::_list(prefix);registerGCObj(list);
    check_list(list,"user.alpha",0xd15);unregisterGCObj(list);
    list=settings::_list(iprefix);registerGCObj(list);
    check_list(list,"#keep",0xd16);unregisterGCObj(list);
    storage_stage=2;
    unregisterGCObj(internal);unregisterGCObj(saved);unregisterGCObj(iprefix);
    unregisterGCObj(prefix);unregisterGCObj(marker);unregisterGCObj(gone);unregisterGCObj(key);
}
