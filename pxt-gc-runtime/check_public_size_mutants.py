#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Hosted isolated selected-size diagnostics; never alter positive images."""
import json
import os
from pathlib import Path
import re
import subprocess
from check_churn_mutants import need, sha

ROOT = Path(__file__).resolve().parents[1]
MUTANTS = {
    'omit-size-permanent-ownership': ('gc-link-evidence/gc-storage.cpp', '9-gc-storage',
        b'r[-1] |= PERMA_MASK;', b'r[-1] |= 0;',
        'public size return must retain selected array ownership and capacity'),
    'wrong-size-cap-panic': ('gc-link-evidence/gc-storage.cpp', '9-gc-storage',
        b'if (numbytes > GC_MAX_ALLOC_SIZE)\n        soft_panic(PANIC_GC_TOO_BIG_ALLOCATION);',
        b'if (numbytes > GC_MAX_ALLOC_SIZE)\n        soft_panic(PANIC_GC_OOM);',
        'selected cap must retain actual panic 22'),
}


def mutate(kind, original):
    _, _, before, after, _ = MUTANTS[kind]
    need(original.count(before) == 1, 'unique exact original mutation site')
    return original.replace(before, after, 1)


def detects(raw, code, expected):
    try:
        text = raw.decode()
    except UnicodeDecodeError:
        return False
    prefix = r'source=[0-9a-f]{40} elf_sha256=[0-9a-f]{64} step=[0-9]+'
    if expected == MUTANTS['wrong-size-cap-panic'][-1]:
        entry = re.findall(r'^PUBLIC_SIZE_ENTRY ' + prefix +
            r' stage=6 index=3 request=131076 actual=0x[0-9a-f]+$', text, re.M)
        rows = re.findall(r'^PUBLIC_SIZE_PANIC ' + prefix +
            r' stage=6 reason=21$', text, re.M)
    elif expected == MUTANTS['omit-size-permanent-ownership'][-1]:
        entry = re.findall(r'^PUBLIC_SIZE_ENTRY ' + prefix +
            r' stage=2 index=0 request=32 actual=0x[0-9a-f]+$', text, re.M)
        rows = re.findall(r'^PUBLIC_SIZE_HEADER ' + prefix +
            r' stage=2 index=1 pointer=([0-9]+) header=0x40000024 expected=0x60000024$', text, re.M)
        if len(rows) == 1 and int(rows[0]) == 0:
            return False
    else:
        return False
    return (code == 101 and expected in text and len(entry) == len(rows) == 1 and
        'test result: FAILED. 0 passed; 1 failed; 0 ignored;' in text and
        'PUBLIC_SIZE_DIAGNOSTIC ' not in text and
        re.search(r'(?m)^PUBLIC_SIZE_BOUND(?:\s|$)', text) is None and
        'did not reach its named terminal within' not in text and
        'error: could not compile' not in text)


def main():
    need(os.environ.get('GITHUB_ACTIONS') == 'true' and
         os.environ.get('RUNNER_ENVIRONMENT') == 'github-hosted', 'hosted CI only')
    baseline, engine, out = [ROOT / p for p in ['gc-link-evidence', 'owned-gc-engine', 'gc-public-size-mutants']]
    need(not out.exists(), 'fresh mutant output required')
    source = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT).decode().strip()
    engine_source = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=engine).decode().strip()
    need(engine_source == '73f8475f8e406b1ff4475bc9886e22479881d67a', 'exact engine')
    report = json.loads((baseline / 'build-report.json').read_bytes())
    need(report['source'] == source and report['status'] == 'GC_PUBLIC_SIZE_LINK_ONLY' and
         report['case'] in ['public-size-cap', 'public-size-conversion'] and report['elf_sha256'] == sha((baseline / 'gc-control.elf').read_bytes()), 'positive source/image')
    for profile in ['default', 'event-scheduler']:
        log = (baseline / f'guest-{profile}.log').read_text()
        need(log.count('PUBLIC_SIZE_DIAGNOSTIC ') == 1 and 'policy_changed=false' in log and
             f'source={source} elf_sha256={report["elf_sha256"]}' in log and
             'test result: ok. 1 passed; 0 failed; 0 ignored;' in log and 'panicked at' not in log,
             'both actual size-diagnostic positives required')
    commands = json.loads((baseline / 'commands.json').read_bytes())
    by_label = {row['label']: row['argv'] for row in commands}
    need(len(by_label) == len(commands) and all(row['returncode'] == 0 for row in commands), 'command census')
    out.mkdir()
    results = []

    def command(argv, folder, label):
        result = subprocess.run(argv, cwd=ROOT, capture_output=True, timeout=120)
        (folder / (label + '.stdout')).write_bytes(result.stdout)
        (folder / (label + '.stderr')).write_bytes(result.stderr)
        (folder / (label + '.json')).write_text(json.dumps(dict(argv=argv, returncode=result.returncode)) + '\n')
        need(result.returncode == 0, 'mutant build/setup must succeed: ' + label)

    kind = 'wrong-size-cap-panic' if report['case'] == 'public-size-cap' else 'omit-size-permanent-ownership'
    for kind in [kind]:
        selected, label, before, after, expected = MUTANTS[kind]
        original = (ROOT / selected).read_bytes()
        if selected.startswith('gc-link-evidence/'):
            need(report['staged'][Path(selected).name] == sha(original), 'staged source binding')
        else:
            need(report['dependencies'][selected] == dict(origin='codal-samd', sha256=sha(original)), 'own dependency binding')
        folder = out / kind
        folder.mkdir()
        relative = folder.relative_to(ROOT).as_posix()
        changed = mutate(kind, original)
        (folder / Path(selected).name).write_bytes(changed)
        argv = list(by_label[label])
        old_object = f'gc-link-evidence/{label}.o'
        need(argv[0] == 'arm-none-eabi-g++' and argv[-4:] == ['-c', selected, '-o', old_object] and argv.count('-MF') == 1, 'exact compile target')
        argv[argv.index('-MF') + 1] = relative + '/' + label + '.d'
        argv[-3], argv[-1] = relative + '/' + Path(selected).name, relative + '/' + label + '.o'
        command(argv, folder, 'compile')
        link = list(by_label['link'])
        old_map = '-Wl,--gc-sections,--no-undefined,-Map=gc-link-evidence/gc-control.map'
        need(link.count(old_object) == link.count(old_map) == 1 and link[-2:] == ['-o', 'gc-link-evidence/gc-control.elf'], 'exact link')
        link[link.index(old_object)] = relative + '/' + label + '.o'
        link[link.index(old_map)] = '-Wl,--gc-sections,--no-undefined,-Map=' + relative + '/gc-control.map'
        link[-1] = relative + '/gc-control.elf'
        command(link, folder, 'link')
        command(['arm-none-eabi-objcopy', '-O', 'binary', relative + '/gc-control.elf', relative + '/gc-control.bin'], folder, 'binary')
        candidate = dict(report, elf_sha256=sha((folder / 'gc-control.elf').read_bytes()), binary_sha256=sha((folder / 'gc-control.bin').read_bytes()))
        candidate['mutation'] = dict(kind=kind, source=selected, original_sha256=sha(original), mutated_sha256=sha(changed),
            before=before.decode(), after=after.decode(), baseline_report_sha256=sha((baseline / 'build-report.json').read_bytes()),
            baseline_elf_sha256=report['elf_sha256'], positive_admission=False)
        (folder / 'build-report.json').write_text(json.dumps(candidate, sort_keys=True) + '\n')
        for profile in ['default', 'event-scheduler']:
            argv = ['cargo', 'test', '--release', '--locked', '-p', 'labwired-core']
            if profile == 'event-scheduler': argv += ['--features', 'event-scheduler']
            argv += ['--test', 'bw_pxt_public_size_consumer', '--', '--ignored', '--nocapture']
            env = dict(os.environ, BW_GC_ENGINE_SOURCE=engine_source, BW_GC_RUNTIME_SOURCE=source, BW_GC_PUBLIC_SIZE_DIR=str(folder))
            try:
                result = subprocess.run(argv, cwd=engine, env=env, capture_output=True, timeout=120)
                stdout, stderr, code = result.stdout, result.stderr, result.returncode
            except subprocess.TimeoutExpired as error:
                stdout, stderr, code = error.stdout or b'', error.stderr or b'', None
            for suffix, raw in [('stdout', stdout), ('stderr', stderr), ('log', stdout + stderr)]:
                (folder / f'guest-{profile}.{suffix}').write_bytes(raw)
            results.append(dict(kind=kind, profile=profile, argv=argv, returncode=code, expected=expected,
                detected=detects(stdout + stderr, code, expected), log_sha256=sha(stdout + stderr), elf_sha256=candidate['elf_sha256']))
            (out / 'results.json').write_text(json.dumps(results, indent=2) + '\n')
    need(len(results) == 2 and all(r['detected'] for r in results), 'two actual wrong-result size controls required')
    need(report['elf_sha256'] == sha((baseline / 'gc-control.elf').read_bytes()) and
         not subprocess.check_output(['git', 'status', '--porcelain', '--untracked-files=no'], cwd=engine), 'original positive/engine unchanged')
    print(json.dumps(dict(status='PASS_TWO_ACTUAL_PUBLIC_SIZE_MUTANTS_ONLY', source=source,
        native_arcade=False, performance_measured=False)))


if __name__ == '__main__':
    main()
