#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Four isolated actual source mutants; specific actual guest wrong outcomes."""
from pathlib import Path
import hashlib,json,os,re,subprocess,sys
ROOT=Path(__file__).resolve().parents[1]
BASE=ROOT/'storage-evidence';ENGINE=ROOT/'owned-storage-engine'
def sha(b):return hashlib.sha256(b).hexdigest()
def need(ok,reason):
 if not ok:raise ValueError(reason)
POLICY={
 'omit-get-copy':('settings-storage.cpp',b'    s->fs.read(NULL, ret->data, ret->length);',b'    // isolated actual settings read copy omitted'),
 'omit-remove':('raffs-storage.cpp',b'    return write(keyName, NULL, M1);',b'    return 0; // isolated remove incorrectly reports success'),
 'omit-saved-root':('storage-caller.cpp.inc',b'auto saved=settings::_get(key); registerGCObj(saved);',b'auto saved=settings::_get(key); (void)saved; // isolated root omitted'),
 'omit-object-remount':('settings-storage.cpp',b'    delete settings::instWStorage;\n    settings::instWStorage = nullptr;',b'    (void)settings::instWStorage; // isolated old cache remains alive')}
def main():
 need(os.getenv('GITHUB_ACTIONS')=='true','hosted actual mutants only')
 profile=sys.argv[1];need(profile in ['default','event-scheduler'],'selected profile')
 out=ROOT/'storage-mutants';need(not out.exists(),'new output');out.mkdir()
 commands=json.loads((BASE/'commands.json').read_bytes());original_report=json.loads((BASE/'build-report.json').read_bytes())
 results=[]
 for kind,(name,before,after) in POLICY.items():
  child=out/kind;child.mkdir();source=ROOT/'pxt-storage-runtime'/name if name.endswith('.inc') else BASE/name
  raw=source.read_bytes();need(raw.count(before)==1,'unique isolated source site')
  changed=raw.replace(before,after,1);(child/name).write_bytes(changed)
  actual='probe.cpp' if kind=='omit-saved-root' else name
  original=ROOT/'pxt-storage-runtime'/actual if actual=='probe.cpp' else BASE/actual
  if kind=='omit-saved-root':(child/actual).write_bytes(original.read_bytes())
  compile_old=next(x for x in commands if '-c' in x['argv'] and x['argv'][x['argv'].index('-c')+1]==original.relative_to(ROOT).as_posix())
  argv=list(compile_old['argv']);oldobject=argv[argv.index('-o')+1]
  argv[argv.index('-c')+1]=(child/actual).relative_to(ROOT).as_posix();argv[argv.index('-o')+1]=(child/'changed.o').relative_to(ROOT).as_posix()
  argv[argv.index('-MF')+1]=(child/'changed.d').relative_to(ROOT).as_posix()
  def command(cmd,label):
   proc=subprocess.run(cmd,cwd=ROOT,capture_output=True,timeout=180)
   (child/(label+'.json')).write_text(json.dumps(dict(argv=cmd,returncode=proc.returncode))+'\n')
   (child/(label+'.stdout')).write_bytes(proc.stdout);(child/(label+'.stderr')).write_bytes(proc.stderr)
   need(proc.returncode==0,'actual mutant compile/link failed: '+kind+'/'+label)
  command(argv,'compile')
  link=list(next(c for c in commands if c['label']=='link')['argv']);need(link.count(oldobject)==1,'one affected object')
  link[link.index(oldobject)]=(child/'changed.o').relative_to(ROOT).as_posix()
  link[-1]=(child/'storage-control.elf').relative_to(ROOT).as_posix()
  oldmap='-Wl,--gc-sections,--no-undefined,-Map=storage-evidence/storage-control.map'
  need(link.count(oldmap)==1,'specific original map')
  link[link.index(oldmap)]='-Wl,--gc-sections,--no-undefined,-Map='+child.relative_to(ROOT).as_posix()+'/storage-control.map'
  command(link,'link')
  command(['arm-none-eabi-objcopy','-O','binary',(child/'storage-control.elf').relative_to(ROOT).as_posix(),(child/'storage-control.bin').relative_to(ROOT).as_posix()],'binary')
  report={**original_report,'elf_sha256':sha((child/'storage-control.elf').read_bytes()),'binary_sha256':sha((child/'storage-control.bin').read_bytes()),'mutation':dict(kind=kind,path=str(source.relative_to(ROOT)),original_sha256=sha(raw),mutated_sha256=sha(changed))}
  report['staged']=dict(original_report['staged'])
  if name in report['staged']:report['staged'][name]=sha(changed)
  # Retain the actual changed dependency census, including the separately
  # selected authored include for the omitted-root case.
  dependencies=dict(original_report['dependencies'])
  dependencies.pop(original.relative_to(ROOT).as_posix(),None)
  if kind=='omit-saved-root':dependencies.pop('pxt-storage-runtime/storage-caller.cpp.inc',None)
  logical=(child/'changed.d').read_text().replace('\\\n',' ')
  _,separator,inputs=logical.partition(':');need(separator and ':' not in inputs,'one actual dependency rule')
  for dependency in inputs.split():dependencies[dependency]=sha((ROOT/dependency).read_bytes())
  report['dependencies']=dependencies
  parent_members={(row['archive'],row['member']):row for row in original_report['archive_members']}
  active={(Path(archive).name,member) for archive,member in re.findall(r'(\S+\.a)\(([^)]+)\)',(child/'storage-control.map').read_text())}
  need(active and active<=set(parent_members),'omission mutant must retain admitted library members only')
  report['archive_members']=[parent_members[key] for key in sorted(active)]
  (child/'build-report.json').write_text(json.dumps(report,indent=2)+'\n')
  cmd=['cargo','test','--release','--locked','-p','labwired-core']
  if profile!='default':cmd+=['--features',profile]
  cmd+=['--test','bw_pxt_storage','--','--ignored','--exact','actual_raffs_settings_composition_and_remount','--nocapture']
  env={**os.environ,'BW_STORAGE_MUTANT':kind,'BW_GC_FIXTURE_DIR':str(child)}
  result=subprocess.run(cmd,cwd=ENGINE,env=env,capture_output=True,timeout=1200)
  (child/('guest-'+profile+'.stdout')).write_bytes(result.stdout);(child/('guest-'+profile+'.stderr')).write_bytes(result.stderr)
  marker=b'PXT_STORAGE_MUTANT_NO_REMOUNT ' if kind=='omit-object-remount' else b'PXT_STORAGE_MUTANT_WRONG '
  need(result.returncode==0 and result.stdout.count(marker)==1 and b'PXT_STORAGE_FINITE_PASS ' not in result.stdout,'specific actual wrong result not admitted: '+kind)
  results.append(dict(kind=kind,profile=profile,command=cmd,returncode=result.returncode,elf_sha256=report['elf_sha256'],log_sha256=sha(result.stdout+result.stderr),mutation=report['mutation']))
 (out/'mutation-report.json').write_text(json.dumps(results,indent=2)+'\n')
 print('FOUR_ACTUAL_STORAGE_SOURCE_MUTANTS_DETECTED '+profile)
if __name__=='__main__':main()
