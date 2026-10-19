#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Hosted actual RAFFS/settings + real GC/services link; no guest execution."""
from pathlib import Path
import argparse, hashlib, json, re, shutil, subprocess, sys
ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT/'pxt-gc-runtime'));import build_gc as gc
sys.path.insert(0,str(ROOT/'pxt-storage-compile'));import compile_flash as flash

CORE_EDITS=[(b'#include "pxtbase.h"\n',b'#include "pxtbase-storage.h"\n'),
 (b'#include <limits.h>\n',b'#include "freestanding.h"\n'),
 (b'#include <stdlib.h>\n',b'#include "freestanding.h"\n'),
 (b'using namespace std;\n',b'// No standard namespace symbols are used here.\n')]
SETTINGS_EDITS=[(b'#include "pxt.h"\n',b'#include "pxtbase-storage.h"\n'),
 (b'#include "GhostFAT.h"\n',b'#include "CodalDevice.h"\n')]
RAFFS_EDITS=[(b'#include "pxtbase.h"\n',b'#include "pxtbase-storage.h"\n'),
 (b'#include <stddef.h>\n',b'#include "freestanding.h"\n')]

def sha(raw):return hashlib.sha256(raw).hexdigest()
def need(ok,reason):
 if not ok:raise ValueError(reason)
def adapt(raw,edits):
 for before,after in edits:
  need(raw.count(before)==1,'unique original adaptation site required')
  raw=raw.replace(before,after,1)
 return raw

def run(core,clean_core,framework,generator,out):
 gc.consumer.hosted()
 for path in [core,clean_core,framework,generator,out]:
  need(path.is_relative_to(ROOT) and path!=ROOT,'owned checkout/output')
 need(not out.exists(),'fresh output required')
 base=ROOT/'storage-gc-base';driver=ROOT/'storage-driver-base'
 need(not base.exists() and not driver.exists(),'fresh separate producer outputs')
 # Reuse exact producers; their link-only originals are retained and never
 # relabelled as the final composition or executed baseline guest.
 gc.run(core,framework,generator,base)
 flash.run(clean_core,framework,driver)
 payloads,notice=gc.sources.verify(gc.sources.load(ROOT/'profiles/pxt-framework-sources.json'),framework)
 out.mkdir();(out/'notices').mkdir()
 for source in (base/'notices').iterdir():shutil.copyfile(source,out/'notices'/source.name)
 selected={
  'core-storage.cpp':adapt(payloads['pxtapp/base/core.cpp'],CORE_EDITS),
  'raffs-storage.cpp':adapt(payloads['pxtapp/settings/RAFFS.cpp'],RAFFS_EDITS),
  'settings-storage.cpp':adapt(payloads['pxtapp/settings/settings.cpp'],SETTINGS_EDITS)+(ROOT/'pxt-storage-runtime/settings-hook.cpp.inc').read_bytes(),
  'Flash.h':payloads['pxtapp/settings/Flash.h'],'RAFFS.h':payloads['pxtapp/settings/RAFFS.h']}
 # Link the actual CODAL compatibility implementation, not a replacement itoa.
 compat=(clean_core/'source/core/CodalCompat.cpp').read_bytes()
 selected['codal-compat.cpp']=b'#include "freestanding.h"\n'+compat
 for name,raw in selected.items():(out/name).write_bytes(raw)
 originals=out/'original-public';originals.mkdir()
 for name in ['pxtapp/base/core.cpp','pxtapp/settings/RAFFS.cpp','pxtapp/settings/RAFFS.h','pxtapp/settings/settings.cpp','pxtapp/settings/Flash.h']:
  (originals/name.replace('/','-')).write_bytes(payloads[name])
 (originals/'codal-core-CodalCompat.cpp').write_bytes(compat)
 commands=[]
 def command(argv,label):
  argv=[str(x) for x in argv];result=subprocess.run(argv,cwd=ROOT,capture_output=True,timeout=180)
  commands.append(dict(label=label,argv=argv,returncode=result.returncode))
  (out/'commands.json').write_text(json.dumps(commands,indent=2)+'\n')
  (out/(label+'.stdout')).write_bytes(result.stdout);(out/(label+'.stderr')).write_bytes(result.stderr)
  need(result.returncode==0,'actual composition command failed: '+label)
  return result.stdout
 old=json.loads((base/'commands.json').read_bytes());by={x['label']:x['argv'] for x in old}
 compiler=next(x for x in old if '-c' in x['argv'] and x['argv'][x['argv'].index('-c')+1]=='pxt-gc-runtime/probe.cpp')['argv']
 flags=compiler[1:compiler.index('-MD')]
 flags=['-I','pxt-storage-runtime','-I',out.relative_to(ROOT).as_posix(),'-include','pxt-storage-runtime/freestanding.h',*flags]
 objects=[];dependencies={}
 inputs=[*[out/n for n in selected if n.endswith('.cpp')],ROOT/'pxt-storage-runtime/probe.cpp']
 for index,path in enumerate(inputs):
  label='storage-'+str(index);obj=out/(label+'.o');objects.append(obj.relative_to(ROOT).as_posix())
  # This NEW translation-unit profile follows the existing boot caller's
  # optimized concrete ZFlash construction. The public abstract Flash base
  # declares an undefined non-pure totalSize; it supplies no fallback body.
  # Preserve the real header and derived vtable rather than invent a fallback.
  selected_flags=[f for f in flags if f!='-O0']+['-O2'] if path.name=='settings-storage.cpp' else flags
  command(['arm-none-eabi-g++',*selected_flags,'-MD','-MF',(out/(label+'.d')).relative_to(ROOT),'-c',path.relative_to(ROOT),'-o',obj.relative_to(ROOT)],label)
  for dep in gc.consumer.dependencies((out/(label+'.d')).read_text()):
   p=(ROOT/dep).resolve();need(p.is_relative_to(ROOT),'external/vendor dependency refused')
   need(p.relative_to(ROOT).parts[0] in ['pxt-storage-runtime','pxt-gc-runtime','pxt-native-allocation','runtime-services','simulation','paired-irq','storage-evidence','storage-gc-base','owned-gc-core','owned-storage-core'],'unselected/vendor dependency refused')
   dependencies[dep]=sha(p.read_bytes())
   target=out/'selected-dependencies'/dep;target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(p.read_bytes())
 # Replace only the original authored caller and extracted config objects.
 link=list(by['link']);remove=[x for x in link if x.endswith(('-probe.o','-pxt-config.o'))]
 need(len(remove)==2,'exact baseline caller/config object replacement')
 link=[x for x in link if x not in remove]
 idx=link.index('-lgcc');link[idx:idx]=[*objects,(driver/'SAMDFlash-storage.o').relative_to(ROOT).as_posix(),'-lm','-lc']
 link[link.index('-T')+1]='pxt-storage-runtime/control.ld'
 oldmap='-Wl,--gc-sections,--no-undefined,-Map='+base.relative_to(ROOT).as_posix()+'/gc-control.map'
 need(link.count(oldmap)==1,'unique original map')
 link[link.index(oldmap)]='-Wl,--gc-sections,--no-undefined,-Map='+out.relative_to(ROOT).as_posix()+'/storage-control.map'
 link[-1]=(out/'storage-control.elf').relative_to(ROOT).as_posix()
 # GNU newlib math/string closure is separately recorded. This report is link
 # evidence only; its exact archives, members and notices require independent
 # admission before any runtime result qualifies.
 libraries={}
 for name in ['libm.a','libc.a','libgcc.a']:
  path=Path(command(['arm-none-eabi-g++','-mcpu=cortex-m4','-mthumb','-print-file-name='+name],'locate-'+name).decode().strip())
  need(path.is_absolute() and path.is_file(),'actual toolchain library missing')
  libraries[name]=dict(path=str(path),sha256=sha(path.read_bytes()))
  target=out/'toolchain-libraries'/name;target.parent.mkdir(exist_ok=True);target.write_bytes(path.read_bytes())
 for package in ['libnewlib-arm-none-eabi','libnewlib-dev','gcc-arm-none-eabi','gcc-13-base']:
  path=Path('/usr/share/doc')/package/'copyright'
  need(path.is_file(),'toolchain component notices required')
  (out/'notices'/('copyright-'+package)).write_bytes(path.read_bytes())
 shutil.copyfile('/usr/share/common-licenses/GPL-3',out/'notices/GPL-3')
 command(link,'link')
 image=(out/'storage-control.elf').relative_to(ROOT)
 need(not command(['arm-none-eabi-nm','-u',image],'undefined').strip(),'closed final image')
 command(['arm-none-eabi-nm','-S',image],'symbols')
 command(['arm-none-eabi-objdump','-h',image],'sections')
 command(['arm-none-eabi-objdump','-d',image],'disassembly')
 command(['arm-none-eabi-objcopy','-O','binary',image,(out/'storage-control.bin').relative_to(ROOT)],'binary')
 members=[]
 text=(out/'storage-control.map').read_text()
 for archive,member in sorted(set(re.findall(r'(\S+\.a)\(([^)]+)\)',text))):
  path=Path(archive);path=path if path.is_absolute() else ROOT/path
  need(path.name in libraries and sha(path.read_bytes())==libraries[path.name]['sha256'],'selected library identity')
  raw=command(['arm-none-eabi-ar','p',path,member],'archive-member-'+str(len(members)))
  candidates=[]
  if path.name in ['libm.a','libc.a']:
   stem=re.sub(r'^(?:lib_a|libc_a|libm_a)-','',member).removesuffix('.o')
   source_root=ROOT/'toolchain-source/extracted'
   for source in sorted((source_root/'newlib').rglob(stem+'.*')):
    relative=source.relative_to(source_root).as_posix()
    if source.suffix not in ['.c','.S','.s'] or not relative.startswith(('newlib/libm/','newlib/libc/')):continue
    if '/machine/' in relative and relative.split('/machine/',1)[1].split('/',1)[0]!='arm':continue
    if '/sys/' in relative and relative.split('/sys/',1)[1].split('/',1)[0]!='arm':continue
    data=source.read_bytes();target=out/'newlib-source-candidates'/relative
    target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(data)
    candidates.append(dict(path=relative,sha256=sha(data)))
   need(candidates,'no source candidates for actual selected newlib member: '+member)
  members.append(dict(archive=path.name,member=member,sha256=sha(raw),source_candidates=candidates))
 need(members,'real archive-member inventory')
 packages=command(['dpkg-query','-W','-f=${Package} ${Version} ${source:Package} ${source:Version}\n','gcc-arm-none-eabi','libnewlib-arm-none-eabi','libnewlib-dev','gcc-13-base'],'toolchain-packages').decode().splitlines()
 source_packages={}
 for path in sorted((ROOT/'toolchain-source').glob('*')):
  if path.is_file() and (path.name.endswith('.dsc') or '.tar.' in path.name):source_packages[path.name]=sha(path.read_bytes())
 need(len(source_packages)>=3,'original source package envelope')
 for path in sorted((ROOT/'toolchain-source/extracted').glob('COPYING*')):
  if path.is_file():(out/'notices'/path.name).write_bytes(path.read_bytes())
 report=dict(schema=1,status='ACTUAL_RAFFS_SETTINGS_GC_SERVICES_LINK_ONLY',source=gc.consumer.git(ROOT,'rev-parse','HEAD').decode().strip(),core=gc.consumer.PIN,framework=gc.sources.PIN,engine='79071f69cd6035a820cbdad93da7343d7d501e4d',storage_start=0x78000,storage_bytes=32768,persistence='object-remount-same-machine-backing',elf_sha256=sha((out/'storage-control.elf').read_bytes()),binary_sha256=sha((out/'storage-control.bin').read_bytes()),staged={n:sha(b) for n,b in selected.items()},dependencies=dependencies,toolchain_libraries=libraries,archive_members=members,toolchain_packages=packages,source_packages=source_packages,baseline_report_sha256=sha((base/'build-report.json').read_bytes()),driver_report_sha256=sha((driver/'compile-report.json').read_bytes()),guest_qualified=False,native_arcade=False,board_qualified=False,reset_persistence=False,power_loss_persistence=False,full_board=False,licence_clearance=False,performance_measured=False)
 (out/'build-report.json').write_text(json.dumps(report,indent=2)+'\n')
 print(json.dumps({k:v for k,v in report.items() if k not in ['staged','dependencies','archive_members','toolchain_libraries']}))

if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('core',type=lambda s:Path(s).resolve());p.add_argument('clean_core',type=lambda s:Path(s).resolve());p.add_argument('framework',type=lambda s:Path(s).resolve());p.add_argument('generator',type=lambda s:Path(s).resolve());p.add_argument('out',type=lambda s:Path(s).resolve());a=p.parse_args();run(a.core,a.clean_core,a.framework,a.generator,a.out)
