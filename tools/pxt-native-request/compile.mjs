#!/usr/bin/env node
// SPDX-License-Identifier: MIT
// Copyright (c) 2026 CrispStrobe contributors
//
// Compile a MakeCode Arcade program natively onto an owned base image, offline,
// with the admitted Arcade 4.2.1 target bundle and PXT 13.2.1 worker. The base is
// supplied through extinfo.hexinfo for the exact request sha it was built from;
// no precompiled MakeCode base or cloud compiler is consulted.
//
// usage: compile.mjs TARGET_JSON PXTWORKER_JS VARIANT BASE_HEX MAIN_TS OUT_DIR
import crypto from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import util from 'node:util';
import vm from 'node:vm';
import {TARGET_SHA256, WORKER_SHA256} from './request.mjs';

const sha256 = bytes => crypto.createHash('sha256').update(bytes).digest('hex');

export async function compileNative (targetBytes, workerBytes, variant, baseHex, mainTs) {
    if (sha256(targetBytes) !== TARGET_SHA256) throw new Error('target.json is not the admitted payload');
    if (sha256(workerBytes) !== WORKER_SHA256) throw new Error('pxtworker.js is not the admitted payload');
    const bundle = JSON.parse(targetBytes.toString('utf8'));
    const quiet = () => {};
    const network = [];
    const sandbox = {
        setTimeout, clearTimeout, setInterval, clearInterval, setImmediate, clearImmediate,
        TextEncoder: util.TextEncoder, TextDecoder: util.TextDecoder, Buffer,
        console: {log: quiet, debug: quiet, info: quiet, warn: quiet, error: quiet},
        pxtTargetBundle: bundle,
        __network: network, __variant: variant, __base: baseHex.toString('utf8').split(/\r?\n/),
        __files: {
            'pxt.json': JSON.stringify({name: 'arcade-base', dependencies: {device: '*'}, files: ['main.ts'], binaryonly: true}),
            'main.ts': mainTs.toString('utf8')
        }
    };
    sandbox.global = sandbox;
    sandbox.self = sandbox;
    vm.createContext(sandbox);
    vm.runInContext(workerBytes.toString('utf8'), sandbox, {filename: 'pxtworker.js'});
    const result = JSON.parse(await vm.runInContext(`(async () => {
        pxt.setupSimpleCompile({
            cacheGet: () => Promise.resolve(null), cacheSet: () => Promise.resolve(),
            httpRequestAsync: o => { __network.push(o && o.url); return Promise.reject(new Error('offline')); },
            pkgOverrideAsync: () => Promise.resolve(null)
        });
        pxt.packagesConfigAsync = () => Promise.resolve({});
        pxt.setupWebConfig({cdnUrl: 'https://offline.invalid'});
        pxt.setHwVariant(__variant);
        pxt.setAppTargetVariant(null);
        const options = await pxt.simpleGetCompileOptionsAsync(__files, {native: true});
        const sha = options.extinfo.sha;
        options.extinfo.hexinfo = {hex: __base};
        const res = pxtc.service.performOperation('compile', {options});
        const diagnostics = (res.diagnostics || []).map(d => {
            let m = d.messageText;
            while (m && typeof m === 'object') m = m.messageText;
            return d.fileName + ':' + d.line + ': ' + m;
        });
        return JSON.stringify({sha, success: !!res.success, outfiles: res.outfiles || {}, diagnostics});
    })()`, sandbox));
    if (network.length) throw new Error('native compile attempted network access: ' + network.join(', '));
    return result;
}

if (import.meta.url === `file://${process.argv[1]}`) {
    const [targetPath, workerPath, variant, basePath, mainPath, out] = process.argv.slice(2);
    if (!out) throw new Error('usage: compile.mjs TARGET_JSON PXTWORKER_JS VARIANT BASE_HEX MAIN_TS OUT_DIR');
    const base = fs.readFileSync(basePath);
    const result = await compileNative(fs.readFileSync(targetPath), fs.readFileSync(workerPath), variant, base,
                                       fs.readFileSync(mainPath));
    fs.mkdirSync(out, {recursive: true});
    for (const [name, data] of Object.entries(result.outfiles)) {
        const bytes = /\.uf2$/.test(name) ? Buffer.from(data, 'base64') : Buffer.from(data, 'utf8');
        fs.writeFileSync(path.join(out, name), bytes);
    }
    fs.writeFileSync(path.join(out, 'compile-result.json'), JSON.stringify({
        request_sha256: result.sha, base_sha256: sha256(base), success: result.success,
        outfiles: Object.fromEntries(Object.keys(result.outfiles).map(n => [n, sha256(fs.readFileSync(path.join(out, n)))])),
        diagnostics: result.diagnostics
    }, null, 2) + '\n');
    console.log(`PXT_NATIVE_COMPILE request=${result.sha} success=${result.success} outfiles=${Object.keys(result.outfiles).join(',')}`);
    if (!result.success) {
        console.log(result.diagnostics.join('\n'));
        process.exit(1);
    }
}
