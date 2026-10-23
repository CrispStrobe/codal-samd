#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Current pinned full-board source census; no preprocessor/compiler/guest."""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('historical_source_helpers', ROOT / 'tools/storage-link-census/census.py')
h = importlib.util.module_from_spec(spec)
spec.loader.exec_module(h)
ENGINE = '79071f69cd6035a820cbdad93da7343d7d501e4d'
CODAL = '382579520832962ff4eb415c19b638a6d899e605'
CORE = '2ed19d64ec3359143409a4b0a9758b4aeef3ee7d'
OWN = {
 'inc/ZPin.h': ['"pwmout_api.h"', '"SAMDEIC.h"'],
 'inc/ZSPI.h': ['"SAMDDMAC.h"', '<hal_spi_m_sync.h>'],
 'inc/ZI2C.h': ['"sercom.h"', '"hal_i2c_m_sync.h"'],
 'inc/ZSingleWireSerial.h': ['"SAMDDMAC.h"', '"hal_usart_async.h"', '"sercom.h"'],
 'inc/SAMDSerial.h': ['"hal_usart_async.h"', '"sercom.h"', '"SAMDDMAC.h"'],
 'inc/SAMDDMAC.h': ['"sam.h"'],
 'inc/platform_includes.h': ['"sam.h"'],
 'CMakeLists.txt': ['set(ASF ', '${ASF}/hal/src/hal_spi_m_sync.c', '${ASF}/hpl/sercom/hpl_sercom.c'],
 'src/SAMDUSB.cpp': ['PORT->Group', 'GCLK->PCHCTRL', 'MCLK->AHBMASK', 'USB->HOST.CTRLA.bit.SWRST', 'USB_FUSES_TRANSN_ADDR', 'USB->HOST.DESCADD', 'USB->DEVICE.DeviceEndpoint'],
}
PUBLIC = {
 'pxtapp/pxt.h': ['"pxtbase.h"', '"pins.h"', 'CONFIG_ENABLED(DEVICE_USB)', '"hf2.h"', '"hf2dbg.h"', '"HIDMouse.h"', '"HIDKeyboard.h"', '"HIDJoystick.h"'],
 'pxtapp/pxtbase.h': ['"platform.h"'],
 'pxtapp/platform.h': ['"'+n+'"' for n in ['ZPin.h','ZSPI.h','ZI2C.h','ZSingleWireSerial.h','SAMDNVM.h','SAMDDAC.h','SAMDPDM.h','SAMDSerial.h','pinmap.h']],
 'pxtapp/core---samd/usb.cpp': ['CodalUSB', 'HF2'],
}

def git(repo, rev, path):
 return subprocess.check_output(['git','-C',str(repo),'show',rev+':'+path])

def witness(raw, tokens):
 lines = raw.decode().splitlines()
 out = {t: [i+1 for i,l in enumerate(lines) if t in l] for t in tokens}
 h.sources.need(all(out.values()), 'missing exact source witness')
 return dict(sha256=h.sha(raw), bytes=len(raw), source_lines=out)

def config(raw, name):
 # Only explicit literal defaults are admitted, never a guessed generated config.
 rows = re.findall(r'^#define\s+'+name+r'\s+([01])\s*$',raw.decode(),re.M)
 h.sources.need(len(rows)==1,'one literal configuration value')
 return int(rows[0])

def projection_uses(payloads, token):
 return {p: [i+1 for i,l in enumerate(raw.decode().splitlines()) if re.search(r'\b'+token+r'\b',l,re.I)] for p,raw in payloads.items() if re.search(r'\b'+token+r'\b',raw.decode(),re.I)}

def build(stage, engine, core):
 profile=h.sources.load(ROOT/'profiles/pxt-framework-sources.json')
 payloads=h.public_payloads(stage,profile)
 own={p:witness(git(ROOT,CODAL,p),ts) for p,ts in OWN.items()}
 public={p:witness(payloads[p],ts) for p,ts in PUBLIC.items()}
 descriptor=git(engine,ENGINE,h.DESCRIPTOR)
 native={p:witness(git(engine,ENGINE,p),ts) for p,ts in {
 'crates/core/src/peripherals/sam/nvmctrl.rs':['pub struct'],
 'crates/core/src/peripherals/sam/cmcc.rs':['pub struct']}.items()}
 defaults=git(core,CORE,'inc/core/CodalConfig.h');service=(ROOT/'runtime-services/platform_includes.h').read_bytes()
 return dict(schema=1,status='CURRENT_FULLBOARD_SOURCE_GAP_CENSUS_ONLY',
  framework=profile['package'],projected_public_files=len(payloads),
  codal=dict(commit=CODAL,inputs=own),core=dict(commit=CORE,config_sha256=h.sha(defaults)),
  public_inputs=public,engine=dict(pin=ENGINE,path=h.DESCRIPTOR,sha256=h.sha(descriptor),descriptor=h.descriptor_observations(descriptor),native_model_sources=native),
  configuration=dict(core_defaults={k:config(defaults,k) for k in ['DEVICE_USB','DEVICE_WEBUSB']},
   qualified_service={k:config(service,k) for k in ['DEVICE_USB','DEVICE_WEBUSB']},service_sha256=h.sha(service),
   fullboard_generated_config='NOT_PRESENT_NOT_INFERRED',usb_one='CONDITIONAL_DESIRED_PROFILE_ONLY'),
  qspi=dict(projected_source_uses=projection_uses(payloads,'QSPI'),scope='64 pinned projected files only; blanket device headers are not runtime use'),
  vector_layout=dict(board_application=16384,qualified_service=0,compatibility='NOT_QUALIFIED'),
  excluded_dependencies=['ASF4 build sources/includes','sam.h device definitions','HAL/HPL SPI USART I2C ADC','pwmout and interrupt definitions'],
  next_steps=[
   'Freeze an actual generated full-board configuration and preprocessor/compiler dependency closure before selecting replacements.',
   'Replace device/HAL header and implementation dependencies with independently authored permitted simulation interfaces, preserving callers and notices.',
   'For conditional USB=1, implement and qualify reset/clock/calibration/descriptors/endpoints/interrupt semantics before claiming USB operation.',
   'Do not implement QSPI from a descriptor stub alone: first establish an actual selected source use.',
   'Admit application vector/startup layout and every selected peripheral in an affected clean guest before full-board claims.'],
  compiled=False,linked=False,executed=False,fullboard=False,usb_qualified=False,qspi_required=False,licence_clearance=False)

def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('stage',type=Path);a.add_argument('engine',type=Path);a.add_argument('core',type=Path)
 g=a.add_mutually_exclusive_group(required=True);g.add_argument('--output',type=Path);g.add_argument('--check',type=Path);x=a.parse_args()
 raw=(json.dumps(build(x.stage,x.engine,x.core),sort_keys=True,indent=2)+'\n').encode()
 if x.output:
  with x.output.open('xb') as f:f.write(raw)
 else:h.sources.need(x.check.read_bytes()==raw,'committed current census differs')
 print('CURRENT_FULLBOARD_SOURCE_CENSUS_ONLY; no compiler, guest, board or licence admission')
if __name__=='__main__':main()
