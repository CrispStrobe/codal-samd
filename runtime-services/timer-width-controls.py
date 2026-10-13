#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Source-bound integer controls; not clock, compiler or guest qualification."""
import argparse
import hashlib
import json
from pathlib import Path
import re

BODY = """
void Timer::sync() {
    target_disable_irq();
    uint32_t val = timer.captureCounter();
    uint32_t elapsed = 0;
    elapsed = val - sigma;
    switch (timer.getBitMode()) {
        case BitMode8: elapsed &= 0xff; break;
        case BitMode16: elapsed &= 0xffff; break;
        case BitMode24: elapsed &= 0xffffff; break;
        case BitMode32: break;
        default: target_panic(DEVICE_INVALID_PARAMETER);
    }
    sigma = val;
    currentTimeUs += elapsed;
    currentTime += elapsed / 1000;
    const uint32_t remainder = delta + elapsed % 1000;
    currentTime += remainder / 1000;
    delta = remainder % 1000;
    target_enable_irq();
}
"""


def normalized(source):
    return re.sub(r"\s+", "", re.sub(r"//[^\n]*", "", source))


def bind(header, source):
    if not re.search(r"class Timer\s*\{\s*uint32_t sigma;\s*uint16_t delta;\s*LowLevelTimer& timer;", header):
        raise ValueError("wrong saved-counter/remainder field types")
    begin = source.index("void Timer::sync()")
    end = source.index("void Timer::recomputeNextTimerEvent()", begin)
    if normalized(source[begin:end]) != normalized(BODY):
        raise ValueError("unreviewed Timer sync arithmetic")


def candidate(before, after, bits, remainder):
    elapsed = ((after - before) & 0xffffffff) & ((1 << bits) - 1)
    tail = remainder + elapsed % 1000
    return elapsed, elapsed // 1000 + tail // 1000, tail % 1000


def controls():
    cases = 0
    for bits in (8, 16, 24, 32):
        mask = (1 << bits) - 1
        for before in (0, 1, mask // 2, mask - 1, mask):
            for interval in sorted({0, 1, min(999, mask), min(1000, mask),
                                    min(65535, mask), min(65536, mask),
                                    min(80002, mask), mask}):
                after = (before + interval) & mask
                for remainder in (0, 1, 499, 999):
                    elapsed, ms, tail = candidate(before, after, bits, remainder)
                    expected_ms, expected_tail = divmod(interval + remainder, 1000)
                    assert (elapsed, ms, tail) == (interval, expected_ms, expected_tail)
                    assert 0 <= tail < 1000
                    cases += 1
    # Explicitly distinguish the old elapsed cast and the separate remainder
    # overflow bug that survives widening only sigma/elapsed.
    assert candidate(421, 80423, 32, 414) == (80002, 80, 416)
    assert (80002 & 0xffff) != 80002
    assert ((80002 + 414) & 0xffff) // 1000 != (80002 + 414) // 1000
    assert candidate(0, 0xffffffff, 32, 999) == (0xffffffff, 4294968, 294)
    # Source counterfeits must refuse, not pass the mathematical model alone.
    header = "class Timer { uint32_t sigma; uint16_t delta; LowLevelTimer& timer;"
    source = BODY + "\nvoid Timer::recomputeNextTimerEvent()"
    bind(header, source)
    for changed_header, changed_source in (
        (header.replace("uint32_t sigma", "uint16_t sigma"), source),
        (header, source.replace("elapsed = val - sigma", "elapsed = (uint16_t)(val - sigma)")),
        (header, source.replace("delta = remainder % 1000", "delta += elapsed")),
        (header, source.replace("elapsed &= 0xffff", "elapsed &= 0xffffff")),
        (header, source.replace("target_disable_irq();", "")),
        (header, source.replace("target_panic(DEVICE_INVALID_PARAMETER)", "break")),
    ):
        try:
            bind(changed_header, changed_source)
        except ValueError:
            pass
        else:
            raise AssertionError("counterfeit source admitted")
    return {"status": "PASS", "integer_cases": cases, "source_refusals": 6,
            "boundary": "Source-bound integer controls only; not hardware/clock/compiler/guest qualification"}


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("core", nargs="?")
    args = parser.parse_args()
    result = controls()
    if args.core:
        root = Path(args.core)
        header = (root / "inc/driver-models/Timer.h").read_bytes()
        source = (root / "source/driver-models/Timer.cpp").read_bytes()
        bind(header.decode(), source.decode())
        result["header_sha256"] = hashlib.sha256(header).hexdigest()
        result["source_sha256"] = hashlib.sha256(source).hexdigest()
    print(json.dumps(result, sort_keys=True))
