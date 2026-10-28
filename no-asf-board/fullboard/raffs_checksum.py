# SPDX-License-Identifier: MIT
# Copyright (c) 2026 CrispStrobe contributors
"""Reviewed, exact-site adaptations for RAFFS v2 data CRCs.

The caller verifies the original request SHA before applying these edits.
Metadata remains eight bytes. A v2 bank appends a CRC32 to each data payload;
legacy banks remain readable and migrate through the normal atomic bank GC.
"""

CRC_HELPERS = b'''
// CRC-32/ISO-HDLC: reflected polynomial, initial/final xor 0xffffffff.
// Bind a payload to its key and length; never treat a CRC as authentication.
static uint32_t crcBytes(uint32_t crc, const void *data, uint32_t size) {
    auto p = (const uint8_t *)data;
    while (size--) {
        crc ^= *p++;
        for (int bit = 0; bit < 8; ++bit)
            crc = (crc >> 1) ^ ((crc & 1) ? 0xedb88320u : 0);
    }
    return crc;
}

static uint32_t dataCRC(const char *name, const void *data, uint32_t size) {
    uint8_t length[4] = {(uint8_t)size, (uint8_t)(size >> 8),
                         (uint8_t)(size >> 16), (uint8_t)(size >> 24)};
    uint32_t crc = crcBytes(0xffffffffu, name, strlen(name) + 1);
    crc = crcBytes(crc, length, sizeof(length));
    return crcBytes(crc, data, size) ^ 0xffffffffu;
}

static bool validEntry(const MetaEntry *m, const uint8_t *base, uint32_t bank);
'''

VALID_ENTRY = b'''// Bounds precede every name/payload/CRC read, including legacy entries.
static bool validEntry(const MetaEntry *m, const uint8_t *base, uint32_t bank) {
    if (m->fnptr < sizeof(FSHeader) || m->fnptr >= bank ||
        !memchr(base + m->fnptr, 0, bank - m->fnptr))
        return false;
    if (m->dataptr == 0)
        return true; // deleted
    uint32_t size = m->_datasize & 0x7fff;
    bool checked = ((const FSHeader *)base)->magic == RAFFS_MAGIC;
    uint32_t extent = size + (checked ? sizeof(uint32_t) : 0);
    if (m->dataptr < sizeof(FSHeader) || m->dataptr > bank || extent > bank - m->dataptr)
        return false;
    if (!checked)
        return true;
    uint32_t stored;
    memcpy(&stored, base + m->dataptr + size, sizeof(stored));
    return stored == dataCRC((const char *)(base + m->fnptr), base + m->dataptr, size);
}

MetaEntry *FS::findMetaEntry(const char *filename) {
'''

EDITS = [
    (b'#define RAFFS_MAGIC 0x7776e0da\n',
     b'#define RAFFS_MAGIC_LEGACY 0x7776e0da\n#define RAFFS_MAGIC 0x7776e0db\n'),
    (b'static uint16_t fnhash(const char *fn) {\n', CRC_HELPERS + b'\nstatic uint16_t fnhash(const char *fn) {\n'),
    (b'#define IS_VALID(hd) ((hd)->magic == RAFFS_MAGIC && (hd)->bytes == bytes)\n',
     b'#define IS_VALID(hd) (((hd)->magic == RAFFS_MAGIC || (hd)->magic == RAFFS_MAGIC_LEGACY) && (hd)->bytes == bytes)\n'),
    (b'    lock();\n    uint32_t szneeded = bytes;\n',
     b'''    lock();
    // Read old banks without changing them. Before a write, migrate through
    // GC: the new bank header is the final commit, so a cut keeps the old one.
    if (((FSHeader *)basePtr)->magic == RAFFS_MAGIC_LEGACY) {
        tryGC(0x7fff0000);
        if (((FSHeader *)basePtr)->magic == RAFFS_MAGIC_LEGACY) {
            unlock();
            return -1; // migration did not fit; old bank remains intact
        }
    }
    if (!isDel && bytes > 0x7fff) {
        unlock();
        return -1;
    }
    uint32_t szneeded = isDel ? 0 : bytes + sizeof(uint32_t);
'''),
    (b'    newMeta._datasize = bytes;\n',
     b'''    newMeta._datasize = bytes;
    if (!isDel) {
        uint32_t crc = dataCRC(keyName, data, bytes);
        writeData(&crc, sizeof(crc));
    }
'''),
    (b'    if (meta != NULL && meta->dataptr) {\n',
     b'    if (meta != NULL && meta->dataptr && validEntry(meta, basePtr, this->bytes / 2)) {\n'),
    (b'    erasePages(newBase, bytes / 2);\n',
     b'''    // A legacy bank can need more space after CRCs are added. Preflight
    // live entries before erasing or writing the alternate bank.
    uint32_t used = sizeof(FSHeader) + 64;
    clearBlocked();
    for (auto p = metaPtr; p < endPtr; ++p) {
        MetaEntry m = *p;
        if (!validEntry(&m, basePtr, bytes / 2))
            continue;
        const char *fn = fnptr(&m);
        if ((filter && !filter(fn)) || checkBlocked(&m) || m.dataptr == 0)
            continue;
        uint32_t cost = strlen(fn) + 1 + m.datasize() + sizeof(uint32_t) + sizeof(MetaEntry);
        if (used > bytes / 2 || cost > bytes / 2 - used) {
            clearBlocked();
            return false;
        }
        used += cost;
    }
    clearBlocked();
    erasePages(newBase, bytes / 2);
'''),
    (b'            auto sz = fnlen + m.datasize();\n',
     b'            auto sz = fnlen + m.datasize() + sizeof(uint32_t);\n'),
    (b'                writeData(basePtr + m.dataptr, m.datasize());\n',
     b'''                writeData(basePtr + m.dataptr, m.datasize());
                uint32_t crc = dataCRC(fn, basePtr + m.dataptr, m.datasize());
                writeData(&crc, sizeof(crc));
'''),
]
