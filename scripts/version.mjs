#!/usr/bin/env node
// Version discipline for the Ascent (#00056, Phase A-4).
//
// The Smuggler’s Ledger's version lives in FOUR tracked manifests that MUST agree,
// because the updater compares the running version (tauri.conf.json, baked
// into the binary) against the release manifest. If package.json says 0.2.0
// while tauri.conf.json still says 0.1.0, a release tagged v0.2.0 ships a
// binary that reports 0.1.0 — and the updater would re-offer the same version
// forever, or never. This script is the single source that keeps them locked.
//
//   node scripts/version.mjs check          — assert all four agree (CI gate)
//   node scripts/version.mjs bump 0.2.0      — set all four to 0.2.0
//
// The fourth manifest is package-lock.json — it carries the version twice
// (top-level + the root `packages[""]` entry). It is easy to forget because
// `bump` historically only touched the three "real" manifests; the lockfile
// then drifts, and the RELEASE job's `npm ci` fails ("can only install when
// package.json and package-lock.json are in sync"). Cost a hand-sync before
// v0.2.3 (2026-06-09). Folding the lockfile into both `bump` and `check`
// makes the desync impossible to ship: `bump` rewrites it, and the existing
// version-lockstep CI gate (which runs `check`) now fails the moment it drifts.
// Cargo.lock is NOT bumped by this script, but (as of v0.2.5) it IS tracked —
// committing it pins the release's dependency tree so CI can't drift onto a
// broken transitive version (it did: a fresh resolve pulled `tauri-utils 2.9.2`,
// which fails to compile, while local stayed green on 2.9.1). The lockfile's
// own `smugglers-ledger` [[package]] version is synced separately via
// `cargo update --workspace` (or any `cargo build`), not by this script — a
// one-version lag there is harmless (cargo rewrites it in place on build).
//
// No dependencies — plain Node fs + scoped regex edits (the Cargo.toml
// [package] version only, never inline dep tables; the package-lock.json
// "smugglers-ledger" version sites only, never the dependency `"version"` fields).

import {readFileSync, writeFileSync} from 'node:fs';
import {dirname, resolve} from 'node:path';
import {fileURLToPath} from 'node:url';

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const PACKAGE_JSON = resolve(ROOT, 'package.json');
const TAURI_CONF = resolve(ROOT, 'src-tauri/tauri.conf.json');
const CARGO_TOML = resolve(ROOT, 'src-tauri/Cargo.toml');
const PACKAGE_LOCK = resolve(ROOT, 'package-lock.json');

const SEMVER = /^\d+\.\d+\.\d+$/;

/** Read the [package] version from Cargo.toml — scoped, never an inline dep. */
function readCargoVersion(text) {
    const lines = text.split('\n');
    let inPackage = false;
    for (const line of lines) {
        const header = line.trim();
        if (header.startsWith('[')) {
            inPackage = header === '[package]';
            continue;
        }
        if (inPackage) {
            const match = line.match(/^\s*version\s*=\s*"([^"]+)"/);
            if (match) {
                return match[1];
            }
        }
    }
    return null;
}

/** Replace the [package] version in Cargo.toml, leaving inline deps untouched. */
function writeCargoVersion(text, version) {
    const lines = text.split('\n');
    let inPackage = false;
    let done = false;
    const next = lines.map((line) => {
        const header = line.trim();
        if (header.startsWith('[')) {
            inPackage = header === '[package]';
            return line;
        }
        if (inPackage && !done && /^\s*version\s*=\s*"[^"]+"/.test(line)) {
            done = true;
            return line.replace(/("version"|version)(\s*=\s*)"[^"]+"/, `version$2"${version}"`);
        }
        return line;
    });
    if (!done) {
        throw new Error('Could not find a [package] version line in Cargo.toml');
    }
    return next.join('\n');
}

function readVersions() {
    const pkg = JSON.parse(readFileSync(PACKAGE_JSON, 'utf8'));
    const conf = JSON.parse(readFileSync(TAURI_CONF, 'utf8'));
    const cargo = readCargoVersion(readFileSync(CARGO_TOML, 'utf8'));
    const lock = JSON.parse(readFileSync(PACKAGE_LOCK, 'utf8'));
    return {
        'package.json': pkg.version,
        'tauri.conf.json': conf.version,
        'Cargo.toml': cargo,
        'package-lock.json': lock.version,
        'package-lock.json (root pkg)': lock.packages?.['']?.version,
    };
}

function check() {
    const versions = readVersions();
    const distinct = new Set(Object.values(versions));
    for (const [file, version] of Object.entries(versions)) {
        console.log(`  ${file.padEnd(30)} ${version ?? '(not found)'}`);
    }
    if (distinct.size === 1 && !distinct.has(undefined) && !distinct.has(null)) {
        console.log(`\nThe ledger's four manifests agree: v${[...distinct][0]}.`);
        return;
    }
    console.error('\nVersion drift: the four manifests disagree. Run `npm run version:bump <version>`.');
    process.exit(1);
}

function bump(version) {
    if (!version || !SEMVER.test(version)) {
        console.error(`Expected a semver version like 0.2.0, got: ${version ?? '(nothing)'}`);
        process.exit(1);
    }

    // Targeted replacement of the single top-level "version" line in each
    // JSON manifest — a JSON round-trip would expand the hand-kept inline
    // objects (e.g. "security": {"csp": null}) and churn the file. The
    // dependency entries carry no "version" key, so the first match is safe.
    const replaceJsonVersion = (path) => {
        const raw = readFileSync(path, 'utf8');
        if (!/("version"\s*:\s*)"[^"]+"/.test(raw)) {
            throw new Error(`Could not find a "version" field in ${path}`);
        }
        writeFileSync(path, raw.replace(/("version"\s*:\s*)"[^"]+"/, `$1"${version}"`));
    };
    replaceJsonVersion(PACKAGE_JSON);
    replaceJsonVersion(TAURI_CONF);

    const cargoRaw = readFileSync(CARGO_TOML, 'utf8');
    writeFileSync(CARGO_TOML, writeCargoVersion(cargoRaw, version));

    // package-lock.json carries the version twice — the top-level field and
    // the root package entry (`packages[""]`). Both sit immediately after a
    // `"name": "smugglers-ledger",` line and nowhere else in the file (dependency
    // entries are never named "smugglers-ledger"), so one scoped replace-all hits
    // exactly those two without a JSON round-trip that would reflow the whole
    // lockfile (and churn line endings on Windows — see Pattern 018).
    const lockRaw = readFileSync(PACKAGE_LOCK, 'utf8');
    const lockPattern = /("name":\s*"smugglers-ledger",\s*"version":\s*)"[^"]+"/g;
    const lockSites = lockRaw.match(lockPattern);
    if (!lockSites || lockSites.length !== 2) {
        throw new Error(
            `Expected exactly 2 "smugglers-ledger" version sites in package-lock.json, found ${lockSites?.length ?? 0}. ` +
                'The lockfile shape changed — fix scripts/version.mjs before bumping.',
        );
    }
    writeFileSync(PACKAGE_LOCK, lockRaw.replace(lockPattern, `$1"${version}"`));

    console.log(`The ledger is now v${version} across all four manifests. Tag it: git tag v${version}`);
}

const [, , mode, arg] = process.argv;
if (mode === 'check') {
    check();
} else if (mode === 'bump') {
    bump(arg);
} else {
    console.error('Usage: node scripts/version.mjs <check | bump <version>>');
    process.exit(1);
}
