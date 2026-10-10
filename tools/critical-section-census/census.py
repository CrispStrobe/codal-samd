# SPDX-License-Identifier: MIT
"""Bind a bounded source census to retained diagnostic evidence, never admission."""
import argparse
import hashlib
import json
from pathlib import PurePosixPath
import re
import zipfile

REPORT_SHA256 = 'bb2429fc32505adbb50fc790592cd3a23d5728cdc438e7d2c2c7a7eca2746c1e'
CONFIG_SHA256 = '1d812a7253fb632cc30537cb34141a1394778c845918172ff5cd36e4953727d8'
PINS = {
    'codal-core': '312ae57e0b31f5b9df07a81e9d846945828e3c5a',
    'codal-samd': '5bd6b93c219c7e784e885ba2d6812809fb6289a8',
    'codal-itsybitsy-m4': '6ecd80ccf1abcc126d06653f178ca03a4d1c6421',
}
# These are reviewed public source hashes, not a licence allowlist.
FILES = {
    'codal-core/source/JACDAC/JACDAC.cpp': '37519d61403d9a13b71828fd81b8046da978e14aacefc560259941ce953e948d',
    'codal-core/source/JACDAC/JDPhysicalLayer.cpp': '0dc5dcf7836d57ff20ce2f49133dd59abddd50fa8b7311a7f49c5913eeb08c1d',
    'codal-core/source/core/CodalDmesg.cpp': '1b4d76a7549e900b41cd0909f4edf3625dc271ca90f9cfa81137b459ce21c44b',
    'codal-core/source/core/CodalFiber.cpp': '814cec34248f23d02345be7fc3def0eb2cc3606b44b88e5202fe03ec89e4d0fd',
    'codal-core/source/core/CodalHeapAllocator.cpp': 'c0f5cc8734a5216dec884da42d32ddf9281fc23afab1ea29fdbf570101bd87f4',
    'codal-core/source/core/codal_default_target_hal.cpp': '5952fb6be268ccaa0dfff5b08d10d62b74f79bdfd2729d6d524125a70338c7ef',
    'codal-core/source/driver-models/Serial.cpp': '0c7e72d551776e42c10e61cc338403967d2a0b39e143f48f2e7daaf90d21fcfa',
    'codal-core/source/driver-models/Timer.cpp': 'd7d1e3ee896a0691093a5919dce389f169b3e72bc606a4cf7bd103f241ef1128',
    'codal-core/source/drivers/MessageBus.cpp': '5df0bfc13e3c1d7f8e9a1ce9493390ee26109d1eaf774ca0c943fb2d03cfb823',
    'codal-core/inc/core/codal_target_hal.h': 'dad862ea056054f10a240579a845608cb832ec5222133ef5b7d98ea4615d424d',
    'codal-itsybitsy-m4/src/codal_target_hal.cpp': 'db423a7fad6d0940343099096fa57408820d39f6b4fc362c9c42dbc6ce2dc750',
    'codal-samd/src/DmaInstance.cpp': 'ed37fd8691af5e26e57cf7c8252c8bd55508bba239f530aabf90ef9054229251',
    'codal-samd/src/SAMDEIC.cpp': 'b8a0a1b61b86c26413b4713ed34b2e65401f09c47fa39bb08c037e2924a8f314',
    'codal-samd/src/ZPin.cpp': '813dd95f55592183e74dcae8a5b8d956860bcaeffa595ac32cd14e5525a1ecee',
    'codal-samd/inc/neopixel.h': 'ed867bb9c65db347f860390a8908f0f034e77eebde6fe6a8a72a90b4c167cb41',
}
UNRESOLVED = [
    'effective-preprocessor-paths', 'request-generated-and-assembly-callers',
    'selected-component-notices', 'retained-and-executed-callers',
    'simulation-build-selection', 'actual-integrated-caller-guests',
]


def closed_object(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError('duplicate JSON key')
        result[key] = value
    return result


def public_path(value):
    if not isinstance(value, str) or re.fullmatch(r'[A-Za-z0-9_./+-]+', value) is None:
        raise ValueError('nonpublic input path')
    path = PurePosixPath(value)
    if path.is_absolute() or '..' in path.parts or str(path) != value:
        raise ValueError('nonpublic input path')
    if not value.startswith(('libraries/', 'pxtapp/')):
        raise ValueError('unexpected source namespace')
    return value


def bind_bytes(data, expected):
    if hashlib.sha256(data).hexdigest() != expected:
        raise ValueError('retained evidence digest mismatch')


def summarize(report):
    """Pure semantic checks; CLI additionally binds exact original report bytes."""
    if type(report) is not dict or type(report.get('schema')) is not int or report['schema'] != 1:
        raise ValueError('unsupported diagnostic schema')
    if report.get('completeSourceToBinaryProof') is not False:
        raise ValueError('diagnostic census cannot establish binary proof')
    units = report.get('units')
    files = report.get('files')
    if type(units) is not list or len(units) != 172 or type(files) is not dict:
        raise ValueError('unexpected diagnostic input set')
    by_source = {}
    for unit in units:
        source = public_path(unit['source'])
        if source in by_source:
            raise ValueError('duplicate diagnostic source')
        if type(unit.get('dependencies')) is not list or any(type(p) is not str for p in unit['dependencies']):
            raise ValueError('invalid diagnostic dependency list')
        by_source[source] = unit
    selected = {}
    for path, digest in sorted(FILES.items()):
        source = 'libraries/' + path
        dependency = 'build-source/' + source
        entry = files.get(dependency)
        if not isinstance(entry, dict) or entry.get('sha256') != digest:
            raise ValueError('missing or mismatched reviewed source/header')
        if entry.get('origin') != 'build-source' or entry.get('path') != source:
            raise ValueError('source/header origin mismatch')
        if path.endswith('.cpp'):
            unit = by_source.get(source)
            if unit is None or unit.get('sourceSha256') != digest:
                raise ValueError('source-unit binding mismatch')
        consumers = sorted(source for source, unit in by_source.items()
                           if dependency in unit['dependencies'])
        selected[path] = {'sha256': digest, 'diagnostic_consumers': consumers}
    config = files.get('build-source/build/codal_extra_definitions.h', {})
    if config.get('sha256') != CONFIG_SHA256 or type(config.get('bytes')) is not int or config['bytes'] != 1634:
        raise ValueError('generated configuration binding mismatch')
    return {'schema': 1, 'status': 'SOURCE_CENSUS_ONLY', 'pins': PINS.copy(),
            'diagnostic_report_sha256': REPORT_SHA256,
            'configuration_sha256': CONFIG_SHA256, 'files': selected,
            'runtime_admission': False, 'runtime_build_selection': False,
            'active_call_proof': False, 'unresolved': list(UNRESOLVED)}


def read_archive(path):
    # No extraction, network, compiler, ELF inspection or execution.
    with zipfile.ZipFile(path) as archive:
        names = archive.namelist()
        needed = {'diagnostic-dependencies.json': (REPORT_SHA256, 3_000_000),
                  'generated/build/codal_extra_definitions.h': (CONFIG_SHA256, 1634)}
        content = {}
        for name, (digest, maximum) in needed.items():
            if names.count(name) != 1:
                raise ValueError('missing or duplicate retained evidence member')
            if archive.getinfo(name).file_size > maximum:
                raise ValueError('evidence member exceeds closed size bound')
            data = archive.read(name)
            bind_bytes(data, digest)
            content[name] = data
    return summarize(json.loads(content['diagnostic-dependencies.json'], object_pairs_hook=closed_object))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('archive', help='already retained original evidence ZIP; no automatic acquisition')
    args = parser.parse_args()
    try:
        print(json.dumps(read_archive(args.archive), indent=2, sort_keys=True))
    except (ValueError, OSError, KeyError, TypeError, zipfile.BadZipFile):
        parser.exit(2, 'source census refused: missing, malformed or mismatched retained evidence\n')


if __name__ == '__main__':
    main()
