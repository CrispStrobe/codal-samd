#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Strict public CF2 source/data admission. Never acquires or runs boot firmware."""
from pathlib import Path
import argparse,hashlib,json,re,struct
ROOT=Path(__file__).resolve().parents[2]
PROFILE=ROOT/'profiles/pxt-cf2-authored-metadata.json'
FALSE=['arm_compiled','guest_executed','runtime_reads_qualified','hf2_qualified','native_arcade','full_board','licence_clearance']
def need(ok,reason):
 if not ok:raise ValueError(reason)
def sha(raw):return hashlib.sha256(raw).hexdigest()
def blob(raw):return hashlib.sha1(b'blob '+str(len(raw)).encode()+b'\0'+raw).hexdigest()
def words(raw):
 text=raw.decode('ascii');parts=text.split('/* CF2 START */');need(len(parts)==2,'one CF2 start');parts=parts[1].split('/* CF2 END */');need(len(parts)==2,'one CF2 end')
 body=re.sub(r'//[^\n]*|/\*.*?\*/','',parts[0],flags=re.S);tokens=body.split(',');need(not tokens[-1].strip(),'explicit trailing comma');values=[]
 for token in tokens[:-1]:
  token=token.strip();need(re.fullmatch(r'(?:0x[0-9a-fA-F]+|[0-9]+)',token) is not None,'only original numeric CF2 literals');value=int(token,16 if token.startswith('0x') else 10);need(0<=value<2**32,'32-bit CF2 word');values.append(value)
 validate_words(values);return values

def validate_words(values):
 need(len(values)>=4 and values[:2]==[0x1e9e10f1,0x20227a79],'CF2 exact magic')
 used,total=values[2:4];need(type(used) is int and type(total) is int and 0<used<total<=100 and len(values)==4+2*total,'bounded complete CF2 pair table')
 pairs=list(zip(values[4::2],values[5::2]));keys=[k for k,v in pairs[:used]];need(all(type(k) is int and 0<k<2**31 for k in keys) and len(set(keys))==used,'nonzero unique used CF2 keys')
 need(all(type(v) is int and 0<=v<2**32 for v in values),'exact 32-bit words')
 need(all(p==(0,0) for p in pairs[used:]),'zero terminator and reserved entries')
 return dict(pairs[:used])

def segments(values):
 validate_words(values)
 return [(0x3000,struct.pack('<'+'I'*len(values),*values)),(0x3400,b'Authored no-ASF PyBadge metadata; no bootloader.\n\0'),(0x3ff0,struct.pack('<IIII',0x3000,0,0,0x3400))]

def validate_segments(rows,expected):
 need(len(rows)==3 and all(type(a) is int and isinstance(b,bytes) and b for a,b in rows),'three declared data fragments')
 need([a for a,b in rows]==[0x3000,0x3400,0x3ff0],'explicit reviewed metadata locations')
 end=8
 for address,raw in rows:need(end<=address and address+len(raw)<=0x4000,'nonoverlap below application, preserving reset words');end=address+len(raw)
 cf2,info,binfo=[b for a,b in rows];need(len(cf2)==816 and len(binfo)==16,'exact CF2/UF2 record lengths')
 decoded=list(struct.unpack('<204I',cf2));need(decoded==expected,'exact original CF2 words');validate_words(decoded)
 pointer,hid,msc,text=struct.unpack('<IIII',binfo);need(pointer==0x3000 and pointer%4==0 and hid==msc==0 and text==0x3400,'actual runtime pointer plus absent handover functions')
 need(0<len(info)<=128 and info[-1:]==b'\0' and b'\0' not in info[:-1] and info==segments(expected)[1][1],'bounded exact authored information string')
 return dict(cf2_bytes=len(cf2),used=decoded[2],total=decoded[3],uf2_information_bytes=len(binfo))

def admit(header,notice,profile):
 need(type(profile['schema']) is int and profile['schema']==1 and profile['status']=='AUTHORED_CF2_DATA_SOURCE_ONLY','source-only schema')
 for key in FALSE:need(profile[key] is False,'no broader qualification claim')
 source=profile['source'];need(source=={'repository': 'adafruit/uf2-samdx1', 'commit': 'd4dc92889759c0c551683420e24c5ef535ac303e', 'path': 'boards/arcade_pybadge/board_config.h', 'git_blob': '27187d69b2cea3079a55ab01c1daf1104f0f4478', 'sha256': '2ed85cf82b16b5aab5bfb26dff81b021a7a966b06a590e33b0037e7248d1b689', 'bytes': 3841},'immutable original public source identity');need(source['repository']=='adafruit/uf2-samdx1' and source['commit']=='d4dc92889759c0c551683420e24c5ef535ac303e' and source['path']=='boards/arcade_pybadge/board_config.h','exact public configuration selection')
 need(blob(header)==source['git_blob'] and sha(header)==source['sha256'] and len(header)==source['bytes'],'exact public source bytes')
 licence=profile['notice'];need(licence=={'path': 'LICENSE', 'git_blob': '7fffbab21e0e650b07b9c702d42c2a30368b0762', 'sha256': '1cc39bd3355e7580ab6e52b95a9faba53cc1b875af18211ad0f86266f1b12443'},'immutable original component notice identity');need(licence['path']=='LICENSE' and blob(notice)==licence['git_blob'] and sha(notice)==licence['sha256'],'exact original notice')
 need(notice==(ROOT/'no-asf-board/cf2/notices/uf2-samdx1-LICENSE').read_bytes(),'retained component notice')
 values=words(header);need(values==profile['words'] and values[2:4]==[62,100],'all exact original configuration values')
 table=validate_words(values);need({k:table[k] for k in [204,205,208,209]}=={204:0x80000,205:0x30000,208:0x239a0033,209:0x55114460},'actual public board memory/identities')
 rows=segments(values);summary=validate_segments(rows,values);need(profile['segments']==[dict(address=a,bytes=len(b),sha256=sha(b)) for a,b in rows],'source-bound authored fragment identities');return summary,rows

def main():
 p=argparse.ArgumentParser();p.add_argument('header',type=Path);p.add_argument('notice',type=Path);p.add_argument('--output',type=Path);a=p.parse_args();profile=json.loads(PROFILE.read_bytes());summary,rows=admit(a.header.read_bytes(),a.notice.read_bytes(),profile)
 if a.output:
  need(not a.output.exists(),'fresh authored data output');a.output.mkdir(parents=True)
  for address,raw in rows:(a.output/f'metadata-{address:04x}.bin').write_bytes(raw)
 summary.update(status=profile['status'],arm_compiled=False,guest_executed=False,native_arcade=False)
 print(json.dumps(summary,sort_keys=True))
if __name__=='__main__':main()
