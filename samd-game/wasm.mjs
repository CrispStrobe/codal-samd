// SPDX-License-Identifier: MIT
// Actual same-source WASM game; no guest replacement or mocked peripherals.
import fs from 'node:fs';
import path from 'node:path';
import {createRequire} from 'node:module';
import {createHash} from 'node:crypto';
import {performance} from 'node:perf_hooks';
const [packageDir, engineDir, gluePath, engine] = process.argv.slice(2);
const require = createRequire(import.meta.url);
const mod = require(path.resolve(gluePath));
const sha = b => createHash('sha256').update(b).digest('hex');
const read = n => fs.readFileSync(path.join(packageDir, n));
const symbols = JSON.parse(read('symbols.json'));
const system = fs.readFileSync(path.join(engineDir, 'configs/systems/samd51-no-asf-simulation.yaml'), 'utf8');
const chip = fs.readFileSync(path.join(engineDir, 'configs/chips/samd51-no-asf-simulation.yaml'), 'utf8');
const need = (ok, reason) => { if (!ok) throw new Error(reason); };
export function rectangle (bytes, colour, size) {
    const points = [];
    for (let i = 0; i < bytes.length; i += 3) {
        if (colour.every((v, n) => bytes[i + n] === v)) points.push([(i / 3) % 128, Math.floor(i / 3 / 128)]);
    }
    if (points.length !== size * size) return null;
    const x = Math.min(...points.map(p => p[0]));
    const y = Math.min(...points.map(p => p[1]));
    if (points.some(p => p[0] >= x + size || p[1] >= y + size)) return null;
    return [159 - y - (size - 1), x];
}
function frame (sim) {
    for (const entering of [true, false]) {
        const address = entering ? symbols.updateScreen : sim.get_register(14) & ~1;
        const stack = entering ? null : sim.get_register(13);
        sim.set_breakpoints([address, symbols.target_panic]);
        let reached = false;
        for (let i = 0; i < 1200; i++) {
            sim.step_batch(500_000);
            const pc = sim.get_pc();
            need(pc !== symbols.target_panic, `actual target_panic(${sim.get_register(0)})`);
            if (pc === address) { reached = true; break; }
        }
        need(reached, 'render boundary budget');
        if (stack !== null) need(stack === sim.get_register(13), 'same render call returned');
    }
    const artifact = sim.get_display('controller_memory', true);
    need(artifact && artifact.width === 128 && artifact.height === 160, 'actual controller artifact geometry');
    need(artifact.meta.unknown_pixels === 0 && artifact.bytes?.length === 128 * 160 * 3, 'complete authored colour view');
    return Buffer.from(artifact.bytes);
}
function snapshot (sim, colour, out, index) {
    for (let i = 0; i < 120; i++) {
        const bytes = frame(sim);
        const player = rectangle(bytes, colour, 16);
        const ball = rectangle(bytes, [0, 48, 170], 8);
        let targets = 0;
        for (let p = 0; p < bytes.length; p += 3) {
            if (bytes[p] === 255 && bytes[p + 1] === 138 && bytes[p + 2] === 48) targets++;
        }
        if (!player || !ball || targets !== 512) continue;
        fs.writeFileSync(path.join(out, `wasm-${index}.rgb`), bytes);
        return {sha256: sha(bytes), player, ball};
    }
    throw new Error('game picture budget');
}
const samples = [];
for (let sample = 0; sample < 3; sample++) {
    const out = path.join(packageDir, `wasm-sample-${sample}`);
    fs.mkdirSync(out);
    const began = performance.now();
    const sim = mod.WasmSimulator.new_from_config(system, chip, read('game.elf'), {
        LABWIRED_SAMD51_SIM_SW0: new Uint8Array(read('sw0.bin')),
        LABWIRED_SAMD51_SIM_SERIAL: new Uint8Array(read('serial.bin'))
    });
    sim.set_jit_enabled(false);
    const pictures = [];
    const times = [];
    const take = colour => {
        const p = snapshot(sim, colour, out, pictures.length);
        pictures.push(p); times.push((performance.now() - began) / 1000);
        return p;
    };
    for (let i = 0; i < 6; i++) take([255, 32, 32]);
    need(JSON.stringify(pictures[0].ball) !== JSON.stringify(pictures[5].ball), 'actual moving game');
    const initial = pictures[5].player[0];
    sim.set_input('right', 1); sim.set_input('a', 1);
    const pressedAt = performance.now();
    let moved = false;
    for (let i = 0; i < 120; i++) {
        if (take([138, 255, 255]).player[0] >= initial + 8) { moved = true; break; }
    }
    need(moved, 'actual right-button displacement');
    const pressSeconds = (performance.now() - pressedAt) / 1000;
    sim.set_input('right', 0); sim.set_input('a', 0);
    const releasedAt = performance.now();
    for (let i = 0; i < 4; i++) take([255, 32, 32]);
    const releaseSeconds = (performance.now() - releasedAt) / 1000;
    need(JSON.stringify(pictures.at(-1).player) === JSON.stringify(pictures.at(-2).player), 'released movement stopped');
    const result = {sample, engine, image_sha256: sha(read('game.bin')), startup_seconds: times[0],
        first_six_render_seconds: times[5] - times[0], render_intervals: 5,
        press_seconds: pressSeconds, release_seconds: releaseSeconds, pictures,
        wall_seconds: (performance.now() - began) / 1000, process_memory_bytes: process.memoryUsage()};
    fs.writeFileSync(path.join(out, 'sample.json'), JSON.stringify(result, null, 2));
    samples.push(result);
    sim.free();
}
fs.writeFileSync(path.join(packageDir, 'wasm-results.json'), JSON.stringify(samples, null, 2));
console.log(`SAMD_GAME_WASM_PASS engine=${engine} image_sha256=${sha(read('game.bin'))} samples=3 physical_capture=false`);
