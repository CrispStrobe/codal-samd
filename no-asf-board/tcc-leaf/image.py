#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Flash image for the TCC timer leaf: an 8-byte reset forward to the leaf's own
vector table at the application origin 0x4000, erased flash below it, no
bootloader code and no CF2 data."""
import hashlib, json, struct, sys
from pathlib import Path

ORIGIN = 0x4000


def main():
    binary, out = Path(sys.argv[1]), Path(sys.argv[2])
    app = binary.read_bytes()
    stack, reset = struct.unpack('<II', app[:8])
    if not (0x20000000 <= stack <= 0x20030000 and ORIGIN <= reset & ~1 < ORIGIN + len(app)):
        raise ValueError('vector table at the application origin')
    image = struct.pack('<II', stack, reset) + b'\xff' * (ORIGIN - 8) + app
    out.write_bytes(image)
    digest = hashlib.sha256(image).hexdigest()
    out.with_suffix('.json').write_text(json.dumps(dict(image_sha256=digest, bootloader_code=False,
                                                         application_bytes=len(app)), indent=2) + '\n')
    print(f'TCC_LEAF_IMAGE bytes={len(image)} image_sha256={digest}')


if __name__ == '__main__':
    main()
