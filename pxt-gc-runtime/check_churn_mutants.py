#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Hosted allocation-churn source mutants; no baseline replay."""
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parents[1]
MUTANTS = {
    'omit-native-copy': ('native-aligned-heap.cpp', '6-native-aligned-heap',
        b'memcpy(replacement, memory, previous < requested ? previous : requested);',
        b'memcpy(replacement, memory, 0);',
        'native realloc must preserve every old payload byte'),
    'omit-native-overflow-refusal': ('native-aligned-heap.cpp', '6-native-aligned-heap',
        b'if (!requested || requested > ~size_t(0) - overhead) return nullptr;',
        b'if (!requested) return nullptr;',
        'native arithmetic refusal must return null'),
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
    prefix = r'source=[0-9a-f]{40} elf_sha256=[0-9a-f]{64} step=[0-9]+'
    if expected == 'native arithmetic refusal must return null':
        observations = re.findall(r'^PXT_GC_CHURN_RETURN ' + prefix +
            r' sequence=2 op=7 slot=8 bytes=4294967295 old=0 pointer=([0-9]+) guest_inputs_unchanged=true$', text, re.M)
        wrong = len(observations) == 1 and int(observations[0]) != 0
    elif expected == 'native realloc must preserve every old payload byte':
        observations = re.findall(r'^PXT_GC_CHURN_COPY ' + prefix +
            r' sequence=21 bytes=1 old=[0-9]+ pointer=[0-9]+ expected_sha256=([0-9a-f]{64}) actual_sha256=([0-9a-f]{64}) guest_inputs_unchanged=true$', text, re.M)
        wrong = len(observations) == 1 and observations[0][0] != observations[0][1]
    else:
        wrong = False
    return (returncode == 101 and named and wrong and
            'test result: FAILED. 0 passed; 1 failed; 0 ignored;' in text and
            'PXT_GC_CHURN_CALLER ' not in text and
            re.search(r'(?m)^PXT_GC_CHURN_BOUND(?:\s|$)', text) is None and
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
    need(report['source'] == source and report['status'] == 'GC_ALLOCATION_CHURN_LINK_ONLY' and
         report['case'] == 'allocation-churn' and
         report['elf_sha256'] == sha((baseline / 'gc-control.elf').read_bytes()), 'baseline image/source')
    # This campaign runs ONLY after both baseline guests completed successfully.
    for profile in ['default', 'event-scheduler']:
        log = (baseline / f'guest-{profile}.log').read_text()
        need(log.count('PXT_GC_CHURN_CALLER ') == 1 and
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
        selected_source = 'pxt-native-allocation/' + filename
        original = (ROOT / selected_source).read_bytes()
        need(report['dependencies'][selected_source] == dict(origin='codal-samd', sha256=sha(original)),
             'actual own source dependency binding before mutation')
        changed = mutate(kind, original)
        (folder / filename).write_bytes(changed)
        argv = list(by_label[object_label])
        old_object = f'gc-link-evidence/{object_label}.o'
        need(argv[0] == 'arm-none-eabi-g++' and argv[-4:] ==
             ['-c', selected_source, '-o', old_object], 'exact mutant compile target')
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
        candidate['mutation'] = dict(kind=kind, source=selected_source, original_sha256=sha(original),
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
            argv += ['--test', 'bw_pxt_gc_churn_consumer', '--', '--ignored', '--nocapture']
            env = dict(os.environ, BW_GC_ENGINE_SOURCE=engine_source,
                       BW_GC_RUNTIME_SOURCE=source, BW_GC_CHURN_DIR=str(folder))
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
    need(len(results) == 4 and all(row['detected'] for row in results),
         'all four actual churn source mutants must fail at their named checks, not bounds/setup')
    need(report['elf_sha256'] == sha((baseline / 'gc-control.elf').read_bytes()) and
         not subprocess.check_output(['git', 'status', '--porcelain', '--untracked-files=no'], cwd=engine),
         'baseline image and tracked engine source must remain unchanged')
    print(json.dumps(dict(status='PASS_FOUR_ACTUAL_CHURN_NEGATIVE_CONTROLS_ONLY',
                          source=source, native_arcade=False, performance_measured=False)))


if __name__ == '__main__':
    main()
