# CI Build & Release — Design

- Date: 2026-06-26
- Status: Approved
- Owner: LeoMartinDev/devapp

## Goal

Build the devapp Tauri desktop app for Linux, Windows, and macOS (arm64) in
GitHub Actions, and publish a GitHub release on every push to `main`. Use a
date-based (CalVer) versioning scheme.

## Non-goals (V1)

- Code signing / notarization (no secrets). Builds are unsigned.
- Auto-updater (requires signing + a hosted update manifest).
- Remote execution, cloud distribution channels, or stores.
- Building for macOS x86_64 (arm64 only, per decision).
- Persistent historical logs beyond V1 in-memory behavior.

## Decisions

| Decision | Choice |
|----------|--------|
| Release trigger | `check` job on every commit (all branches); `release` job only on `push` to `main` (plus manual `workflow_dispatch`) |
| Architectures | Linux x86_64, Windows x86_64, macOS arm64 |
| Release style | GitHub **prerelease** (`latest = false`) per commit on main |
| Version scheme | CalVer `YY.M.D-n` (Approach A) |

## Version scheme: CalVer `YY.M.D-n`

### Why not `YYYY.MM.DD`

The bundle `version` in `tauri.conf.json` is parsed and converted into
platform-specific formats that impose hard constraints:

- **Windows MSI** requires `major.minor.patch.build` with **major and minor ≤
  255** (per Tauri config reference). `2026` as major exceeds this and would
  break the Windows build.
- **Strict SemVer** (used by Tauri/Cargo) forbids leading zeros in numeric
  identifiers, so `06` is invalid.

### Format

- `YY` = 4-digit year minus 2000 (e.g. `26` for 2026). `26 ≤ 255` is
  Windows-safe.
- `M`, `D` = month and day with **no leading zeros** (e.g. `6`, `9`, `26`).
  Required for SemVer validity.
- `n` = per-day counter starting at 1, incremented when more than one commit
  lands on `main` the same day. Computed at build time from existing git tags.

Examples: `26.6.26-1`, then `26.6.26-2` for a second main commit that day,
`26.7.1-1` the next day.

The same string is used as the **bundle version** (tauri.conf.json / Cargo.toml
/ package.json) **and** the **git tag** (`v26.6.26-1`). One source of truth; the
date is visible in the app's About dialog.

### Source files stay clean

`tauri.conf.json`, `src-tauri/Cargo.toml`, and `package.json` keep `0.1.0` in
the repository. The version is **patched in CI only** (never committed), so git
history is not polluted by version bumps and developers keep working from
`0.1.0` locally.

## Components

### 1. `.github/workflows/release.yml`

Single workflow with two jobs.

**Triggers:**

- `push` (any branch) → `check`
- `push` to `main` → `check` + `release`
- `workflow_dispatch` → `check` + `release` (so the `release` → `check`
  dependency is satisfiable on manual runs)

**Concurrency:** group `release-${{ github.ref }}`, `cancel-in-progress: true`.
Serializes runs per branch and prevents the rare same-day counter race when two
commits land on main near-simultaneously.

#### Job `check` (every commit)

- Runner: `ubuntu-latest`
- Steps:
  1. `actions/checkout@v4`
  2. `actions/setup-node@v4` + `npm install` (no lockfile exists; `npm install`
     rather than `npm ci`)
  3. `denoland/setup-deno@v2`
  4. `deno task check` (svelte-check) and `deno task build` (frontend compiles)
  5. `dtolnay/rust-toolchain@stable` + Rust cache (`Swatinem/rust-cache@v2`,
     scoped to `src-tauri`)
  6. `cargo test` (run from `src-tauri`)
- Fast feedback only; never publishes.

#### Job `release` (main + manual)

- `needs: check` and `if: github.ref == 'refs/heads/main' || github.event_name
  == 'workflow_dispatch'`. (`check` also runs on `workflow_dispatch` so this
  dependency is satisfiable.)
- `fail-fast: false` matrix (jobs run in parallel and upload to **the same
  release**):

  | `platform` | `args` | Notes |
  |------------|--------|-------|
  | `ubuntu-22.04` | `--bundles deb,appimage` | Installs Tauri Linux system deps |
  | `windows-latest` | (none → default msi, nsis) | |
  | `macos-latest` | `--target aarch64-apple-darwin --bundles dmg` | `rustup target add aarch64-apple-darwin` |

- Steps (per matrix entry):
  1. `actions/checkout@v4` with `fetch-depth: 0` (needed to read tags for the
     per-day counter)
  2. `denoland/setup-deno@v2`
  3. `actions/setup-node@v4` + `npm install`
  4. `dtolnay/rust-toolchain@stable` + `Swatinem/rust-cache@v2`; on macOS add
     the arm64 target
  5. (Linux only) install system deps: `libwebkit2gtk-4.1-dev build-essential
     curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev
     librsvg2-dev` (Tauri 2 requirements)
  6. **Compute version**: `deno run --allow-read --allow-write --allow-run
     scripts/ci_version.ts` → patches the three files and exports `VERSION` and
     `TAG` to `$GITHUB_ENV`
  7. `tauri-apps/tauri-action@v0` with:
     - `tagName: ${{ env.TAG }}`
     - `releaseName: devapp ${{ env.VERSION }}`
     - `prerelease: true`
     - `generateReleaseNotes: true`
     - `args: ${{ matrix.args }}`
     - `env: GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}`

The Tauri `beforeBuildCommand` is `deno task build`, so deno + node must be
available in every matrix job (installed in steps 2–3).

### 2. `scripts/ci_version.ts` (Deno)

Single source of the computed version. Run with
`--allow-read --allow-write --allow-run`.

Behavior:

1. Compute today's date parts: `YY = year - 2000`, `M` (1–12), `D` (1–31),
   all without leading zeros. Build the date prefix `YY.M.D`.
2. `git fetch --tags` then list tags matching `v{prefix}-*` to compute the next
   counter `n` (max existing + 1, default 1).
3. Final `VERSION = "{prefix}-{n}"`, `TAG = "v{VERSION}"`.
4. Patch (in place, working copy only):
   - `tauri.conf.json` → `.version = VERSION` (JSON parse/serialize, preserves
     key order).
   - `package.json` → `.version = VERSION`.
   - `src-tauri/Cargo.toml` → the `version = "..."` line under `[package]`
     (regex on first top-level match; leaves dependency `version = "..."` inside
     braces untouched).
5. Append to `$GITHUB_ENV`:
   ```
   VERSION=<value>
   TAG=<value>
   ```

All three matrix jobs compute the same value deterministically (same day, same
shared tags). The first job to reach tauri-action creates the release; later
jobs attach their artifacts. tauri-action retries on concurrent-release
conflicts.

## Data flow

```
push (any branch)      ─► check ─────────────────────────────────► done
push (main)            ─► check ─► release (matrix) ─► tauri-action ─► GitHub prerelease
workflow_dispatch      ─► check ─► release (matrix) ─► tauri-action ─► GitHub prerelease
                                              └─ ci_version.ts patches 3 files
```

## Error handling & edge cases

- **Counter race:** two main commits the same second could both compute the same
  counter. Mitigated by the per-branch concurrency group; acceptable for a solo
  developer. If a release already exists for the tag, tauri-action attaches
  rather than failing.
- **Unsigned builds:** macOS Gatekeeper and Windows SmartScreen will warn.
  Users run via right-click → Open. Documented as V1 behavior.
- **No lockfile:** `npm install` (not `npm ci`). Less reproducible but the
  project currently ships no lockfile; out of scope to add one here.
- **Version never committed:** a failed/cancelled run leaves a patched working
  copy in CI only; the repository is unaffected.

## Testing

- Local dry-run of `scripts/ci_version.ts` against a temp checkout with mock
  tags to verify counter logic (e.g. first-of-day = `-1`, second = `-2`) and
  that the three files are patched correctly without touching dependency
  versions.
- Verify the produced `VERSION` is valid SemVer and Windows-safe (major ≤ 255,
  no leading zeros).
- First real run: push a no-op commit to `main` and confirm three artifacts
  attach to one prerelease, the tag matches `vYY.M.D-1`, and each platform's
  installer installs/launches.
- `cargo test` must remain green in the `check` job.
