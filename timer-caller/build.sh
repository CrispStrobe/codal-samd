#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
set -euo pipefail
test "$#" -eq 2
core=$(realpath "$1")
out=$(realpath -m "$2")
root=$(git rev-parse --show-toplevel)
pin=2ed19d64ec3359143409a4b0a9758b4aeef3ee7d
test "$(git -C "$core" rev-parse HEAD)" = "$pin"
test -z "$(git -C "$core" status --porcelain --untracked-files=no)"
test -z "$(git -C "$root" status --porcelain --untracked-files=no)"
mkdir -p "$out"
python3 -B "$root/timer-caller/test_provenance.py"
python3 -B "$root/timer-caller/provenance.py" before "$core" "$root" "$out" > "$out/source-before.json"
includes=(-I "$root/timer-caller" -isystem "$core/inc/core"
          -isystem "$core/inc/driver-models" -isystem "$core/inc/types"
          -I "$root/paired-irq/include" -I "$root/simulation/include")
flags=(-mcpu=cortex-m4 -mthumb -std=c++11 -O0 -ffreestanding
       -ffunction-sections -fdata-sections -fno-builtin -fno-exceptions
       -fno-rtti -fno-unwind-tables -fno-asynchronous-unwind-tables
       -DBW_SIMULATION_STRICT_PAIRED_IRQ=1 -DBW_SIMULATION_TIMER_CALLER=1)

# Require the fixture's explicitly selected platform; no vendor include fallback.
if arm-none-eabi-g++ -mcpu=cortex-m4 -mthumb -std=c++11 \
    -c "$root/timer-caller/platform_includes.h" -o "$out/unselected.o" \
    2> "$out/unselected.stderr"; then
    exit 1
fi
grep -F 'Select the ARM simulation-only Timer caller fixture explicitly' "$out/unselected.stderr"
for pair in "timer:$core/source/driver-models/Timer.cpp" \
            "profile:$root/paired-irq/profile.cpp" \
            "probe:$root/timer-caller/probe.cpp"; do
    name=${pair%%:*}
    source=${pair#*:}
    arm-none-eabi-g++ "${flags[@]}" "${includes[@]}" -Wall \
        -MD -MF "$out/$name.d" -c "$source" -o "$out/$name.o"
done
arm-none-eabi-gcc -mcpu=cortex-m4 -mthumb \
    -c "$root/simulation/arm/irq-startup.S" -o "$out/startup.o"
arm-none-eabi-g++ -mcpu=cortex-m4 -mthumb -nostdlib -nostartfiles \
    -Wl,--gc-sections,--no-undefined,-Map="$out/control.map" \
    -T "$root/simulation/arm/control.ld" "$out/startup.o" "$out/timer.o" \
    "$out/profile.o" "$out/probe.o" -lgcc -o "$out/control.elf"
test -z "$(arm-none-eabi-nm -u "$out/control.elf")"
if arm-none-eabi-objdump -h "$out/control.elf" | grep -E '\.(init_array|ctors)'; then
    exit 1
fi
arm-none-eabi-nm -S "$out/control.elf" > "$out/symbols.txt"
arm-none-eabi-size "$out/control.elf"
sha256sum "$out/control.elf" "$out/control.map"
python3 -B "$root/timer-caller/provenance.py" after "$core" "$root" "$out" > "$out/build-provenance.json"
# This is only compile/link evidence. The caller requires new source-bound guests.
