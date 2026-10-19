// SPDX-License-Identifier: MIT
// Appended after the byte-preserved real settings translation unit. This
// separately selected diagnostic exposes original APIs, never return stubs.
extern "C" bool bw_storage_try_mount() {
    return settings::getWStorage()->fs.tryMount();
}
extern "C" void bw_storage_format() {
    settings::getWStorage()->fs.format();
}
extern "C" unsigned bw_storage_base() {
    return settings::getWStorage()->fs.baseAddr;
}
extern "C" unsigned bw_storage_size() {
    return settings::getWStorage()->fs.bytes;
}
extern "C" void bw_storage_drop() {
    delete settings::instWStorage;
    settings::instWStorage = nullptr;
}
