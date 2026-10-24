#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Reproduce the exact source-only generated configuration; never build CODAL."""
import argparse, hashlib, json, re, subprocess, tempfile
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
def need(ok, reason):
    if not ok: raise ValueError(reason)
def sha(raw): return hashlib.sha256(raw).hexdigest()
def verify(sources, board, temporary):
    profile=json.loads((ROOT/'profiles/pxt-generated-board-profile.json').read_bytes())
    need(profile['schema']==1 and profile['status']=='GENERATED_CONFIGURATION_REPRODUCED_SOURCE_ONLY','profile schema')
    for flag in ['compiled','executed','usb_qualified','fullboard_qualified','licence_clearance']:
        need(profile[flag] is False,'source-only scope')
    repos={'codal':sources,'codal-itsybitsy-m4':board}
    for name, folder in repos.items():
        actual=subprocess.check_output(['git','-C',str(folder),'rev-parse','HEAD'],text=True).strip()
        need(actual==profile['source_pins'][name],'exact public source revision')
    for item in profile['public_inputs']:
        raw=(repos[item['repo']]/item['path']).read_bytes()
        need(len(raw)==item['bytes'] and sha(raw)==item['sha256'],'public input identity: '+item['path'])
        need(hashlib.sha1(b'blob '+str(len(raw)).encode()+b'\0'+raw).hexdigest()==item['blob'],'public Git blob')
    app=ROOT/'tools/generated-board-profile/empty-project-codal.json'
    need(sha(app.read_bytes())==profile['application_sha256'],'authored application identity')
    header=temporary/'codal_extra_definitions.h'
    result=subprocess.run(['cmake','-DSOURCE_ROOT='+str(sources.resolve()),'-DBOARD_ROOT='+str(board.resolve()),'-DAPP_CONFIG='+str(app),'-DHEADER_OUTPUT='+str(header),'-P',str(ROOT/'tools/generated-board-profile/reconstruct-config.cmake')],capture_output=True,text=True,timeout=30)
    need(result.returncode==0,'original source configuration functions failed: '+result.stderr)
    raw=header.read_bytes();need(len(raw)==profile['header']['bytes'] and sha(raw)==profile['header']['sha256'],'generated header differs')
    definitions=re.findall(rb'^ #define ([A-Za-z_0-9]+)\t (.*)$',raw,re.M)
    need(len(definitions)==len(set(k for k,v in definitions)),'unique final definition keys')
    values={k.decode():v.decode() for k,v in definitions}
    for key,value in profile['features'].items(): need(values.get(key)==value,'required generated feature: '+key)
    need(values['DEVICE_USB']=='1' and values['DEVICE_WEBUSB']=='1','enabled USB preserved')
    return {'status':profile['status'],'header_sha256':sha(raw),'definitions':len(definitions),'compiled':False,'executed':False}
def main():
    a=argparse.ArgumentParser(description=__doc__);a.add_argument('sources',type=Path);a.add_argument('board',type=Path);args=a.parse_args()
    with tempfile.TemporaryDirectory(prefix='generated-config-',dir=ROOT) as tmp:
        print(json.dumps(verify(args.sources,args.board,Path(tmp)),sort_keys=True))
if __name__=='__main__':main()
