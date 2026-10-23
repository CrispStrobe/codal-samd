// SPDX-License-Identifier: MIT
namespace settings {
int _set(String, Buffer); int _remove(String); bool _exists(String);
Buffer _get(String); RefCollection *_list(String);
}
extern "C" {
bool bw_storage_try_mount(); void bw_storage_format(); void bw_storage_drop();
unsigned bw_storage_base(); unsigned bw_storage_size(); unsigned bw_compaction_spacing();
volatile unsigned storage_stage=0, storage_checks=0, storage_native=0, storage_saved=0;
volatile unsigned compaction_mode=0, storage_version=0, storage_spacing=0;
volatile unsigned storage_canary[4] __attribute__((used,section(".storage_canary")))={0x13579bdf,0x2468ace0,0x5a5a5a5a,0xc3c3c3c3};
}
static void storage_require(bool ok,unsigned code) { if(!ok) target_panic(code); ++storage_checks; }
static unsigned char keep_byte(unsigned i) { return (i*37u+11u)&255u; }
static unsigned char rotate_byte(unsigned i,unsigned version) { return (i*13u+version*17u+3u)&255u; }
static unsigned generation() {
    auto a=reinterpret_cast<volatile unsigned *>(0x78000u);
    auto b=reinterpret_cast<volatile unsigned *>(0x7c000u);
    unsigned x=a[0]==0x7776e0dau && a[1]==32768 ? a[2] : 0;
    unsigned y=b[0]==0x7776e0dau && b[1]==32768 ? b[2] : 0;
    return x>y?x:y;
}
static void inspect_value(String key,unsigned length,unsigned version,unsigned code) {
    auto value=settings::_get(key); registerGCObj(value); storage_saved=uintptr_t(value);
    storage_require(value && unsigned(value->length)==length,code);
    for(unsigned i=0;i<length;++i) storage_require(value->data[i]==(version?rotate_byte(i,version):keep_byte(i)),code);
    unregisterGCObj(value);
}
static void inspect_lists(String prefix,String internal) {
    auto list=settings::_list(prefix); registerGCObj(list);
    storage_require(list && list->length()==2,0xc41);
    unsigned keep=0,rotate=0;
    for(unsigned i=0;i<2;++i) {
        auto s=reinterpret_cast<String>(list->getAt(i));
        storage_require(s!=nullptr,0xc42);
        keep+=strcmp(s->getUTF8Data(),"user.keep")==0;
        rotate+=strcmp(s->getUTF8Data(),"user.rotate")==0;
    }
    storage_require(keep==1 && rotate==1,0xc43); unregisterGCObj(list);
    list=settings::_list(internal); registerGCObj(list);
    storage_require(list && list->length()==1,0xc44);
    auto s=reinterpret_cast<String>(list->getAt(0));
    storage_require(s && strcmp(s->getUTF8Data(),"#keep")==0,0xc45);
    unregisterGCObj(list);
}
static void storage_run() {
    auto keep=mkString("user.keep",-1);registerGCObj(keep);
    auto gone=mkString("user.gone",-1);registerGCObj(gone);
    auto rotate=mkString("user.rotate",-1);registerGCObj(rotate);
    auto internal=mkString("#keep",-1);registerGCObj(internal);
    auto prefix=mkString("user.",-1);registerGCObj(prefix);
    auto iprefix=mkString("#",-1);registerGCObj(iprefix);
    auto kept=mkBuffer(nullptr,257);registerGCObj(kept);
    auto small=mkBuffer(nullptr,19);registerGCObj(small);
    auto value=mkBuffer(nullptr,4096);registerGCObj(value);
    auto native=static_cast<unsigned char *>(bw_native_alloc(257));
    storage_require(native && (uintptr_t(native)&7)==0,0xc01);storage_native=uintptr_t(native);
    for(unsigned i=0;i<257;++i) native[i]=kept->data[i]=keep_byte(i);
    for(unsigned i=0;i<19;++i) small->data[i]=keep_byte(i);
    storage_require(bw_storage_base()==0x78000 && bw_storage_size()==32768,0xc02);
    storage_spacing=bw_compaction_spacing();storage_require(storage_spacing==10000,0xc03);
    storage_require(!bw_storage_try_mount(),0xc04);bw_storage_format();
    storage_require(bw_storage_try_mount() && generation()==1,0xc05);
    storage_require(settings::_set(keep,kept)==0,0xc06);
    storage_require(settings::_set(gone,small)==0,0xc07);
    storage_require(settings::_remove(gone)==0,0xc08);
    storage_require(settings::_set(internal,small)==0,0xc09);
    storage_stage=1;
    unsigned limit=compaction_mode?12u:6u;
    for(unsigned version=1;version<=limit;++version) {
        for(unsigned i=0;i<4096;++i)value->data[i]=rotate_byte(i,version);
        storage_version=version;
        storage_require(settings::_set(rotate,value)==0,0xc10);
        if(version==3) { storage_require(generation()==1,0xc11); storage_stage=2; }
        if(version==4 || version==6) {
            storage_require(generation()==(version==4?2u:3u),0xc20);
            storage_stage=version==4?3:4;
            gc(0);
            inspect_value(keep,257,0,0xc31);
            inspect_value(internal,19,0,0xc32);
            inspect_value(rotate,4096,version,0xc33);
            storage_require(!settings::_exists(gone),0xc34);
            storage_require(settings::_get(gone)==nullptr,0xc35);
            inspect_lists(prefix,iprefix);
            for(unsigned i=0;i<257;++i) storage_require(native[i]==keep_byte(i),0xc36);
            for(unsigned i=0;i<257;++i) storage_require(kept->data[i]==keep_byte(i),0xc37);
        }
    }
    if(compaction_mode) target_panic(0xc99); // Missing original spacing refusal.
    bw_storage_drop(); gc(0);
    storage_require(bw_storage_try_mount() && generation()==3,0xc50);
    inspect_value(keep,257,0,0xc51);inspect_value(internal,19,0,0xc52);
    inspect_value(rotate,4096,6,0xc53);
    storage_require(!settings::_exists(gone),0xc54);
    storage_require(settings::_get(gone)==nullptr,0xc55);
    inspect_lists(prefix,iprefix);
    for(unsigned i=0;i<257;++i) storage_require(native[i]==keep_byte(i),0xc56);
    for(unsigned i=0;i<257;++i) storage_require(kept->data[i]==keep_byte(i),0xc57);
    storage_stage=5;
    unregisterGCObj(value);unregisterGCObj(small);unregisterGCObj(kept);
    unregisterGCObj(iprefix);unregisterGCObj(prefix);unregisterGCObj(internal);
    unregisterGCObj(rotate);unregisterGCObj(gone);unregisterGCObj(keep);
    bw_native_free(native);
}
