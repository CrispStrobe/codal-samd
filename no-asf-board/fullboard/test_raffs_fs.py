#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 CrispStrobe contributors
"""Run the byte-bound legacy and adapted FS on an authored host flash.

This fast gate covers migration cuts and a full legacy bank that cannot fit
CRC overhead. Native guest cases qualify the real board path separately.
"""
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

import build

REQUEST = Path(sys.argv.pop())


class FilesystemTests(unittest.TestCase):
    def test_actual_filesystems_migration_cuts_and_capacity_refusal(self):
        request = REQUEST.read_bytes()
        self.assertEqual(build.sha(request), build.VARIANTS['base']['request_sha256'])
        files = json.loads(request)['replaceFiles']
        with tempfile.TemporaryDirectory(prefix='raffs-fs-') as temporary:
            root = Path(temporary)
            shared = root / 'stub'
            shared.mkdir()
            (shared / 'Flash.h').write_text('''#pragma once
#include <stdint.h>
#include <stddef.h>
#include <string.h>
#include <array>
#include <stdexcept>
struct PowerCut {};
namespace codal {
struct Flash {
    alignas(512) std::array<uint8_t,4096> data;
    int cut = -1, calls = 0;
    Flash() { data.fill(0xff); }
    void before() { if (cut == 0) throw PowerCut(); if (cut > 0) --cut; ++calls; }
    int pageSize(uintptr_t) { return 512; }
    int erasePage(uintptr_t at) { before(); memset((void *)at, 0xff, 512); return 0; }
    int writeBytes(uintptr_t at, const void *src, uint32_t size) {
        before();
        for (uint32_t i=0; i<size; ++i) ((uint8_t *)at)[i] &= ((const uint8_t *)src)[i];
        return 0;
    }
};
inline uint16_t allocateNotifyEvent() { return 1; }
inline void fiber_wait_for_event(int, int) { throw std::runtime_error("unexpected wait"); }
inline uint32_t system_timer_current_time() { return 0; }
}
inline void target_panic(int) { throw std::runtime_error("FS panic"); }
inline uint32_t hash_fnv1(const void *data, uint32_t size) {
    uint32_t h=2166136261u;
    for (uint32_t i=0;i<size;++i) h=(h*16777619u)^((const uint8_t *)data)[i];
    return h;
}
#define DEVICE_FLASH_ERROR 922
#define DEVICE_ID_NOTIFY 1
#define LOG(...) ((void)0)
#define LOGV(...) ((void)0)
''')
            for name in ['CodalDmesg.h', 'NotifyEvents.h', 'MessageBus.h', 'Timer.h', 'pxtbase.h']:
                (shared / name).write_text('#include "Flash.h"\n')
            units = []
            for version in ['legacy', 'modern']:
                folder = root / version
                source = folder / 'pxtapp/settings/RAFFS.cpp'
                source.parent.mkdir(parents=True)
                source.write_text(files['/pxtapp/settings/RAFFS.cpp'])
                source.with_suffix('.h').write_text(files['/pxtapp/settings/RAFFS.h'])
                if version == 'modern':
                    build.apply_fixes(folder, build.REQUEST_FIXES[:1])
                wrapper = folder / 'wrapper.cpp'
                wrapper.write_text('''#define RAFFS_TEST 1
#define pxt VERSION_pxt
#include "pxtapp/settings/RAFFS.cpp"
extern "C" void *VERSION_new(codal::Flash *flash) {
    return new pxt::raffs::FS(*flash, (uintptr_t)flash->data.data(), flash->data.size());
}
extern "C" int VERSION_write(void *fs, const char *key, const void *value, int size) {
    return ((pxt::raffs::FS *)fs)->write(key, value, size);
}
extern "C" int VERSION_read(void *fs, const char *key, void *value, int size) {
    return ((pxt::raffs::FS *)fs)->read(key, value, size);
}
extern "C" void VERSION_delete(void *fs) { delete (pxt::raffs::FS *)fs; }
'''.replace('VERSION', version))
                units.append(wrapper)
            driver = root / 'main.cpp'
            driver.write_text('''#include "Flash.h"
#include <assert.h>
#include <stdio.h>
extern "C" void *legacy_new(codal::Flash *);
extern "C" void *modern_new(codal::Flash *);
extern "C" int legacy_write(void *, const char *, const void *, int);
extern "C" int modern_write(void *, const char *, const void *, int);
extern "C" int legacy_read(void *, const char *, void *, int);
extern "C" int modern_read(void *, const char *, void *, int);
extern "C" void legacy_delete(void *);
extern "C" void modern_delete(void *);
int main() {
    codal::Flash seed;
    uint32_t old=0x11223344, next=0x55667788, read=0;
    void *fs=legacy_new(&seed);
    assert(legacy_write(fs,"key",&old,4)==0);
    legacy_delete(fs);
    auto snapshot=seed.data;
    fs=modern_new(&seed);
    assert(modern_read(fs,"key",&read,4)==4 && read==old);
    assert(seed.data==snapshot); // legacy reads do not mutate flash
    int start=seed.calls;
    assert(modern_write(fs,"key",&next,4)==0);
    int operations=seed.calls-start;
    modern_delete(fs);
    for (int cut=0;cut<=operations;++cut) {
        codal::Flash flash;
        flash.data=snapshot;
        flash.cut=cut;
        fs=modern_new(&flash);
        try { modern_write(fs,"key",&next,4); } catch (PowerCut &) {}
        modern_delete(fs);
        flash.cut=-1;
        fs=modern_new(&flash);
        read=0;
        assert(modern_read(fs,"key",&read,4)==4);
        assert(read==old || read==next);
        modern_delete(fs);
    }
    // Fill a real legacy FS until another live key cannot fit. A v2 migration
    // then needs more space than the legacy bank, and must preserve all bytes.
    codal::Flash full;
    fs=legacy_new(&full);
    int count=0;
    for (;count<1000;++count) {
        char key[16]; snprintf(key,sizeof(key),"k%d",count);
        if (legacy_write(fs,key,&old,4)!=0) break;
    }
    legacy_delete(fs);
    assert(count>10 && count<1000);
    snapshot=full.data;
    fs=modern_new(&full);
    assert(modern_write(fs,"new",&next,4)==-1);
    assert(full.data==snapshot);
    for (int i=0;i<count;++i) {
        char key[16]; snprintf(key,sizeof(key),"k%d",i);
        assert(modern_read(fs,key,&read,4)==4 && read==old);
    }
    modern_delete(fs);
    printf("PASS actual RAFFS: %d migration commit cuts, %d legacy keys preserved on capacity refusal\\n",operations+1,count);
}
''')
            binary = root / 'fs-test'
            compiled = subprocess.run([os.environ.get('CXX', 'c++'), '-std=c++11', '-O2',
                                       '-I' + str(shared), *map(str, units), str(driver), '-o', str(binary)],
                                      capture_output=True, text=True)
            self.assertEqual(compiled.returncode, 0, compiled.stderr)
            subprocess.run([str(binary)], check=True)


if __name__ == '__main__':
    unittest.main()
