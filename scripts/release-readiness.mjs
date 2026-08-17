#!/usr/bin/env node
// The release-readiness reminder — the "merged ≠ released" trap, watched.
//
// A feat/fix that touches runtime code but ships no version bump strands its
// change on main the way the v0.2.2 dispatch fix was stranded: merged, green,
// and in nobody's hands. This script compares the PR's base..head and WARNS
// (a GitHub annotation, never a failure — lab Decision 017 / Pattern 024: new
// enforcement starts as warn) when runtime files change while package.json's
// version stands still.
//
// Dependency-free by design: Node built-ins + git only, so the workflow runs
// on a bare runner with no install step. Invoked by version-lockstep.yml with
// BASE_REF/HEAD_REF in the environment.

import { execFileSync } from "node:child_process";

const base = process.env.BASE_REF;
const head = process.env.HEAD_REF;
if (!base || !head) {
  console.log(
    "release-readiness: BASE_REF/HEAD_REF not provided — nothing to compare, standing down.",
  );
  process.exit(0);
}

const git = (...args) => execFileSync("git", args, { encoding: "utf8" });

// Advisory means advisory: an unexpected git or parse failure stands down
// with a note instead of going red — this job never fails the PR.
process.on("uncaughtException", (err) => {
  console.log(`release-readiness: could not compare (${err.message.split("\n")[0]}) — standing down, advisory only.`);
  process.exit(0);
});

// What counts as runtime: the code an installed Ledger actually executes.
// Docs, tests, workflows, and bench scripts merge without owing a release.
const RUNTIME = [
  /^src\//,
  /^src-tauri\/src\//,
  /^src-tauri\/tauri\.conf\.json$/,
  /^src-tauri\/Cargo\.toml$/,
];

const changed = git("diff", "--name-only", `${base}..${head}`)
  .split("\n")
  .filter(Boolean);
const runtimeChanged = changed.filter((file) => RUNTIME.some((re) => re.test(file)));

const versionAt = (ref) => JSON.parse(git("show", `${ref}:package.json`)).version;
const baseVersion = versionAt(base);
const headVersion = versionAt(head);

if (runtimeChanged.length === 0) {
  console.log(
    `release-readiness: no runtime files change in this PR (v${headVersion}) — no release owed.`,
  );
} else if (baseVersion !== headVersion) {
  console.log(
    `release-readiness: runtime changes AND the version moves ${baseVersion} → ${headVersion} — the shipment is provisioned. Tag v${headVersion} after merge.`,
  );
} else {
  const listed = runtimeChanged.slice(0, 10).join(", ");
  console.log(
    `::warning title=Merged is not released::${runtimeChanged.length} runtime file(s) change with no version bump (still v${headVersion}): ${listed}. ` +
      "If this is user-facing, the change reaches no installed Ledger until a bump + tag fires the Shipment (npm run version:bump <v>). Advisory only — the author decides whether a release is owed.",
  );
}
process.exit(0);
