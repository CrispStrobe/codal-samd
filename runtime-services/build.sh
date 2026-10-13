#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
set -euo pipefail
test "$#" -eq 3
core=$(realpath "$1")
out=$(realpath -m "$2")
root=$(git rev-parse --show-toplevel)
mode=$3
test "$mode" = 0 || test "$mode" = 1
pin=2ed19d64ec3359143409a4b0a9758b4aeef3ee7d
test "$(git -C "$core" rev-parse HEAD)" = "$pin"
test -z "$(git -C "$core" status --porcelain --untracked-files=no)"
test -z "$(git -C "$root" status --porcelain --untracked-files=no)"
test ! -e "$out"
mkdir -p "$out"
git -C "$core" apply --check "$root/runtime-services/fiber-guards.patch"
git -C "$core" apply "$root/runtime-services/fiber-guards.patch"
test "$(git -C "$core" diff --name-only)" = source/core/CodalFiber.cpp
git -C "$core" diff --binary > "$out/actual-core-patch.diff"
includes=(-I "$root/runtime-services" -I "$root/simulation/include"
          -I "$root/paired-irq/include" -isystem "$core/inc/core"
          -isystem "$core/inc/types" -isystem "$core/inc/driver-models"
          -isystem "$core/inc/drivers")
flags=(-mcpu=cortex-m4 -mthumb -std=c++11 -O0 -ffreestanding
       -ffunction-sections -fdata-sections -fno-builtin -fno-exceptions
       -fno-rtti -fno-unwind-tables -fno-asynchronous-unwind-tables
       -DBW_SIMULATION_CODAL_SERVICES=1 -DBW_RUNTIME_TEST_MODE="$mode")
if arm-none-eabi-g++ -mcpu=cortex-m4 -mthumb -std=c++11 \
    -c "$root/runtime-services/platform_includes.h" -o "$out/unselected.o" \
    2> "$out/unselected.stderr"; then
    exit 1
fi
grep -F 'Select the Cortex-M4 simulation-only real CODAL service profile explicitly' "$out/unselected.stderr"
objects=()
for source in source/driver-models/Timer.cpp source/core/CodalComponent.cpp \
              source/core/CodalHeapAllocator.cpp source/core/CodalFiber.cpp \
              source/core/CodalListener.cpp source/drivers/MessageBus.cpp \
              source/types/Event.cpp; do
    name=$(basename "$source" .cpp)
    arm-none-eabi-g++ "${flags[@]}" "${includes[@]}" -Wall \
        -MD -MF "$out/$name.d" -c "$core/$source" -o "$out/$name.o"
    objects+=("$out/$name.o")
done
for name in clock target probe; do
    arm-none-eabi-g++ "${flags[@]}" "${includes[@]}" -Wall \
        -MD -MF "$out/$name.d" -c "$root/runtime-services/$name.cpp" -o "$out/$name.o"
    objects+=("$out/$name.o")
done
arm-none-eabi-gcc -mcpu=cortex-m4 -mthumb \
    -c "$root/runtime-services/startup.S" -o "$out/startup.o"
arm-none-eabi-gcc -mcpu=cortex-m4 -mthumb \
    -c "$root/asm/CortexContextSwitch.s" -o "$out/context.o"
arm-none-eabi-g++ -mcpu=cortex-m4 -mthumb -nostdlib -nostartfiles \
    -Wl,--gc-sections,--no-undefined,-Map="$out/control.map" \
    -T "$root/runtime-services/control.ld" "${objects[@]}" \
    "$out/startup.o" "$out/context.o" -lgcc -o "$out/control.elf"
test -z "$(arm-none-eabi-nm -u "$out/control.elf")"
if arm-none-eabi-objdump -h "$out/control.elf" | grep -E '\.(init_array|ctors)'; then
    exit 1
fi
arm-none-eabi-nm -S "$out/control.elf" > "$out/symbols.txt"
arm-none-eabi-objdump -h "$out/control.elf" > "$out/sections.txt"
arm-none-eabi-size "$out/control.elf"
sha256sum "$out/control.elf" "$out/control.map"
arm-none-eabi-g++ --version > "$out/compiler.txt"
arm-none-eabi-g++ -print-libgcc-file-name > "$out/libgcc-path.txt"
python3 -B "$root/runtime-services/provenance.py" "$core" "$root" "$out" "$mode"
# Build/provenance evidence only. Both source-bound guest profiles must follow.
