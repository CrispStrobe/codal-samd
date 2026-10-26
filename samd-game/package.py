#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Package the actual native binary as a flash ELF plus explicit authored inputs."""
from pathlib import Path
import hashlib,json,struct,sys,subprocess
image=Path(sys.argv[1]);out=Path(sys.argv[2]);out.mkdir(exist_ok=False)
raw=image.read_bytes()
if not 0x4008<=len(raw)<=0x80000:raise ValueError('native flash image within declared 512KiB capacity required')
sp,reset=struct.unpack('<II',raw[:8])
if sp!=0x20030000 or not reset&1 or not 0x4000<=reset<len(raw):raise ValueError('actual reset pair')
# One loadable flash segment; the loader resets from its real vector pair.
ident=b'\x7fELF\x01\x01\x01'+b'\0'*9
header=ident+struct.pack('<HHIIIIIHHHHHH',2,40,1,reset,52,0,0x05000000,52,32,1,0,0,0)
program=struct.pack('<8I',1,0x1000,0,0,len(raw),len(raw),5,0x1000)
(out/'game.elf').write_bytes(header+program+b'\0'*(0x1000-84)+raw)
(out/'game.bin').write_bytes(raw)
sw=bytearray([255]*16);sw[4:8]=struct.pack('<I',0x1b29)
serial=bytearray(512);serial[0x10:0x1c]=struct.pack('<3I',1,2,3);serial[0x1fc:]=struct.pack('<I',0x42570001)
(out/'sw0.bin').write_bytes(sw);(out/'serial.bin').write_bytes(serial)
symbols={}
for line in subprocess.check_output(['arm-none-eabi-nm','--defined-only',sys.argv[3]],text=True).splitlines():
 parts=line.split()
 if len(parts)==3:
  if parts[2].startswith('_ZN3pxt12updateScreenE'):key='updateScreen'
  elif parts[2]=='target_panic':key='target_panic'
  else:continue
  if key in symbols:raise ValueError('unique source render/panic symbols required')
  symbols[key]=int(parts[0],16)&~1
if set(symbols)!={'updateScreen','target_panic'}:raise ValueError('actual base render/panic symbols')
(out/'symbols.json').write_text(json.dumps(symbols,indent=2))
(out/'package.json').write_text(json.dumps(dict(schema=1,bootloader_code=False,physical_capture=False,authored_calibration=True,files={p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in out.iterdir()}),indent=2))
