#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Validate/stage exact public framework bytes, never generated/private inputs."""
import argparse
import hashlib
import json
from pathlib import Path, PurePosixPath
import subprocess

ROOT = Path(__file__).resolve().parents[2]
PIN = '31abf23d118f35010fb75122e60a1eb2b6dffe9f'
TREE = 'ca0b059d1142cdc91ae96181268d8e2d6de65e3c'
NOTICE = '4b1ad51b2f0efc36f38aa3349a9f30fbd9217547'
PROFILE_SHA = 'b617a5808bda23b33b52866049ea3b09fe5967efde9495b0d90033291a39484f'
FALSE_FLAGS = ['consumer_compiled', 'generated_inputs_qualified', 'header_abi_qualified',
               'image_admitted', 'licence_clearance']
UNRESOLVED = {'/codal.json', '/functions.json', '/pxtapp/main.cpp',
              '/pxtapp/pointers.cpp', '/pxtapp/pxtconfig.h'}
REQUIRED = {'/pxtapp/pxt.h': 'libs/core/pxt.h',
            '/pxtapp/pxtbase.h': 'libs/base/pxtbase.h',
            '/pxtapp/platform.h': 'libs/core---samd/platform.h',
            **{'/pxtapp/settings/' + name: 'libs/settings/' + name for name in
               ['Flash.h', 'RAFFS.h', 'SAMDFlash.cpp', 'RAFFS.cpp', 'settings.cpp']}}


def need(ok, reason):
    if not ok:
        raise ValueError(reason)


def pairs(items):
    result = {}
    for key, value in items:
        need(key not in result, 'duplicate JSON key')
        result[key] = value
    return result


def load(path):
    return json.loads(path.read_bytes(), object_pairs_hook=pairs)


def safe(name):
    need(isinstance(name, str) and bool(name), 'missing relative path')
    path = PurePosixPath(name)
    need(not path.is_absolute() and '..' not in path.parts and '\\' not in name
         and str(path) == name, 'unsafe/noncanonical relative path')
    return path


def identity(raw):
    return (hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest(),
            hashlib.sha256(raw).hexdigest(), len(raw))


def validate(profile):
    need(type(profile['schema']) is int and profile['schema'] == 1, 'schema')
    need(profile['status'] == 'PXT_FRAMEWORK_SOURCE_BINDINGS_ONLY', 'source-only status')
    for key in FALSE_FLAGS:
        need(profile[key] is False, 'broadened/mistyped qualification flag')
    package = profile['package']
    need(package == dict(name='pxt-common-packages', version='14.1.4',
                         repository='microsoft/pxt-common-packages', commit=PIN, tree=TREE,
                         notice_path='LICENSE', notice_git_blob=NOTICE), 'package/notice binding')
    need(profile['archive']['sha256'] ==
         '85361c5387b4eddf7bb0622b64cc89c09f4f367e465e9b12a8a36723620b7f27'
         and profile['archive']['request_sha256'] ==
         '19efcdc73769fdfdeb51aa215c528bebad59782cbc538f72f4a194326f1f42b1'
         and profile['archive']['artifact'] ==
         'https://api.github.com/repos/CrispStrobe/labwired-core/actions/artifacts/11559576651/zip',
         'original source-origin binding')
    rows = profile['sources']
    need(len(rows) == 64, 'public source census')
    requests, paths = set(), set()
    for row in rows:
        request, path = row['request'], row['path']
        need(isinstance(request, str) and request.startswith('/pxtapp/'), 'virtual request path')
        safe(request[1:])
        safe(path)
        need(path.startswith('libs/') and request not in UNRESOLVED, 'source selection')
        need(request not in requests and path not in paths, 'duplicate request/source alias')
        requests.add(request)
        paths.add(path)
        need(isinstance(row['git_blob'], str) and len(row['git_blob']) == 40
             and all(c in '0123456789abcdef' for c in row['git_blob']), 'Git blob shape')
        need(isinstance(row['sha256'], str) and len(row['sha256']) == 64
             and all(c in '0123456789abcdef' for c in row['sha256']), 'SHA256 shape')
        need(type(row['bytes']) is int and row['bytes'] >= 0, 'source byte count')
    need({row['request']: row['path'] for row in rows if row['request'] in REQUIRED}
         == REQUIRED, 'actual storage/shared header bindings')
    unresolved = profile['unresolved']
    need(len(unresolved) == 5 and {r['request'] for r in unresolved} == UNRESOLVED,
         'unresolved generated/request census')
    need(all(r['status'] == 'UNRESOLVED_GENERATED_OR_REQUEST_INPUT' for r in unresolved),
         'generated source misclassified')
    need(hashlib.sha256(json.dumps(profile, sort_keys=True, separators=(',', ':')).encode())
         .hexdigest() == PROFILE_SHA, 'reviewed original-request/public-source mapping changed')


def git(checkout, *args):
    return subprocess.check_output(['git', '-C', str(checkout), *args])


def verify(profile, checkout):
    validate(profile)
    checkout = checkout.resolve()
    need(git(checkout, 'rev-parse', 'HEAD').decode().strip() == PIN, 'wrong framework commit')
    need(git(checkout, 'rev-parse', 'HEAD^{tree}').decode().strip() == TREE, 'wrong framework tree')
    need(not git(checkout, 'status', '--porcelain', '--untracked-files=no'), 'dirty framework')
    payloads = {}
    for row in profile['sources']:
        path = checkout / row['path']
        need(path.resolve().is_relative_to(checkout) and not path.is_symlink(), 'source escape')
        data = path.read_bytes()
        need(identity(data) == (row['git_blob'], row['sha256'], row['bytes']), 'changed source bytes')
        need(data == git(checkout, 'show', PIN + ':' + row['path']), 'public Git byte identity')
        payloads[row['request'][1:]] = data
    notice = (checkout / 'LICENSE').read_bytes()
    need(identity(notice)[0] == NOTICE and notice == git(checkout, 'show', PIN + ':LICENSE'),
         'changed component notice')
    return payloads, notice


def stage(profile, payloads, notice, destination):
    validate(profile)
    need(not destination.exists() and not destination.is_symlink(), 'owned new destination required')
    expected = {r['request'][1:]: r for r in profile['sources']}
    need(set(payloads) == set(expected), 'staging source census')
    need(identity(notice)[0] == NOTICE, 'staging notice binding')
    # Validate every target BEFORE making any directory or copying any source.
    for name in payloads:
        safe(name)
        need(name.startswith('pxtapp/') and '/' + name not in UNRESOLVED, 'unadmitted staged input')
        row = expected[name]
        need(identity(payloads[name]) == (row['git_blob'], row['sha256'], row['bytes']),
             'source mutated after verification')
    destination.mkdir(parents=True)
    for name, data in payloads.items():
        target = destination / name
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(data)
    (destination / 'LICENSE-pxt-common-packages').write_bytes(notice)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--verify-checkout', type=Path)
    parser.add_argument('--stage', type=Path)
    parser.add_argument('--require-consumer-build', action='store_true')
    args = parser.parse_args()
    profile = load(ROOT / 'profiles/pxt-framework-sources.json')
    validate(profile)
    if args.require_consumer_build:
        parser.error('source staging only: actual PXT consumer compilation remains unqualified')
    if args.stage and not args.verify_checkout:
        parser.error('staging requires immutable checkout verification')
    if args.verify_checkout:
        payloads, notice = verify(profile, args.verify_checkout)
        if args.stage:
            stage(profile, payloads, notice, args.stage)
    print(json.dumps(dict(status=profile['status'], public_framework_sources=64,
                         unresolved_generated_or_request_inputs=5, consumer_compiled=False,
                         image_admitted=False, licence_clearance=False), sort_keys=True))
