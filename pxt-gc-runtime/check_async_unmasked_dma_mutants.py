#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Hosted copied-source mutants of finite unmasked async completion."""
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
from check_churn_mutants import need, sha

ROOT = Path(__file__).resolve().parents[1]
MUTANTS = {
    'wrong-native-byte': ('pxt-gc-runtime/probe-allocated-dma-async-unmasked.cpp', '10-probe-allocated-dma-async-unmasked',
        b'buffer[i] = pattern(i);', b'buffer[i] = pattern(i) ^ (i == 7 ? 1u : 0u);',
        'native live payload byte', 'ALLOCATED_DMA_ASYNC_UNMASKED_BAD_PAYLOAD'),
    'duplicate-original-callback': ('runtime-services/Sercom4TxSPI.cpp', '9-Sercom4TxSPI',
        b'if (callback) callback(argument);',
        b'if (callback) { callback(argument); callback(argument); }',
        'at most one original callback', 'ALLOCATED_DMA_ASYNC_UNMASKED_WRONG_CALLBACK'),
    'omit-native-free': ('pxt-gc-runtime/probe-allocated-dma-async-unmasked.cpp', '10-probe-allocated-dma-async-unmasked',
        b'    bw_native_free(pointer);', b'    (void)pointer; // Mutant omits real native free.',
        'actual two native frees before public release', 'ALLOCATED_DMA_ASYNC_UNMASKED_MISSING_FREE'),
    'omit-pending-collections': ('pxt-gc-runtime/probe-allocated-dma-async-unmasked.cpp', '10-probe-allocated-dma-async-unmasked',
        b'        gc(0);', b'        // Mutant omits the actual GC collection.',
        'actual collections while DMA pending', 'ALLOCATED_DMA_ASYNC_UNMASKED_MISSING_COLLECTIONS'),
    'omit-quiet-loop': ('pxt-gc-runtime/probe-allocated-dma-async-unmasked.cpp', '10-probe-allocated-dma-async-unmasked',
        b'for (unsigned i = 0; i < 8192; ++i)', b'for (unsigned i = 0; i < 0; ++i)',
        'actual unmasked post-release thread interval', 'ALLOCATED_DMA_ASYNC_UNMASKED_SHORT_QUIET'),
    'mask-quiet-phase': ('pxt-gc-runtime/probe-allocated-dma-async-unmasked.cpp', '10-probe-allocated-dma-async-unmasked',
        b'    gc_dma_stage = 5;', b'    Mask::restore(1);\n    gc_dma_stage = 5;',
        'actual admitted quiet phase', 'ALLOCATED_DMA_ASYNC_UNMASKED_BAD_QUIET'),
}


def mutate(kind, original):
    before, after = MUTANTS[kind][2:4]
    need(original.count(before) == 1, 'unique exact source mutation required')
    return original.replace(before, after, 1)


def detects(raw, code, kind, source, image):
    try:
        text = raw.decode()
    except UnicodeDecodeError:
        return False
    expected, marker = MUTANTS[kind][-2:]
    prefix = f'ALLOCATED_DMA_ASYNC_UNMASKED_IMAGE source={source} elf_sha256={image} case=allocated-dma-async-unmasked'
    markers = re.findall(r'^' + marker + r' [^\r\n]+$', text, re.M)
    actual = {'wrong-native-byte': r'pointer=([0-9]+) index=7 actual=15 expected=14',
              'duplicate-original-callback': r'step=[0-9]+ pc=0x[0-9a-f]+ returned=true active=79',
              'omit-native-free': r'step=[0-9]+ actual=0 expected=2',
              'omit-pending-collections': r'step=[0-9]+ actual=0 expected=2',
              'omit-quiet-loop': r'step=[0-9]+ actual=([0-9]+) expected_min=2048',
              'mask-quiet-phase': r'step=[0-9]+ active=0 primask=true reason=masked-thread'}[kind]
    witness = len(markers) == 1 and re.fullmatch(marker + ' ' + actual, markers[0]) is not None
    if witness and kind == 'wrong-native-byte':
        pointer = int(re.fullmatch(marker + ' ' + actual, markers[0]).group(1))
        witness = pointer % 8 == 0 and 0x20000008 <= pointer <= 0x2002d000
    if witness and kind == 'omit-quiet-loop':
        witness = int(re.fullmatch(marker + ' ' + actual, markers[0]).group(1)) < 2048
    return (code == 101 and text.count(prefix) == 1 and expected in text and witness and
            text.count('test result: FAILED. 0 passed; 1 failed; 0 ignored;') == 1 and
            'ALLOCATED_DMA_ASYNC_UNMASKED_RESULT ' not in text and
            'exceeded unchanged bounded budget' not in text and
            'architectural fault' not in text and 'error: could not compile' not in text)


def main():
    need(os.environ.get('GITHUB_ACTIONS') == 'true' and
         os.environ.get('RUNNER_ENVIRONMENT') == 'github-hosted', 'hosted CI only')
    baseline, engine, out = [ROOT / p for p in
                             ['gc-link-evidence', 'owned-gc-engine', 'gc-dma-async-unmasked-mutants']]
    need(not out.exists(), 'fresh mutant output required')
    source = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT).decode().strip()
    engine_source = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=engine).decode().strip()
    need(engine_source == '73f8475f8e406b1ff4475bc9886e22479881d67a', 'exact held engine')
    report = json.loads((baseline / 'build-report.json').read_bytes())
    need(report['source'] == source and report['case'] == 'allocated-dma-async-unmasked' and
         report['status'] == 'GC_ALLOCATED_DMA_ASYNC_UNMASKED_LINK_ONLY' and
         report['elf_sha256'] == sha((baseline / 'gc-control.elf').read_bytes()), 'positive source/image')
    for profile in ['default', 'event-scheduler']:
        log = (baseline / f'guest-{profile}.log').read_text()
        need(log.count('ALLOCATED_DMA_ASYNC_UNMASKED_RESULT ') == 1 and
             f'source={source} elf_sha256={report["elf_sha256"]}' in log and
             'test result: ok. 1 passed; 0 failed; 0 ignored;' in log and
             'panicked at' not in log, 'both actual finite completion positives required')
    commands = json.loads((baseline / 'commands.json').read_bytes())
    by_label = {row['label']: row['argv'] for row in commands}
    need(len(by_label) == len(commands) and all(c['returncode'] == 0 for c in commands), 'positive command census')
    out.mkdir()
    shutil.copytree(baseline / 'notices', out / 'notices')
    results = []

    def command(argv, folder, label):
        result = subprocess.run(argv, cwd=ROOT, capture_output=True, timeout=120)
        (folder / (label + '.stdout')).write_bytes(result.stdout)
        (folder / (label + '.stderr')).write_bytes(result.stderr)
        (folder / (label + '.json')).write_text(json.dumps(dict(argv=argv, returncode=result.returncode)) + '\n')
        need(result.returncode == 0, 'mutant build/setup must succeed: ' + label)

    for kind, (selected, label, before, after, expected, marker) in MUTANTS.items():
        original = (ROOT / selected).read_bytes()
        need(report['dependencies'][selected] == dict(origin='codal-samd', sha256=sha(original)),
             'exact original own source dependency')
        folder = out / kind
        folder.mkdir()
        relative = folder.relative_to(ROOT).as_posix()
        changed = mutate(kind, original)
        (folder / Path(selected).name).write_bytes(changed)
        argv = list(by_label[label])
        old_object = f'gc-link-evidence/{label}.o'
        need(argv[0] == 'arm-none-eabi-g++' and argv[-4:] == ['-c', selected, '-o', old_object] and
             argv.count('-MF') == 1, 'exact positive compile target')
        argv[argv.index('-MF') + 1] = relative + '/' + label + '.d'
        argv[-3], argv[-1] = relative + '/' + Path(selected).name, relative + '/' + label + '.o'
        command(argv, folder, 'compile')
        link = list(by_label['link'])
        old_map = '-Wl,--gc-sections,--no-undefined,-Map=gc-link-evidence/gc-control.map'
        need(link.count(old_object) == link.count(old_map) == 1 and
             link[-2:] == ['-o', 'gc-link-evidence/gc-control.elf'], 'exact positive link target')
        link[link.index(old_object)] = relative + '/' + label + '.o'
        link[link.index(old_map)] = '-Wl,--gc-sections,--no-undefined,-Map=' + relative + '/gc-control.map'
        link[-1] = relative + '/gc-control.elf'
        command(link, folder, 'link')
        command(['arm-none-eabi-objcopy', '-O', 'binary', relative + '/gc-control.elf', relative + '/gc-control.bin'], folder, 'binary')
        candidate = dict(report, elf_sha256=sha((folder / 'gc-control.elf').read_bytes()),
                         binary_sha256=sha((folder / 'gc-control.bin').read_bytes()))
        candidate['mutation'] = dict(kind=kind, source=selected, original_sha256=sha(original),
            mutated_sha256=sha(changed), before=before.decode(), after=after.decode(),
            baseline_report_sha256=sha((baseline / 'build-report.json').read_bytes()),
            baseline_elf_sha256=report['elf_sha256'], positive_admission=False)
        (folder / 'build-report.json').write_text(json.dumps(candidate, sort_keys=True) + '\n')
        for profile in ['default', 'event-scheduler']:
            argv = ['cargo', 'test', '--release', '--locked', '-p', 'labwired-core']
            if profile == 'event-scheduler':
                argv += ['--features', 'event-scheduler']
            argv += ['--test', 'bw_pxt_allocated_dma_async_unmasked', '--', '--ignored', '--nocapture']
            env = dict(os.environ, BW_GC_ENGINE_SOURCE=engine_source, BW_GC_RUNTIME_SOURCE=source,
                       BW_GC_FIXTURE_DIR=str(folder))
            try:
                result = subprocess.run(argv, cwd=engine, env=env, capture_output=True, timeout=120)
                stdout, stderr, code = result.stdout, result.stderr, result.returncode
            except subprocess.TimeoutExpired as error:
                stdout, stderr, code = error.stdout or b'', error.stderr or b'', None
            for suffix, raw in [('stdout', stdout), ('stderr', stderr), ('log', stdout + stderr)]:
                (folder / f'guest-{profile}.{suffix}').write_bytes(raw)
            results.append(dict(kind=kind, profile=profile, argv=argv, returncode=code,
                expected=expected, marker=marker, detected=detects(stdout + stderr, code, kind, source, candidate['elf_sha256']),
                log_sha256=sha(stdout + stderr), elf_sha256=candidate['elf_sha256']))
            (out / 'results.json').write_text(json.dumps(results, indent=2) + '\n')
    need(len(results) == 12 and all(row['detected'] for row in results), 'twelve actual named wrong-result controls required')
    need(report['elf_sha256'] == sha((baseline / 'gc-control.elf').read_bytes()) and
         not subprocess.check_output(['git', 'status', '--porcelain', '--untracked-files=no'], cwd=engine),
         'original positive image and engine unchanged')
    print(json.dumps(dict(status='PASS_TWELVE_ACTUAL_UNMASKED_ASYNC_SOURCE_MUTANTS_ONLY',
        source=source, full_lifetime=False, native_arcade=False, performance_measured=False)))


if __name__ == '__main__':
    main()
