#!/usr/bin/env node
// SPDX-License-Identifier: MIT
// Copyright (c) 2026 CrispStrobe contributors
//
// Reproduce MakeCode's native C++ build request for an Arcade hardware variant,
// offline, from the admitted Arcade 4.2.1 target bundle and PXT 13.2.1 worker.
// Both payloads are byte-bound before evaluation; network access is refused.
// The request is what MakeCode's cloud compiler would receive; extinfo.sha is
// SHA-256 of exactly these bytes and names the base the compiler links onto.
//
// usage: request.mjs TARGET_JSON PXTWORKER_JS VARIANT OUT_JSON
import crypto from 'node:crypto';
import fs from 'node:fs';
import util from 'node:util';
import vm from 'node:vm';

export const TARGET_SHA256 = '6f35aadf1456436b3318d2f2339cc3d1add6e6c3b0bf8566f86d7f7bb4f7722e';
export const WORKER_SHA256 = '1d9e73fd4bc0c0939b938395537ff1934b398654f9026f88e9293c32ea5acb0e';
// The empty Arcade project every base is keyed on.
export const EMPTY_PROJECT = Object.freeze({
    'pxt.json': JSON.stringify({name: 'arcade-base', dependencies: {device: '*'}, files: ['main.ts'], binaryonly: true}),
    'main.ts': ''
});

const sha256 = bytes => crypto.createHash('sha256').update(bytes).digest('hex');

export async function nativeRequest (targetBytes, workerBytes, variant, files = EMPTY_PROJECT) {
    if (sha256(targetBytes) !== TARGET_SHA256) throw new Error('target.json is not the admitted payload');
    if (sha256(workerBytes) !== WORKER_SHA256) throw new Error('pxtworker.js is not the admitted payload');
    const bundle = JSON.parse(targetBytes.toString('utf8'));
    // pxt strips the C++ files from the request unless asked to keep them.
    bundle.compile.keepCppFiles = true;
    const quiet = () => {};
    const network = [];
    const sandbox = {
        setTimeout, clearTimeout, setInterval, clearInterval, setImmediate, clearImmediate,
        TextEncoder: util.TextEncoder, TextDecoder: util.TextDecoder, Buffer,
        console: {log: quiet, debug: quiet, info: quiet, warn: quiet, error: quiet},
        pxtTargetBundle: bundle,
        __network: network, __variant: variant, __files: files
    };
    sandbox.global = sandbox;
    sandbox.self = sandbox;
    vm.createContext(sandbox);
    vm.runInContext(workerBytes.toString('utf8'), sandbox, {filename: 'pxtworker.js'});
    const extinfo = JSON.parse(await vm.runInContext(`(async () => {
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
        return JSON.stringify(options.extinfo);
    })()`, sandbox));
    if (network.length) throw new Error('request generation attempted network access: ' + network.join(', '));
    if (!extinfo || !extinfo.compileData) throw new Error('pxt formed no native build request');
    const data = Buffer.from(extinfo.compileData, 'base64');
    if (sha256(data) !== extinfo.sha) throw new Error('request bytes do not hash to extinfo.sha');
    return {sha: extinfo.sha, data, request: JSON.parse(data.toString('utf8'))};
}

if (import.meta.url === `file://${process.argv[1]}`) {
    const [targetPath, workerPath, variant, out, mainTs] = process.argv.slice(2);
    if (!out) throw new Error('usage: request.mjs TARGET_JSON PXTWORKER_JS VARIANT OUT_JSON [MAIN_TS]');
    // A program selects the extensions its native request includes (serial, ...).
    const project = mainTs ? {...EMPTY_PROJECT, 'main.ts': fs.readFileSync(mainTs, 'utf8')} : EMPTY_PROJECT;
    const result = await nativeRequest(fs.readFileSync(targetPath), fs.readFileSync(workerPath), variant, project);
    fs.writeFileSync(out, result.data);
    const files = Object.keys(result.request.replaceFiles).length;
    console.log(`PXT_NATIVE_REQUEST variant=${variant} sha256=${result.sha} config=${result.request.config} tag=${result.request.tag} files=${files}`);
}
