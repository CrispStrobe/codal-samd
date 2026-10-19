// SPDX-License-Identifier: MIT
namespace settings {
int _set(String, Buffer);
int _remove(String);
bool _exists(String);
Buffer _get(String);
RefCollection *_list(String);
}
extern "C" {
bool bw_storage_try_mount();
void bw_storage_format();
void bw_storage_drop();
unsigned bw_storage_base();
unsigned bw_storage_size();
volatile unsigned storage_stage=0, storage_checks=0;
volatile unsigned storage_native=0, storage_saved=0;
volatile unsigned storage_canary[4] __attribute__((used, section(".storage_canary")))={0x13579bdf,0x2468ace0,0x5a5a5a5a,0xc3c3c3c3};
}
static void storage_require(bool ok, unsigned reason) {
    if (!ok) target_panic(reason);
    ++storage_checks;
}
static unsigned char pattern(unsigned i) { return (i * 37u + 11u) & 255u; }
static void check_buffer(Buffer value, unsigned size, unsigned reason) {
    storage_require(value && unsigned(value->length)==size, reason);
    for (unsigned i=0;i<size;++i) storage_require(value->data[i]==pattern(i),reason);
}
static void check_list(RefCollection *list, const char *name, unsigned reason) {
    storage_require(list && list->length()==1,reason);
    auto key=reinterpret_cast<String>(list->getAt(0));
    storage_require(key && strcmp(key->getUTF8Data(),name)==0,reason);
}
static void storage_pressure() {
    pressure();
    // Equal-sized unrooted actual buffers force reuse after real collection.
    for (unsigned i=0;i<12;++i) {
        auto scratch=mkBuffer(nullptr,257);
        memset(scratch->data,0xcc,257);
        gc(0);
    }
}
static void storage_run() {
    auto key=mkString("user.alpha",-1); registerGCObj(key);
    auto beta=mkString("user.beta",-1); registerGCObj(beta);
    auto internal=mkString("#keep",-1); registerGCObj(internal);
    auto prefix=mkString("user.",-1); registerGCObj(prefix);
    auto iprefix=mkString("#",-1); registerGCObj(iprefix);
    auto value=mkBuffer(nullptr,257); registerGCObj(value);
    auto short_value=mkBuffer(nullptr,19); registerGCObj(short_value);
    auto native=static_cast<unsigned char *>(bw_native_alloc(257));
    storage_require(native && (uintptr_t(native)&7)==0,0xb01);
    for (unsigned i=0;i<257;++i) native[i]=value->data[i]=pattern(i);
    for (unsigned i=0;i<19;++i) short_value->data[i]=pattern(i);
    storage_native=uintptr_t(native);
    storage_require(getConfig(CFG_SETTINGS_SIZE_DEFL,-1)==32768,0xb02);
    storage_require(getConfig(CFG_SETTINGS_SIZE,-1)==0,0xb03);
    storage_require(bw_storage_base()==0x78000 && bw_storage_size()==32768,0xb04);
    storage_require(!bw_storage_try_mount(),0xb05);
    bw_storage_format();
    storage_require(bw_storage_try_mount(),0xb06);
    storage_require(!settings::_exists(key),0xb07);
    storage_require(settings::_get(key)==nullptr,0xb08);
    storage_require(settings::_remove(key)==-1,0xb09);
    storage_stage=1;

    storage_require(settings::_set(key,value)==0,0xb10);
    storage_require(settings::_exists(key),0xb11);
    auto saved=settings::_get(key); registerGCObj(saved);
    storage_saved=uintptr_t(saved);
    check_buffer(saved,257,0xb20);
    auto list=settings::_list(prefix); registerGCObj(list);
    check_list(list,"user.alpha",0xb12);
    unregisterGCObj(list);
    storage_stage=2;

    storage_require(settings::_set(beta,short_value)==0,0xb13);
    storage_require(settings::_set(internal,short_value)==0,0xb14);
    storage_pressure();
    check_buffer(saved,257,0xb21);
    check_buffer(value,257,0xb22);
    for (unsigned i=0;i<257;++i) storage_require(native[i]==pattern(i),0xb23);
    storage_stage=3;

    storage_require(settings::_remove(key)==0,0xb15);
    storage_require(!settings::_exists(key),0xb30);
    storage_require(settings::_get(key)==nullptr,0xb16);
    // The pinned FS retains tombstone metadata: a repeated deletion succeeds.
    storage_require(settings::_remove(key)==0,0xb17);
    list=settings::_list(prefix); registerGCObj(list);
    check_list(list,"user.beta",0xb18);
    unregisterGCObj(list);
    storage_stage=4;

    // All real operations have returned. Retire the only old filesystem cache
    // before opening a fresh wrapper over this same machine/flash backing.
    bw_storage_drop();
    storage_pressure();
    check_buffer(saved,257,0xb24);
    storage_stage=5;
    storage_require(bw_storage_try_mount(),0xb40);
    storage_require(!settings::_exists(key) && settings::_exists(beta),0xb41);
    auto recovered=settings::_get(beta); registerGCObj(recovered);
    check_buffer(recovered,19,0xb42);
    list=settings::_list(prefix); registerGCObj(list);
    check_list(list,"user.beta",0xb43);
    unregisterGCObj(list);
    list=settings::_list(iprefix); registerGCObj(list);
    check_list(list,"#keep",0xb44);
    unregisterGCObj(list);
    storage_pressure();
    check_buffer(recovered,19,0xb45);
    check_buffer(saved,257,0xb46);
    for (unsigned i=0;i<257;++i) storage_require(native[i]==pattern(i),0xb47);
    storage_stage=6;
    unregisterGCObj(recovered);
    unregisterGCObj(saved);
    unregisterGCObj(short_value);
    unregisterGCObj(value);
    unregisterGCObj(iprefix);
    unregisterGCObj(prefix);
    unregisterGCObj(internal);
    unregisterGCObj(beta);
    unregisterGCObj(key);
    bw_native_free(native);
}
