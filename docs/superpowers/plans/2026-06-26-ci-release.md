# CI Build & Release Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the devapp Tauri app for Linux, Windows, and macOS (arm64) in GitHub Actions and publish a GitHub prerelease on every push to `main`, versioned with CalVer `YY.M.D-n`.

**Architecture:** A single workflow with two jobs — `check` (fast feedback on every commit) and `release` (a 3-OS matrix on `main` only). A small Deno script computes the date-based version at build time, patches the bundle version into three config files in the CI working copy only (never committed), and exports `VERSION`/`TAG` for `tauri-apps/tauri-action`.

**Tech Stack:** GitHub Actions, `tauri-apps/tauri-action@v0`, Deno (version script + tests), Rust/Tauri 2, SvelteKit.

## Global Constraints

- Use Deno as the JS tooling entrypoint (`deno task ...`), never call `npm`/`pnpm`/`yarn` directly in scripts or docs. (npm is invoked internally by `deno task` and by the CI `npm install` setup step only.)
- Source files keep `version: "0.1.0"`; the CalVer version is patched in CI only and must never be committed to the repo.
- Bundle version format MUST be valid strict SemVer AND Windows-safe: `YY.M.D-n` with NO leading zeros (year − 2000, where YY ≤ 255). Windows MSI rejects major/minor > 255; strict SemVer rejects leading zeros like `06`.
- Targets: Linux x86_64 (`deb,appimage`), Windows x86_64 (`msi,nsis`), macOS arm64 (`dmg`). No macOS x86_64.
- Releases are **prereleases** (`latest = false`).
- `tauri-apps/tauri-action@v0` (major moving tag; current is `v0.6.2`).

**Reference spec:** `docs/superpowers/specs/2026-06-26-ci-release-design.md`

---

## File Structure

- **Create `scripts/version_calc.ts`** — pure version computation. Exported `computeVersion(now, existingTags)`. No side effects, fully unit-testable.
- **Create `scripts/version_calc_test.ts`** — Deno tests for `computeVersion`. No external deps (inline assert).
- **Create `scripts/ci_version.ts`** — CI entrypoint. Uses `computeVersion`; runs git, patches the 3 config files, writes `$GITHUB_ENV`. Supports `--dry-run`.
- **Create `.github/workflows/release.yml`** — `check` job (every commit) + `release` matrix job (main + manual).

---

## Task 1: Version computation library (TDD)

**Files:**
- Create: `scripts/version_calc.ts`
- Test: `scripts/version_calc_test.ts`

**Interfaces:**
- Produces: `export function computeVersion(now: Date, existingTags: string[]): VersionResult` where `VersionResult = { version: string; tag: string }`. `version` e.g. `"26.6.26-1"`, `tag` e.g. `"v26.6.26-1"`.

- [ ] **Step 1: Write the failing tests**

Create `scripts/version_calc_test.ts`:

```ts
import { computeVersion } from "./version_calc.ts";

function assertEqual<T>(actual: T, expected: T, msg?: string) {
  const a = JSON.stringify(actual);
  const e = JSON.stringify(expected);
  if (a !== e) {
    throw new Error(
      `Assertion failed${msg ? `: ${msg}` : ""}\n  expected: ${e}\n  actual:   ${a}`,
    );
  }
}

Deno.test("first build of the day gets counter 1", () => {
  const now = new Date(2026, 5, 26); // 2026-06-26 (month is 0-indexed)
  assertEqual(computeVersion(now, []), {
    version: "26.6.26-1",
    tag: "v26.6.26-1",
  });
});

Deno.test("second build same day increments counter", () => {
  const now = new Date(2026, 5, 26);
  assertEqual(computeVersion(now, ["v26.6.26-1"]), {
    version: "26.6.26-2",
    tag: "v26.6.26-2",
  });
});

Deno.test("ignores tags from other days", () => {
  const now = new Date(2026, 5, 26);
  assertEqual(
    computeVersion(now, ["v26.6.25-3", "v26.7.1-1", "v25.12.31-9"]),
    { version: "26.6.26-1", tag: "v26.6.26-1" },
  );
});

Deno.test("ignores non-calver tags", () => {
  const now = new Date(2026, 5, 26);
  assertEqual(
    computeVersion(now, ["v0.1.0", "latest", "26.6.26-1-bogus"]),
    { version: "26.6.26-1", tag: "v26.6.26-1" },
  );
});

Deno.test("uses max counter + 1 when there are gaps", () => {
  const now = new Date(2026, 5, 26);
  assertEqual(computeVersion(now, ["v26.6.26-1", "v26.6.26-5"]), {
    version: "26.6.26-6",
    tag: "v26.6.26-6",
  });
});

Deno.test("no leading zeros for single-digit month and day", () => {
  const now = new Date(2026, 1, 9); // 2026-02-09
  assertEqual(computeVersion(now, []), {
    version: "26.2.9-1",
    tag: "v26.2.9-1",
  });
});
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `deno test scripts/version_calc_test.ts`
Expected: FAIL with an error like `Module not found .../version_calc.ts` or `computeVersion is not a function`.

- [ ] **Step 3: Implement `computeVersion`**

Create `scripts/version_calc.ts`:

```ts
export interface VersionResult {
  version: string;
  tag: string;
}

/**
 * Compute a CalVer version of the form YY.M.D-n, where:
 *  - YY = year - 2000 (Windows MSI requires major <= 255)
 *  - M, D have no leading zeros (strict SemVer forbids leading zeros)
 *  - n is (max existing counter for this date) + 1, defaulting to 1
 */
export function computeVersion(now: Date, existingTags: string[]): VersionResult {
  const yy = now.getFullYear() - 2000;
  const m = now.getMonth() + 1;
  const d = now.getDate();
  const prefix = `${yy}.${m}.${d}`;
  const tagPrefix = `v${prefix}-`;

  let max = 0;
  for (const tag of existingTags) {
    if (!tag.startsWith(tagPrefix)) continue;
    const suffix = tag.slice(tagPrefix.length);
    const n = Number.parseInt(suffix, 10);
    if (Number.isInteger(n) && n > max) max = n;
  }

  const n = max + 1;
  const version = `${prefix}-${n}`;
  return { version, tag: `v${version}` };
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `deno test scripts/version_calc_test.ts`
Expected: `6 passed` (or all passing), exit code 0.

- [ ] **Step 5: Commit**

```bash
git add scripts/version_calc.ts scripts/version_calc_test.ts
git commit -m "ci: add CalVer version computation with tests"
```

---

## Task 2: CI version entrypoint script

**Files:**
- Create: `scripts/ci_version.ts`

**Interfaces:**
- Consumes: `computeVersion` from `./version_calc.ts` (Task 1).
- Produces: a runnable script that (1) reads tags via git, (2) patches `src-tauri/tauri.conf.json`, `package.json`, `src-tauri/Cargo.toml` in the working copy, (3) appends `VERSION=<v>` and `TAG=<v>` to `$GITHUB_ENV`. Accepts `--dry-run` to print the result without writing.

- [ ] **Step 1: Implement the entrypoint**

Create `scripts/ci_version.ts`:

```ts
import { computeVersion } from "./version_calc.ts";

const FILES = {
  tauriConf: "src-tauri/tauri.conf.json",
  packageJson: "package.json",
  cargoToml: "src-tauri/Cargo.toml",
};

const dryRun = Deno.args.includes("--dry-run");

async function git(args: string[]): Promise<string> {
  const cmd = new Deno.Command("git", {
    args,
    stdout: "piped",
    stderr: "piped",
  });
  const { code, stdout, stderr } = await cmd.output();
  const out = new TextDecoder().decode(stdout);
  const err = new TextDecoder().decode(stderr);
  if (code !== 0) {
    throw new Error(`git ${args.join(" ")} failed (${code}): ${err || out}`);
  }
  return out.trim();
}

async function patchJson(path: string, version: string): Promise<void> {
  const text = await Deno.readTextFile(path);
  const json = JSON.parse(text);
  json.version = version;
  await Deno.writeTextFile(path, JSON.stringify(json, null, 2) + "\n");
}

async function patchCargoVersion(path: string, version: string): Promise<void> {
  let text = await Deno.readTextFile(path);
  // Only the [package] version line starts the line with `version =`.
  // Dependency versions live inside `{ ... }` on lines that start with the dep name.
  text = text.replace(
    /^version\s*=\s*"[^"]*"/m,
    `version = "${version}"`,
  );
  await Deno.writeTextFile(path, text);
}

async function main(): Promise<void> {
  await git(["fetch", "--tags", "--force"]);
  const tagList = await git(["tag", "-l", "v*"]);
  const existingTags = tagList ? tagList.split("\n").filter(Boolean) : [];

  const { version, tag } = computeVersion(new Date(), existingTags);

  if (dryRun) {
    console.log(`VERSION=${version}`);
    console.log(`TAG=${tag}`);
    console.log(`(dry-run: no files patched, no GITHUB_ENV written)`);
    return;
  }

  await patchJson(FILES.tauriConf, version);
  await patchJson(FILES.packageJson, version);
  await patchCargoVersion(FILES.cargoToml, version);

  const ghEnv = Deno.env.get("GITHUB_ENV");
  if (ghEnv) {
    const line = `VERSION=${version}\nTAG=${tag}\n`;
    await Deno.writeTextFile(ghEnv, line, { append: true });
  }

  console.log(`VERSION=${version}`);
  console.log(`TAG=${tag}`);
}

main();
```

- [ ] **Step 2: Verify the dry-run output locally**

Run: `deno run --allow-read --allow-run=git scripts/ci_version.ts --dry-run`
Expected: prints two lines like `VERSION=26.6.26-N` and `TAG=v26.6.26-N` (N reflects existing tags for today), then the dry-run note. No files changed. Exit 0.

Then confirm nothing was modified: `git status --porcelain`
Expected: empty (no working-tree changes from a dry run).

- [ ] **Step 3: Verify real patching, then restore the working copy**

Run the real script once: `deno run --allow-read --allow-write --allow-env --allow-run=git scripts/ci_version.ts`
Expected: same `VERSION=`/`TAG=` output. `$GITHUB_ENV` is unset locally so nothing is appended.

Inspect the patched version: `git diff src-tauri/tauri.conf.json package.json src-tauri/Cargo.toml`
Expected: each file's `version` changed from `0.1.0` to today's CalVer (e.g. `26.6.26-N`).

Restore so the version is never committed: `git restore src-tauri/tauri.conf.json package.json src-tauri/Cargo.toml`
Then confirm clean: `git status --porcelain` → empty.

- [ ] **Step 4: Commit**

```bash
git add scripts/ci_version.ts
git commit -m "ci: add CI version entrypoint that patches bundle version"
```

---

## Task 3: GitHub Actions workflow

**Files:**
- Create: `.github/workflows/release.yml`

**Interfaces:**
- Consumes: `scripts/ci_version.ts` (Task 2) in the `release` job; `deno task check`/`build`, `cargo test` in the `check` job.
- Produces: the runnable CI that builds 3 platforms and publishes one GitHub prerelease per main push.

- [ ] **Step 1: Create the workflow**

Create `.github/workflows/release.yml`:

```yaml
name: Release

on:
  push:
  workflow_dispatch:

concurrency:
  group: release-${{ github.ref }}
  cancel-in-progress: true

jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - uses: actions/setup-node@v4
        with:
          node-version: "20"

      - run: npm install

      - uses: denoland/setup-deno@v2
        with:
          deno-version: v2.x

      - run: deno task check
      - run: deno task build
      - run: deno test scripts/

      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
        with:
          workspaces: src-tauri
      - run: cargo test
        working-directory: src-tauri

  release:
    needs: check
    if: github.ref == 'refs/heads/main' || github.event_name == 'workflow_dispatch'
    permissions:
      contents: write
    strategy:
      fail-fast: false
      matrix:
        include:
          - platform: ubuntu-22.04
            args: "--bundles deb,appimage"
          - platform: windows-latest
            args: ""
          - platform: macos-latest
            args: "--target aarch64-apple-darwin --bundles dmg"
    runs-on: ${{ matrix.platform }}
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - uses: denoland/setup-deno@v2
        with:
          deno-version: v2.x

      - uses: actions/setup-node@v4
        with:
          node-version: "20"
      - run: npm install

      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
        with:
          workspaces: src-tauri
      - if: matrix.platform == 'macos-latest'
        run: rustup target add aarch64-apple-darwin
      - if: matrix.platform == 'ubuntu-22.04'
        run: |
          sudo apt-get update
          sudo apt-get install -y libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev

      - name: Compute version
        run: deno run --allow-read --allow-write --allow-env --allow-run=git scripts/ci_version.ts

      - uses: tauri-apps/tauri-action@v0
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        with:
          tagName: ${{ env.TAG }}
          releaseName: devapp ${{ env.VERSION }}
          prerelease: true
          generateReleaseNotes: true
          args: ${{ matrix.args }}
```

- [ ] **Step 2: Validate the YAML parses**

Run: `python3 -c "import yaml; yaml.safe_load(open('.github/workflows/release.yml')); print('YAML OK')"`
Expected: prints `YAML OK`, exit 0.

If `actionlint` is installed, also run it for stricter validation:
`actionlint .github/workflows/release.yml` (informational; not required).

- [ ] **Step 3: Commit**

```bash
git add .github/workflows/release.yml
git commit -m "ci: add build + prerelease workflow for linux/windows/macos-arm64"
```

---

## Task 4: End-to-end verification (manual gate)

**Files:** none (verifies Tasks 1–3).

**Interfaces:**
- Consumes: the workflow from Task 3.

- [ ] **Step 1: Verify the check job on a feature branch**

Push the branch and open the Actions tab. The `check` job must run and pass on the branch push (no release job should run).
Expected: `check` green; `release` not triggered (branch is not `main`).

- [ ] **Step 2: Trigger a release**

Either push to `main`, or use the Actions UI → "Release" → "Run workflow" on `main`.
Expected:
- `check` runs and passes.
- `release` runs a 3-job matrix (ubuntu-22.04, windows-latest, macos-latest), all green.
- Exactly ONE GitHub prerelease is created, named `devapp <VERSION>`, tagged `v<VERSION>` (e.g. `v26.6.26-1`).
- The release contains artifacts from all three platforms (`.deb` + `.AppImage`, `.msi` + `.exe` NSIS, macOS `.dmg`).
- `latest` is `false` (it is a prerelease).

- [ ] **Step 3: Same-day second commit increments the counter**

Make a trivial commit on `main` the same day and push.
Expected: a new prerelease tagged `v26.6.26-2` (counter incremented), distinct from the first.

- [ ] **Step 4: Spot-check an artifact**

Download one installer from the release and confirm it installs/launches on the matching OS.
Expected: app opens (unsigned-build warnings on macOS/Windows are acceptable and documented as V1 behavior).

---

## Notes for the implementer

- The version is patched ONLY in the CI working copy; the repo's `0.1.0` must remain. Task 2 Step 3 demonstrates the local restore.
- `npm install` (not `npm ci`) is intentional — the repo has no lockfile. Adding a lockfile is out of scope.
- Unsigned builds: macOS Gatekeeper and Windows SmartScreen will warn users. This is accepted for V1.
- The `release` tag is created by `GITHUB_TOKEN`, which does not trigger new workflow runs, so there is no loop.
