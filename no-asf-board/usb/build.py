#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Hosted coherent USB1 core/driver leaf; never runs a compiler locally or guest."""
from pathlib import Path
import hashlib,importlib.util,json,os,re,shutil,subprocess,sys
ROOT=Path(__file__).resolve().parents[2]
CORE='2ed19d64ec3359143409a4b0a9758b4aeef3ee7d'
UNITS=['source/driver-models/Timer.cpp','source/core/CodalComponent.cpp',
 'source/core/CodalHeapAllocator.cpp','source/core/CodalFiber.cpp','source/core/CodalListener.cpp',
 'source/drivers/MessageBus.cpp','source/core/MemberFunctionCallback.cpp','source/types/Event.cpp',
 'source/driver-models/CodalUSB.cpp','source/core/CodalDmesg.cpp','source/core/CodalCompat.cpp']
def sha(raw):return hashlib.sha256(raw).hexdigest()
def need(ok,reason):
 if not ok:raise ValueError(reason)
def git(folder,*args):return subprocess.check_output(['git','-C',str(folder),*args])
def run(core,generator,board,out):
 need(os.environ.get('GITHUB_ACTIONS')=='true','ARM compile is hosted CI only')
 for path in [core,generator,board,out]:need(path.is_relative_to(ROOT) and path!=ROOT and ' ' not in str(path),'owned in-checkout paths')
 need(not out.exists() and not git(ROOT,'status','--porcelain','--untracked-files=no'),'fresh output and clean candidate')
 need(git(core,'rev-parse','HEAD').decode().strip()==CORE and not git(core,'status','--porcelain','--untracked-files=no'),'clean exact core')
 out.mkdir();commands=[]
 def command(args,label):
  args=[str(x) for x in args];p=subprocess.run(args,cwd=ROOT,capture_output=True,timeout=180)
  commands.append(dict(label=label,argv=args,returncode=p.returncode));(out/'commands.json').write_text(json.dumps(commands,indent=2)+'\n')
  (out/(label+'.stdout')).write_bytes(p.stdout);(out/(label+'.stderr')).write_bytes(p.stderr)
  need(p.returncode==0,'actual USB leaf command failed: '+label);return p.stdout
 spec=importlib.util.spec_from_file_location('source_configuration',ROOT/'tools/generated-board-profile/verify.py');cfg=importlib.util.module_from_spec(spec);spec.loader.exec_module(cfg)
 configuration=cfg.verify(generator,board,out)
 patches=[ROOT/'runtime-services'/n for n in ['fiber-guards.patch','timer-width.patch','heap-domains.patch']]
 command(['git','-C',core,'apply','--check',*patches],'patch-check')
 command(['git','-C',core,'apply',*patches],'apply-reviewed-service-patches')
 need(git(core,'diff','--name-only').decode().splitlines()==['inc/driver-models/Timer.h','source/core/CodalFiber.cpp','source/core/CodalHeapAllocator.cpp','source/driver-models/Timer.cpp'],'exact inherited service adaptations')
 (out/'actual-core-patch.diff').write_bytes(git(core,'diff','--binary'))
 original=(ROOT/'src/SAMDUSB.cpp').read_bytes();before=b'#include "sam.h"';need(original.count(before)==1,'one vendor include adaptation')
 driver=out/'SAMDUSB.cpp';driver.write_bytes(original.replace(before,b'#include "samd51_usb_mmio.h"'))
 includes=[ROOT/'no-asf-board/usb',ROOT/'runtime-services',ROOT/'simulation/include',ROOT/'paired-irq/include']
 includes+=sorted({p.parent for p in (core/'inc').rglob('*.h')})
 flags=['-mcpu=cortex-m4','-mthumb','-mfloat-abi=softfp','-mfpu=fpv4-sp-d16',
  '-std=c++11','-O0','-g','-DNDEBUG','-ffreestanding','-fno-builtin','-fno-exceptions',
  '-fno-rtti','-fno-unwind-tables','-fno-asynchronous-unwind-tables','-fno-threadsafe-statics',
  '-ffunction-sections','-fdata-sections','-DBW_NO_ASF_USB_LEAF=1','-DBW_RUNTIME_TEST_MODE=0',
  '-include',str(out/'codal_extra_definitions.h')]
 for p in includes:flags+=['-I',str(p)]
 compiler_headers=Path(command(['arm-none-eabi-gcc','-print-file-name=include'],'compiler-include-root').decode().strip()).resolve()
 system_headers=Path('/usr/lib/arm-none-eabi/include').resolve()
 selected={};objects=[]
 def admit(path):
  path=path.resolve();raw=path.read_bytes()
  if path.is_relative_to(ROOT):
   name=path.relative_to(ROOT).as_posix()
   need(not any(part in ['asf4','samd-peripherals','CMSIS'] for part in path.parts),'rejected vendor/header source')
   need(path.is_relative_to(core) or path.is_relative_to(out) or name.startswith(('no-asf-board/usb/','runtime-services/','simulation/include/','paired-irq/include/','asm/')),'unselected own input: '+name)
  else:
   need(path.is_relative_to(compiler_headers) or path.is_relative_to(system_headers),'unreviewed external header')
   name='toolchain/'+('gcc/' if path.is_relative_to(compiler_headers) else 'newlib/')+path.relative_to(compiler_headers if path.is_relative_to(compiler_headers) else system_headers).as_posix()
  need(name not in selected or selected[name]==sha(raw),'stable dependency bytes');selected[name]=sha(raw)
  dest=out/'selected-dependencies'/name;dest.parent.mkdir(parents=True,exist_ok=True);dest.write_bytes(raw)
 sources=[core/p for p in UNITS]+[ROOT/'runtime-services'/n for n in ['clock.cpp','target.cpp','aligned-heap.cpp']]+[ROOT/'no-asf-board/usb/probe.cpp',driver]
 for i,source in enumerate(sources):
  obj=out/f'unit-{i}.o';dep=out/f'unit-{i}.d';objects.append(obj)
  command(['arm-none-eabi-g++',*flags,'-Wall','-MD','-MF',dep,'-c',source,'-o',obj],f'compile-{i}')
  logical=dep.read_text().replace('\\\n',' ');target,sep,inputs=logical.partition(':');need(sep and ':' not in inputs and '\\' not in inputs,'closed compiler dependency rule')
  for path in sorted(set(inputs.split())):admit(ROOT/path)
 for i,source in enumerate([ROOT/'no-asf-board/usb/startup.S',ROOT/'asm/CortexContextSwitch.s']):
  obj=out/f'assembly-{i}.o';objects.append(obj)
  need(not re.search(rb'^\s*(?:\.include\b|#\s*include\b)',source.read_bytes(),re.M),'assembly has no unclosed includes');admit(source)
  command(['arm-none-eabi-gcc','-mcpu=cortex-m4','-mthumb','-mfloat-abi=softfp','-mfpu=fpv4-sp-d16','-c',source,'-o',obj],f'assembly-{i}')
 linker=ROOT/'no-asf-board/usb/control.ld';admit(linker)
 command(['arm-none-eabi-g++','-mcpu=cortex-m4','-mthumb','-mfloat-abi=softfp','-mfpu=fpv4-sp-d16','-nostdlib','-nostartfiles','-Wl,--gc-sections,--no-undefined,-Map='+str(out/'usb-leaf.map'),'-T',linker,*objects,'-lc','-lgcc','-o',out/'usb-leaf.elf'],'link')
 need(not command(['arm-none-eabi-nm','-u',out/'usb-leaf.elf'],'undefined').strip(),'closed actual core/driver image')
 command(['arm-none-eabi-nm','-S',out/'usb-leaf.elf'],'symbols')
 command(['arm-none-eabi-objdump','-d',out/'usb-leaf.elf'],'disassembly')
 command(['arm-none-eabi-objcopy','-O','binary',out/'usb-leaf.elf',out/'usb-leaf.bin'],'binary')
 cases={}
 base_members=set(re.findall(r'(\S+\.a)\(([^)]+)\)',(out/'usb-leaf.map').read_text()))
 case_rules={
  'bulk':None,
  'interrupt':dict(kind='probe-flag',flag='-DBW_USB_LEAF_INTERRUPT=1'),
  'hidden-webusb':dict(kind='probe-flag',flag='-DBW_USB_LEAF_HIDE_WEBUSB=1'),
  'missing-irq':dict(kind='driver',before='    NVIC_EnableIRQ(USB_0_IRQn);',after='    // isolated actual-driver NVIC80 enable omitted'),
  'missing-in-arm':dict(kind='driver',before='    USB->DEVICE.DeviceEndpoint[ep].EPSTATUSSET.reg = USB_DEVICE_EPSTATUSSET_BK1RDY;',after='    // isolated actual-driver IN bank arm omitted'),
  'bad-descriptor':dict(kind='driver',before='    USB->HOST.DESCADD.reg = (uint32_t)(&usb_endpoints[0]);',after='    USB->HOST.DESCADD.reg = (uint32_t)(&usb_endpoints[0]) + 2;'),
 }
 for name,rule in case_rules.items():
  case=out/'cases'/name;case.mkdir(parents=True)
  case_objects=list(objects);case_flags=list(flags);mutation=None
  if rule:
   if rule['kind']=='probe-flag':
    source=ROOT/'no-asf-board/usb/probe.cpp';case_flags.append(rule['flag']);mutation=dict(compile_flag=rule['flag'])
   else:
    before=rule['before'].encode();after=rule['after'].encode();raw=driver.read_bytes();need(raw.count(before)==1,'one exact driver source mutant: '+name)
    source=case/'SAMDUSB.cpp';source.write_bytes(raw.replace(before,after));mutation=dict(before_sha256=sha(before),after_sha256=sha(after),occurrences=1,source_sha256=sha(source.read_bytes()))
   obj=case/'changed.o';dep=case/'changed.d';index=sources.index(ROOT/'no-asf-board/usb/probe.cpp' if rule['kind']=='probe-flag' else driver);case_objects[index]=obj
   command(['arm-none-eabi-g++',*case_flags,'-Wall','-MD','-MF',dep,'-c',source,'-o',obj],'case-'+name+'-compile')
   logical=dep.read_text().replace('\\\n',' ');target,sep,inputs=logical.partition(':');need(sep and ':' not in inputs and '\\' not in inputs,'closed case dependency rule')
   for path in sorted(set(inputs.split())):admit(ROOT/path)
   command(['arm-none-eabi-g++','-mcpu=cortex-m4','-mthumb','-mfloat-abi=softfp','-mfpu=fpv4-sp-d16','-nostdlib','-nostartfiles','-Wl,--gc-sections,--no-undefined,-Map='+str(case/'usb-leaf.map'),'-T',linker,*case_objects,'-lc','-lgcc','-o',case/'usb-leaf.elf'],'case-'+name+'-link')
   need(not command(['arm-none-eabi-nm','-u',case/'usb-leaf.elf'],'case-'+name+'-undefined').strip(),'closed variant image')
   command(['arm-none-eabi-objcopy','-O','binary',case/'usb-leaf.elf',case/'usb-leaf.bin'],'case-'+name+'-binary')
   need(set(re.findall(r'(\S+\.a)\(([^)]+)\)',(case/'usb-leaf.map').read_text()))==base_members,'same explicitly admitted library members for each case')
  else:
   for suffix in ['elf','bin','map']:shutil.copyfile(out/('usb-leaf.'+suffix),case/('usb-leaf.'+suffix))
  command(['arm-none-eabi-nm','-S',case/'usb-leaf.elf'],'case-'+name+'-symbols')
  command(['arm-none-eabi-objdump','-d',case/'usb-leaf.elf'],'case-'+name+'-disassembly')
  cases[name]=dict(elf_sha256=sha((case/'usb-leaf.elf').read_bytes()),binary_sha256=sha((case/'usb-leaf.bin').read_bytes()),mutation=mutation,guest_qualified=False)
 members=sorted(set(re.findall(r'(\S+\.a)\(([^)]+)\)',(out/'usb-leaf.map').read_text())))
 notice=out/'notices';notice.mkdir()
 for package in ['gcc-arm-none-eabi','gcc-13-base','libnewlib-arm-none-eabi','libnewlib-dev']:
  source=Path('/usr/share/doc')/package/'copyright';need(source.is_file(),'required component notices');shutil.copyfile(source,notice/('copyright-'+package))
 shutil.copyfile('/usr/share/common-licenses/GPL-3',notice/'GPL-3')
 for name in ['COPYING.RUNTIME','COPYING3']:shutil.copyfile(ROOT/'runtime-services/reviewed-toolchain'/name,notice/name)
 shutil.copyfile(ROOT/'LICENSE',notice/'codal-samd-LICENSE');shutil.copyfile(core/'LICENSE',notice/'codal-core-LICENSE')
 inventory=[];libraries={};source_root=ROOT/'toolchain-source/extracted'
 need(source_root.is_dir(),'exact newlib source package required for selected-member admission')
 for archive,member in members:
  archive=Path(archive);archive=archive if archive.is_absolute() else ROOT/archive
  need(archive.name in ['libc.a','libgcc.a'],'only explicitly selected toolchain libraries')
  libraries[archive.name]=dict(path=str(archive),sha256=sha(archive.read_bytes()))
  target=out/'toolchain-libraries'/archive.name;target.parent.mkdir(exist_ok=True);target.write_bytes(archive.read_bytes())
  raw=command(['arm-none-eabi-ar','p',archive,member],f'archive-member-{len(inventory)}');candidates=[]
  if archive.name=='libc.a':
   stem=re.sub(r'^(?:lib_a|libc_a)-','',member).removesuffix('.o')
   for source in sorted((source_root/'newlib').rglob(stem+'.*')):
    relative=source.relative_to(source_root).as_posix()
    if source.suffix not in ['.c','.S','.s'] or not relative.startswith('newlib/libc/'):continue
    if '/machine/' in relative and relative.split('/machine/',1)[1].split('/',1)[0]!='arm':continue
    if '/sys/' in relative and relative.split('/sys/',1)[1].split('/',1)[0]!='arm':continue
    data=source.read_bytes();dest=out/'newlib-source-candidates'/relative;dest.parent.mkdir(parents=True,exist_ok=True);dest.write_bytes(data);candidates.append(dict(path=relative,sha256=sha(data)))
   need(candidates,'actual libc member source candidates required: '+member)
  inventory.append(dict(archive=archive.name,member=member,sha256=sha(raw),source_candidates=candidates))
 packages=command(['dpkg-query','-W','-f=${Package} ${Version} ${source:Package} ${source:Version}\n','gcc-arm-none-eabi','gcc-13-base','libnewlib-arm-none-eabi','libnewlib-dev'],'toolchain-packages').decode().splitlines()
 command(['arm-none-eabi-g++','--version'],'compiler')
 source_packages={p.name:sha(p.read_bytes()) for p in (ROOT/'toolchain-source').glob('*') if p.is_file() and (p.name.endswith('.dsc') or '.tar.' in p.name)}
 need(len(source_packages)>=3,'original newlib source envelope')
 for source in source_root.glob('COPYING*'):
  if source.is_file():shutil.copyfile(source,notice/source.name)
 report=dict(schema=1,status='USB1_ACTUAL_CORE_DRIVER_LEAF_LINK_ONLY',source=git(ROOT,'rev-parse','HEAD').decode().strip(),core=CORE,configuration=configuration,
  cases=cases,original_driver_sha256=sha(original),adapted_driver_sha256=sha(driver.read_bytes()),dependencies=selected,system_header_roots=dict(gcc=str(compiler_headers),newlib=str(system_headers)),cpp_units=len(sources),assembly_units=2,assembly_include_closure=True,
  abi='cortex-m4/thumb/softfp/fpv4-sp-d16',optimization='O0_leaf_not_historical_O2',vectors=16384,authored_boot_vector_bytes=8,
  elf_sha256=sha((out/'usb-leaf.elf').read_bytes()),binary_sha256=sha((out/'usb-leaf.bin').read_bytes()),archive_members=inventory,toolchain_libraries=libraries,toolchain_packages=packages,source_packages=source_packages,reviewed_service_patches={p.name:sha(p.read_bytes()) for p in patches},
  guest_qualified=False,usb_qualified=False,webusb_qualified=False,native_arcade=False,full_board=False,licence_clearance=False,performance_measured=False)
 (out/'build-report.json').write_text(json.dumps(report,sort_keys=True,indent=2)+'\n');print(json.dumps(dict(status=report['status'],dependencies=len(selected),guest_qualified=False)))
if __name__=='__main__':
 need(len(sys.argv)==5,'core generator board output')
 run(*(Path(s).resolve() for s in sys.argv[1:]))
