#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Hosted-only actual-source negative controls; no baseline replay."""
import hashlib
import json
import os
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[1]
MUTANTS = {
    'old-public-copy': ('gc-storage.cpp', '9-gc-storage',
        b'memcpy(mem, ptr, min((blockSize - 1) * sizeof(void *), size));',
        b'memcpy(mem, ptr, min(blockSize * sizeof(void *), size));',
        'public realloc must not copy its header word'),
    'omit-current-fiber-roots': ('pxt-fiber-roots.cpp', '11-pxt-fiber-roots',
        b'        auto fib = fibers[i];\n',
        b'        auto fib = fibers[i];\n        if (fib == currentFiber) continue;\n',
        'guest panic status=0xe0000a0c stage=2 checks=53'),
    'omit-suspended-fiber-roots': ('pxt-fiber-roots.cpp', '11-pxt-fiber-roots',
        b'        auto fib = fibers[i];\n',
        b'        auto fib = fibers[i];\n        if (fib != currentFiber) continue;\n',
        'guest panic status=0xe0000a02 stage=3 checks=58'),
}


def need(value, reason):
    if not value:
        raise ValueError(reason)


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def mutate(kind, raw):
    _, _, before, after, _ = MUTANTS[kind]
    need(raw.count(before) == 1, 'unique exact original mutation site')
    return raw.replace(before, after, 1)


def detects(raw, returncode, expected):
    text = raw.decode()
    named = expected in text.splitlines() if expected.startswith('guest panic status=') else expected in text
    return (returncode == 101 and named and
            'test result: FAILED. 0 passed; 1 failed; 0 ignored;' in text and
            'PXT_GC_CALLER ' not in text and 'PXT_GC_BOUND_DIAGNOSTIC' not in text and
            'did not reach terminal within' not in text and 'error: could not compile' not in text)


def main():
    need(os.environ.get('GITHUB_ACTIONS') == 'true' and
         os.environ.get('RUNNER_ENVIRONMENT') == 'github-hosted', 'hosted CI only')
    baseline = ROOT / 'gc-link-evidence'
    engine = ROOT / 'owned-gc-engine'
    out = ROOT / 'gc-mutant-evidence'
    need(not out.exists(), 'new isolated mutant output required')
    source = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT).decode().strip()
    engine_source = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=engine).decode().strip()
    need(engine_source == '73f8475f8e406b1ff4475bc9886e22479881d67a', 'exact engine')
    report = json.loads((baseline / 'build-report.json').read_bytes())
    need(report['source'] == source and report['status'] == 'GC_SERVICES_LINK_ONLY' and
         report['elf_sha256'] == sha((baseline / 'gc-control.elf').read_bytes()), 'baseline image/source')
    # This campaign runs ONLY after both baseline guests completed successfully.
    for profile in ['default', 'event-scheduler']:
        log = (baseline / f'guest-{profile}.log').read_text()
        need(log.count('PXT_GC_CALLER ') == 1 and
             f'source={source} elf_sha256={report["elf_sha256"]}' in log and
             'test result: ok. 1 passed; 0 failed; 0 ignored;' in log and
             'panicked at' not in log, 'actual successful baseline prerequisite')
    commands = json.loads((baseline / 'commands.json').read_bytes())
    by_label = {row['label']: row['argv'] for row in commands}
    need(len(by_label) == len(commands) and all(row['returncode'] == 0 for row in commands),
         'successful unique original command census')
    out.mkdir()
    results = []

    def command(argv, folder, label):
        result = subprocess.run(argv, cwd=ROOT, capture_output=True, timeout=120)
        (folder / (label + '.stdout')).write_bytes(result.stdout)
        (folder / (label + '.stderr')).write_bytes(result.stderr)
        (folder / (label + '.json')).write_text(json.dumps(dict(argv=argv, returncode=result.returncode)) + '\n')
        need(result.returncode == 0, 'mutant build failed before guest: ' + label)

    for kind, (filename, object_label, before, after, expected) in MUTANTS.items():
        folder = out / kind
        folder.mkdir()
        relative = folder.relative_to(ROOT).as_posix()
        original = (baseline / filename).read_bytes()
        changed = mutate(kind, original)
        (folder / filename).write_bytes(changed)
        argv = list(by_label[object_label])
        old_object = f'gc-link-evidence/{object_label}.o'
        need(argv[0] == 'arm-none-eabi-g++' and argv[-4:] ==
             ['-c', 'gc-link-evidence/' + filename, '-o', old_object], 'exact mutant compile target')
        need(argv.count('-MF') == 1, 'exact dependency output option')
        argv[argv.index('-MF') + 1] = f'{relative}/{object_label}.d'
        argv[-3] = f'{relative}/{filename}'
        argv[-1] = f'{relative}/{object_label}.o'
        command(argv, folder, 'compile')
        link = list(by_label['link'])
        need(link[0] == 'arm-none-eabi-g++' and link.count(old_object) == 1 and
             link[-2:] == ['-o', 'gc-link-evidence/gc-control.elf'], 'exact baseline link output')
        link[link.index(old_object)] = f'{relative}/{object_label}.o'
        old_map = '-Wl,--gc-sections,--no-undefined,-Map=gc-link-evidence/gc-control.map'
        need(link.count(old_map) == 1, 'unchanged link policy')
        link[link.index(old_map)] = '-Wl,--gc-sections,--no-undefined,-Map=' + relative + '/gc-control.map'
        link[-1] = relative + '/gc-control.elf'
        command(link, folder, 'link')
        command(['arm-none-eabi-objcopy', '-O', 'binary', relative + '/gc-control.elf',
                 relative + '/gc-control.bin'], folder, 'binary')
        candidate = dict(report)
        candidate['elf_sha256'] = sha((folder / 'gc-control.elf').read_bytes())
        candidate['binary_sha256'] = sha((folder / 'gc-control.bin').read_bytes())
        candidate['mutation'] = dict(kind=kind, source=filename, original_sha256=sha(original),
                                     mutated_sha256=sha(changed), before=before.decode(), after=after.decode(),
                                     baseline_elf_sha256=report['elf_sha256'], positive_admission=False)
        # The original dependency/staged receipts are retained as BASELINE facts,
        # not falsely rewritten as a qualified positive build of the mutation.
        candidate['mutation']['baseline_report_sha256'] = sha((baseline / 'build-report.json').read_bytes())
        (folder / 'build-report.json').write_text(json.dumps(candidate, sort_keys=True) + '\n')
        for profile in ['default', 'event-scheduler']:
            argv = ['cargo', 'test', '--release', '--locked', '-p', 'labwired-core']
            if profile == 'event-scheduler':
                argv += ['--features', 'event-scheduler']
            argv += ['--test', 'bw_pxt_gc_consumer', '--', '--ignored', '--nocapture']
            env = dict(os.environ, BW_GC_ENGINE_SOURCE=engine_source,
                       BW_GC_RUNTIME_SOURCE=source, BW_GC_FIXTURE_DIR=str(folder))
            try:
                result = subprocess.run(argv, cwd=engine, env=env, capture_output=True, timeout=120)
                stdout, stderr, returncode = result.stdout, result.stderr, result.returncode
            except subprocess.TimeoutExpired as error:
                stdout, stderr, returncode = error.stdout or b'', error.stderr or b'', None
            (folder / f'guest-{profile}.stdout').write_bytes(stdout)
            (folder / f'guest-{profile}.stderr').write_bytes(stderr)
            raw = stdout + stderr
            (folder / f'guest-{profile}.log').write_bytes(raw)
            observed = dict(kind=kind, profile=profile, argv=argv, returncode=returncode,
                            expected=expected, detected=detects(raw, returncode, expected),
                            log_sha256=sha(raw), elf_sha256=candidate['elf_sha256'])
            results.append(observed)
            (out / 'results.json').write_text(json.dumps(results, indent=2) + '\n')
    need(len(results) == 6 and all(row['detected'] for row in results),
         'all six actual source mutants must fail at their named checks, not bounds/setup')
    need(report['elf_sha256'] == sha((baseline / 'gc-control.elf').read_bytes()) and
         not subprocess.check_output(['git', 'status', '--porcelain', '--untracked-files=no'], cwd=engine),
         'baseline image and tracked engine source must remain unchanged')
    print(json.dumps(dict(status='PASS_SIX_ACTUAL_SOURCE_NEGATIVE_CONTROLS_ONLY',
                          source=source, native_arcade=False, performance_measured=False)))


if __name__ == '__main__':
    main()
