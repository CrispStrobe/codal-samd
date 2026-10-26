#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Hosted interruption-role composition; parent remains link-only, never replayed."""
from pathlib import Path
import hashlib,json,re,subprocess,sys
ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT/'pxt-storage-runtime'));import build_storage as parent
PARENT=ROOT/'storage-evidence'
def sha(b):return hashlib.sha256(b).hexdigest()
def need(ok,reason):
 if not ok:raise ValueError(reason)
def run(core,clean,framework,generator,out,role):
 need(not out.exists() and out.is_relative_to(ROOT) and out!=ROOT,'fresh owned output')
 if role==0: parent.run(core,clean,framework,generator,PARENT)
 out.mkdir();base=json.loads((PARENT/'build-report.json').read_bytes());old=json.loads((PARENT/'commands.json').read_bytes())
 commands=[]
 def command(a,label):
  a=[str(x) for x in a];p=subprocess.run(a,cwd=ROOT,capture_output=True,timeout=180)
  commands.append(dict(label=label,argv=a,returncode=p.returncode));(out/'commands.json').write_text(json.dumps(commands,indent=2)+'\n')
  (out/(label+'.stdout')).write_bytes(p.stdout);(out/(label+'.stderr')).write_bytes(p.stderr)
  need(p.returncode==0,'new actual composition command failed: '+label)
 dependencies=dict(base['dependencies']);changes={}
 for label,oldsource,newsource in [('probe',ROOT/'pxt-storage-runtime/probe.cpp',ROOT/'pxt-storage-interruption/probe.cpp')]:
  a=list(next(c['argv'] for c in old if '-c' in c['argv'] and c['argv'][c['argv'].index('-c')+1]==oldsource.relative_to(ROOT).as_posix()))
  original_object=a[a.index('-o')+1];obj=(out/(label+'.o')).relative_to(ROOT).as_posix();changes[original_object]=obj
  a[a.index('-c')+1]=newsource.relative_to(ROOT).as_posix();a[a.index('-o')+1]=obj;a[a.index('-MF')+1]=(out/(label+'.d')).relative_to(ROOT).as_posix()
  a.insert(1,'-DBW_INTERRUPTION_ROLE='+str(role))
  command(a,label);dependencies.pop(oldsource.relative_to(ROOT).as_posix())
  if label=='probe':dependencies.pop('pxt-storage-runtime/storage-caller.cpp.inc')
  for path in parent.gc.consumer.dependencies((out/(label+'.d')).read_text()):
   p=(ROOT/path).resolve();need(p.is_relative_to(ROOT),'unselected external dependency')
   dependencies[path]=sha(p.read_bytes())
 link=[changes.get(a,a) for a in next(c['argv'] for c in old if c['label']=='link')]
 oldmap='-Wl,--gc-sections,--no-undefined,-Map=storage-evidence/storage-control.map';need(link.count(oldmap)==1,'exact parent map')
 link[link.index(oldmap)]='-Wl,--gc-sections,--no-undefined,-Map='+out.relative_to(ROOT).as_posix()+'/storage-control.map'
 link[-1]=(out/'storage-control.elf').relative_to(ROOT).as_posix();command(link,'link')
 image=(out/'storage-control.elf').relative_to(ROOT).as_posix()
 for a,label in [(['arm-none-eabi-nm','-u',image],'undefined'),(['arm-none-eabi-nm','-S',image],'symbols'),(['arm-none-eabi-objdump','-d',image],'disassembly'),(['arm-none-eabi-objcopy','-O','binary',image,(out/'storage-control.bin').relative_to(ROOT).as_posix()],'binary')]:command(a,label)
 need(not (out/'undefined.stdout').read_bytes().strip(),'closed reopen image')
 symbols=(out/'symbols.stdout').read_text().splitlines()
 required=['storage_canary','storage_stage','storage_result','runtime_status',
           'gc_probe_stage','gc_probe_checks','gc_static_init_entry','runtime_probe',
           'swap_context']
 for name in required:
  need(sum(line.split()[-1]==name for line in symbols if line.split())==1,
       'unique linked observation symbol before guest: '+name)
 for name in ['_ZN3pxt5raffs2FS6formatE','_ZN3pxt5raffs2FS8tryMountE']+(['_ZN8settings4_setE'] if role==1 else []):
  need(sum(line.split()[-1].startswith(name) for line in symbols if line.split())==1,
       'unique linked caller before guest: '+name)

 members={(Path(a).name,b) for a,b in re.findall(r'(\S+\.a)\(([^)]+)\)',(out/'storage-control.map').read_text())};known={(r['archive'],r['member']):r for r in base['archive_members']}
 need(members and members<=set(known),'only parent-admitted actual library members')
 for path,digest in dependencies.items():
  p=(ROOT/path).resolve();need(p.is_relative_to(ROOT),'owned dependency');raw=p.read_bytes();need(sha(raw)==digest,'actual dependency bytes')
  canonical=p.relative_to(ROOT);target=out/'selected-dependencies'/canonical;target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(raw)
 report={**base,'composition':'actual-raffs-interruption-atomic-boundary','parent_report_sha256':sha((PARENT/'build-report.json').read_bytes()),'elf_sha256':sha((out/'storage-control.elf').read_bytes()),'binary_sha256':sha((out/'storage-control.bin').read_bytes()),'dependencies':dependencies,'archive_members':[known[k] for k in sorted(members)],'parent_link_executed':True,'parent_guest_executed':False,'interruption_role':role,'engine':'21ab8403c6b9d32d3e28152b4da7ad6b2e2908ce','interruption_qualified':False,'physical_power_loss':False}
 (out/'build-report.json').write_text(json.dumps(report,indent=2)+'\n')
 print(json.dumps(dict(status='ACTUAL_INTERRUPTION_LINK_ONLY',elf_sha256=report['elf_sha256'],guest_qualified=False)))
if __name__=='__main__':
 need(len(sys.argv)==6,'core clean-core framework generator output')
 args=[Path(x).resolve() for x in sys.argv[1:]]
 root=args[-1];need(not root.exists(),'fresh role union');root.mkdir()
 # Build the closed parent only once; each role then reuses its admitted objects.
 for role in range(3):
  run(*args[:-1],root/str(role),role)
