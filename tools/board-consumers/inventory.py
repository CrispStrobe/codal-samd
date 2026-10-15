#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Project retained historical dependency/link metadata; never build or admit it."""
import argparse
import collections
import hashlib
import json
from pathlib import Path, PurePosixPath
import re
import zipfile

ARCHIVE_SHA = '85361c5387b4eddf7bb0622b64cc89c09f4f367e465e9b12a8a36723620b7f27'
ARCHIVE_BYTES = 948805
MAP_SHA = '2322e427353ba623b3efadbccfcef1a2ee50c09322d8016dc05b1b782bdb1f64'
ARTIFACT = 'https://api.github.com/repos/CrispStrobe/labwired-core/actions/artifacts/11559576651/zip'
ORIGIN_KINDS = {'captured-generated-header', 'pinned-git-blob',
                'original-request', 'toolchain-unreviewed'}


def need(condition, message):
    if not condition:
        raise ValueError(message)


def closed(pairs):
    result = {}
    for key, value in pairs:
        need(key not in result, 'duplicate JSON key')
        result[key] = value
    return result


def safe(value):
    need(isinstance(value, str) and bool(value), 'missing relative path')
    path = PurePosixPath(value)
    need(not path.is_absolute() and '..' not in path.parts and '\\' not in value
         and str(path) == value, 'noncanonical relative path')
    return value


def sha(value, width=64):
    need(isinstance(value, str) and re.fullmatch('[0-9a-f]{%d}' % width, value),
         'invalid digest')
    return value


def graph(diagnostic, origins):
    files, units, records = diagnostic['files'], diagnostic['units'], origins['records']
    need(set(files) == set(records), 'file/origin closure')
    file_ids = {name: index for index, name in enumerate(sorted(files))}
    seen_sources, seen_objects = set(), set()
    projected_units = []
    repeats = 0
    for unit in units:
        source, target = safe(unit['source']), safe(unit['target'])
        need(source not in seen_sources and target not in seen_objects,
             'duplicate unit source/object')
        seen_sources.add(source)
        seen_objects.add(target)
        source_key = 'build-source/' + source
        need(source_key in files, 'source missing from dependency inventory')
        need(sha(unit['sourceSha256']) == files[source_key]['sha256'], 'unit/source hash')
        deps = unit['dependencies']
        need(isinstance(deps, list) and source_key in deps, 'source dependency missing')
        repeats += len(deps) - len(set(deps))
        for dependency in sorted(set(deps)):
            need(dependency in files, 'unknown dependency')
        projected_units.append({'source': source, 'source_sha256': unit['sourceSha256'],
                                'object': target,
                                'dependency_file_ids': sorted(file_ids[x] for x in set(deps))})
    projected_files = []
    kinds = collections.Counter()
    for name in sorted(files):
        safe(name)
        item, origin = files[name], records[name]
        digest = sha(item['sha256'])
        need(digest == sha(origin['sha256']), 'file/origin hash')
        need(origin['kind'] in ORIGIN_KINDS, 'unknown origin kind')
        kinds[origin['kind']] += 1
        # Explicit allow-list: never copy raw invocations, host locations or source.
        selected_origin = {'kind': origin['kind']}
        if origin['kind'] == 'pinned-git-blob':
            selected_origin.update(component=safe(origin['component']),
                                   commit=sha(origin['commit'], 40),
                                   blob=sha(origin['blob'], 40), path=safe(origin['path']))
            url = origin['source']
            need(url.startswith('https://github.com/') and
                 '/blob/' + origin['commit'] + '/' in url, 'nonpublic source URL')
            selected_origin['source'] = url
        elif origin['kind'] == 'original-request':
            # Request keys use a virtual leading slash, not a host location.
            need(origin['requestKey'] == '/' + name.removeprefix('build-source/'),
                 'request source binding')
        projected_files.append({'path': name, 'sha256': digest, 'origin': selected_origin})
    return {'units': sorted(projected_units, key=lambda x: x['source']),
            'files': projected_files, 'origin_counts': dict(sorted(kinds.items())),
            'repeated_dependency_occurrences': repeats}


def storage_sections(text):
    """GNU map input .text sections ONLY, after the allocated-map marker.

    LOAD records, discarded sections, vtables and debug rows are not code.
    Addresses establish allocation, not execution or runtime call relationships.
    """
    marker = 'Linker script and memory map'
    need(text.count(marker) == 1, 'allocated-map boundary missing/ambiguous')
    allocated = text.split(marker, 1)[1].split('OUTPUT(', 1)[0]
    pattern = re.compile(
        r'^ (\.text\.[^\s]+)\s*\n?\s*(0x[0-9a-f]+)\s+'
        r'(0x[0-9a-f]+)\s+(\S+)\s*$', re.MULTILINE)
    result = []
    for match in pattern.finditer(allocated):
        section, address, size, obj = match.groups()
        if '/pxtapp/settings/' not in '/' + obj or not obj.endswith('.cpp.o'):
            continue
        address, size = int(address, 16), int(size, 16)
        need(0x2000 <= address < 0x80000 and 0 < size <= 0x80000 - address,
             'settings text outside historical flash range')
        safe(obj)
        result.append({'section': section, 'address': address, 'bytes': size, 'object': obj})
    need(len({x['section'] for x in result}) == len(result), 'duplicate settings section')
    return result


def inventory(path):
    raw = path.read_bytes()
    need(len(raw) == ARCHIVE_BYTES and hashlib.sha256(raw).hexdigest() == ARCHIVE_SHA,
         'retained archive identity')
    with zipfile.ZipFile(path) as archive:
        names = archive.namelist()
        need(len(names) == len(set(names)), 'duplicate archive member')
        for name in names:
            safe(name)

        def read(name):
            return json.loads(archive.read(name), object_pairs_hook=closed)

        diagnostic_raw = archive.read('diagnostic-dependencies.json')
        diagnostic, origins = read('diagnostic-dependencies.json'), read('dependency-origins.json')
        need(origins['diagnosticReportSha256'] == hashlib.sha256(diagnostic_raw).hexdigest(),
             'diagnostic/origin binding')
        for member, field in [('request.json', 'requestSha256'),
                              ('builder-manifest.json', 'builderManifestSha256')]:
            need(origins[field] == hashlib.sha256(archive.read(member)).hexdigest(),
                 'original source metadata binding')
        report = graph(diagnostic, origins)
        need(len(report['units']) == 172 and len(report['files']) == 542, 'closed census')
        need(report['repeated_dependency_occurrences'] == 51, 'dependency repetition census')
        map_raw = archive.read('original-link.map')
        link = read('original-link-evidence.json')
        need(hashlib.sha256(map_raw).hexdigest() == MAP_SHA == link['mapSha256']
             and len(map_raw) == link['mapBytes'] and link['originalUnchanged'] is True,
             'original map binding')
        sections = storage_sections(map_raw.decode('utf-8'))
        expected = {'_ZN5codal6ZFlash9totalSizeEv', '_ZN5codal6ZFlash10writeBytesEjPKvm',
                    '_ZN5codal6ZFlash8pageSizeEj', '_ZN5codal6ZFlash9erasePageEj'}
        selected = [x for x in sections if x['object'].endswith('/SAMDFlash.cpp.o')]
        need({x['section'].removeprefix('.text.') for x in selected} == expected,
             'four retained ZFlash routines')
        known_objects = {x['object'] for x in report['units']}
        need(all(x['object'] in known_objects for x in sections), 'unknown settings object')
        report.update(schema=1, status='HISTORICAL_MIGRATION_INVENTORY_ONLY',
                      artifact=ARTIFACT, archive_sha256=ARCHIVE_SHA, map_sha256=MAP_SHA,
                      allocated_settings_text=sections, assembly_coverage=False,
                      current_profile_compiled=False, image_admitted=False,
                      runtime_qualified=False, licence_clearance=False,
                      boundary='Historical dependency inclusion and allocated text are not calls, '
                      'new-core ABI compatibility, retained-inline provenance, an ASF-free build '
                      'or flash-command/runtime qualification. Raw source and host invocations '
                      'are deliberately excluded.')
        return report


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('archive', type=Path)
    parser.add_argument('--output', type=Path)
    args = parser.parse_args()
    payload = json.dumps(inventory(args.archive), indent=2, sort_keys=True) + '\n'
    if args.output:
        # Refuse accidental replacement of another worker's result.
        with args.output.open('x') as handle:
            handle.write(payload)
    else:
        print(payload, end='')
