# Phase 6: GitHub CI/CD - Research

**Researched:** 2026-03-16
**Domain:** GitHub Actions CI/CD for Tauri 2 + Rust + React desktop app
**Confidence:** HIGH

## Summary

This phase configures GitHub Actions for a Windows-only Tauri 2 desktop application. The app uses Win32 APIs (`windows` crate) with `#[cfg(windows)]` gates throughout `game_capture`, `safety`, and `auto_applier` modules. The codebase already provides `#[cfg(not(windows))]` stubs in `lib.rs` so it **compiles on all platforms**, but meaningful tests for Win32-dependent modules can only run on Windows runners.

The CI pipeline should have two workflows: (1) a **CI workflow** on every push/PR that runs frontend type-checking, Rust compilation, clippy lints, and cargo tests on a Windows runner; and (2) a **Release workflow** triggered manually or by tag push that builds Windows installers using `tauri-apps/tauri-action`.

**Primary recommendation:** Use `windows-latest` GitHub runner for both CI and release. Do not attempt Linux/macOS builds -- this is a Windows-only app. Use `tauri-apps/tauri-action@v0` for release packaging.

## Standard Stack

### Core

| Tool | Version | Purpose | Why Standard |
|------|---------|---------|--------------|
| `tauri-apps/tauri-action` | `@v0` | Build Tauri app + create GitHub Release | Official Tauri action, handles NSIS/MSI bundling |
| `actions/checkout` | `@v4` | Clone repository | Standard |
| `actions/setup-node` | `@v4` | Install Node.js for frontend build | Standard |
| `dtolnay/rust-toolchain` | `@stable` | Install Rust toolchain | Community standard, faster than rustup manual |
| `swatinem/rust-cache` | `@v2` | Cache Cargo build artifacts | Dramatically speeds up Rust CI builds |

### Supporting

| Tool | Version | Purpose | When to Use |
|------|---------|---------|-------------|
| `actions/upload-artifact` | `@v4` | Upload build artifacts | CI build verification |
| `actions/cache` | `@v4` | Cache npm dependencies | Frontend build speed |

### Not Needed (per project context)

| Originally Mentioned | Why Not Needed |
|---------------------|----------------|
| Docker packaging | Desktop app, not a service |
| npm publish | Private app, not a library |
| Linux/macOS runners | Windows-only app (Win32 APIs) |

## Architecture Patterns

### Recommended Workflow Structure

```
.github/
  workflows/
    ci.yml           # Runs on every push/PR to main
    release.yml      # Runs on tag push (v*) or manual dispatch
```

### Pattern 1: CI Workflow (ci.yml)

**What:** Compilation check, linting, testing, frontend type-check on every push/PR
**When to use:** Every push to main, every PR

```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  check-frontend:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: lts/*
          cache: npm
      - run: npm ci
      - run: npx tsc --noEmit
      - run: npm run build  # Vite build (frontend only)

  check-rust:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: swatinem/rust-cache@v2
        with:
          workspaces: './src-tauri -> target'
      - name: Clippy
        run: cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
      - name: Tests
        run: cargo test --manifest-path src-tauri/Cargo.toml
      - name: Build check
        run: cargo check --manifest-path src-tauri/Cargo.toml
```

**Key insight:** Frontend type-checking can run on `ubuntu-latest` (cheaper, faster). Rust must run on `windows-latest` because the `windows` crate v0.61 with Win32 features only compiles on Windows targets.

### Pattern 2: Release Workflow (release.yml)

**What:** Build Windows installer and upload to GitHub Release
**When to use:** Tag push matching `v*` pattern, or manual dispatch

```yaml
name: Release

on:
  workflow_dispatch:
  push:
    tags:
      - 'v*'

jobs:
  release:
    permissions:
      contents: write
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: lts/*
          cache: npm
      - uses: dtolnay/rust-toolchain@stable
      - uses: swatinem/rust-cache@v2
        with:
          workspaces: './src-tauri -> target'
      - run: npm ci
      - uses: tauri-apps/tauri-action@v0
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        with:
          tagName: v__VERSION__
          releaseName: 'Diablo4 Build Applier v__VERSION__'
          releaseBody: 'See assets to download the Windows installer.'
          releaseDraft: true
          prerelease: false
```

### Anti-Patterns to Avoid

- **Building on Linux/macOS:** The `windows` crate with Win32 features will not compile. The `#[cfg(not(windows))]` stubs allow compilation but not meaningful testing.
- **Using `actions-rs/cargo`:** Deprecated / unmaintained. Use `dtolnay/rust-toolchain` + direct cargo commands.
- **Skipping Rust cache:** Without `swatinem/rust-cache`, Rust builds take 10-15 minutes. With cache, incremental builds take 2-3 minutes.
- **Running `cargo test` on ubuntu:** Tests in `game_capture`, `safety/detector`, `safety/hotkey` modules use Win32 types that won't exist. Only `web_parser` and `auto_applier/coords` tests are platform-independent.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Tauri packaging | Custom MSI/NSIS scripts | `tauri-apps/tauri-action@v0` | Handles signing, bundling, upload to GitHub Release |
| Rust caching | Manual target/ caching | `swatinem/rust-cache@v2` | Handles cache keys, workspace detection, pruning |
| Release creation | Manual `gh release create` | `tauri-apps/tauri-action` with tagName | Atomic build+release, handles version extraction |
| Node caching | Manual npm cache | `actions/setup-node` with `cache: npm` | Built-in, handles lock file hashing |

## Common Pitfalls

### Pitfall 1: Windows Runner Costs

**What goes wrong:** Windows runners cost 2x Linux runner minutes on GitHub Actions
**Why it happens:** GitHub charges different rates per runner OS
**How to avoid:** Split frontend checks to `ubuntu-latest` (free tier friendly). Only use `windows-latest` for Rust compilation and testing.
**Warning signs:** Unexpectedly high Actions minutes usage

### Pitfall 2: Rust Build Time

**What goes wrong:** Cold Rust builds take 10-15 minutes, busting CI feedback loop
**Why it happens:** Compiling the `windows` crate with many Win32 features is expensive
**How to avoid:** Always use `swatinem/rust-cache@v2`. Set `workspaces: './src-tauri -> target'` since Cargo.toml is in a subdirectory.
**Warning signs:** CI taking >5 minutes after initial warm-up

### Pitfall 3: GITHUB_TOKEN Permissions

**What goes wrong:** Release workflow fails with 403 creating release
**Why it happens:** Default GITHUB_TOKEN has read-only permissions
**How to avoid:** Add `permissions: contents: write` to the release job
**Warning signs:** "Resource not accessible by integration" error

### Pitfall 4: tauri-action projectPath

**What goes wrong:** tauri-action can't find `tauri.conf.json`
**Why it happens:** Tauri project is in `src-tauri/` subdirectory, not repo root
**How to avoid:** In this project, `tauri.conf.json` is at `src-tauri/tauri.conf.json` which is the default Tauri convention. The action should auto-detect it. If not, set `projectPath: src-tauri`.
**Warning signs:** "Could not find tauri.conf.json" error

### Pitfall 5: npm ci vs npm install

**What goes wrong:** Non-deterministic builds or missing lockfile error
**Why it happens:** `npm install` can modify lockfile; `npm ci` requires exact lockfile
**How to avoid:** Use `npm ci` in CI (requires `package-lock.json` to be committed). If the project currently uses `npm install` without a lockfile, generate one first with `npm install` locally and commit it.
**Warning signs:** "npm warn: could not read lockfile" or inconsistent builds

### Pitfall 6: Tauri v2 CLI Version Mismatch

**What goes wrong:** Build fails with incompatible Tauri CLI version
**Why it happens:** `tauri-apps/tauri-action` installs its own global `@tauri-apps/cli` which may differ from the project's devDependency
**How to avoid:** Ensure `@tauri-apps/cli` version in `package.json` devDependencies is pinned and compatible. The action will use the project's local CLI when `npm ci` has been run.
**Warning signs:** "incompatible plugin" or schema validation errors

## Code Examples

### Minimal CI Workflow

```yaml
# .github/workflows/ci.yml
# Source: https://v2.tauri.app/distribute/pipelines/github/
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  frontend:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: lts/*
          cache: npm
      - run: npm ci
      - run: npx tsc --noEmit

  rust:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: swatinem/rust-cache@v2
        with:
          workspaces: './src-tauri -> target'
      - name: Clippy lint
        run: cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
      - name: Unit tests
        run: cargo test --manifest-path src-tauri/Cargo.toml
```

### Release Workflow with Draft Release

```yaml
# .github/workflows/release.yml
# Source: https://v2.tauri.app/distribute/pipelines/github/
name: Release

on:
  workflow_dispatch:
  push:
    tags:
      - 'v*'

jobs:
  build-and-release:
    permissions:
      contents: write
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: lts/*
          cache: npm
      - uses: dtolnay/rust-toolchain@stable
      - uses: swatinem/rust-cache@v2
        with:
          workspaces: './src-tauri -> target'
      - run: npm ci
      - uses: tauri-apps/tauri-action@v0
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        with:
          tagName: v__VERSION__
          releaseName: 'Diablo4 Build Applier v__VERSION__'
          releaseBody: 'Windows installer for Diablo4 Build Applier.'
          releaseDraft: true
          prerelease: false
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `actions-rs/cargo` | `dtolnay/rust-toolchain` + direct cargo | 2023 | actions-rs unmaintained |
| `actions-rs/clippy-check` | Direct `cargo clippy` | 2023 | Simpler, no extra action needed |
| Manual NSIS scripts | `tauri-apps/tauri-action@v0` | Tauri v2 GA | Handles all bundling |
| npm install in CI | npm ci | Standard practice | Deterministic, faster |

## Open Questions

1. **package-lock.json**
   - What we know: `npm ci` requires a committed lockfile
   - What's unclear: Whether the project currently has one committed (it's in `.gitignore` or just not generated)
   - Recommendation: Generate and commit `package-lock.json` before creating CI workflow. Run `npm install` locally and commit the lockfile.

2. **Code signing**
   - What we know: Windows code signing requires a certificate and is optional for GitHub releases
   - What's unclear: Whether the user wants signed releases
   - Recommendation: Skip for initial CI/CD setup. Can be added later by setting `TAURI_SIGNING_PRIVATE_KEY` secret.

3. **Integration tests (network-dependent)**
   - What we know: `cargo test -- --ignored` runs integration tests that hit d2core.com API
   - What's unclear: Whether these should run in CI (flaky due to external dependency)
   - Recommendation: Do NOT run `--ignored` tests in CI. Only run offline unit tests. Integration tests are for local development.

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in test + TypeScript compiler |
| Config file | `src-tauri/Cargo.toml` (test config), `tsconfig.json` |
| Quick run command | `cargo test --manifest-path src-tauri/Cargo.toml` |
| Full suite command | `cargo test --manifest-path src-tauri/Cargo.toml && npx tsc --noEmit` |

### Phase Requirements -> Test Map

Phase 6 has TBD requirements. The CI/CD workflows themselves are the deliverable. Validation is:

| Behavior | Test Type | Automated Command |
|----------|-----------|-------------------|
| CI workflow runs on push | Manual | Push to branch, check Actions tab |
| Clippy passes | Automated | `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings` |
| Unit tests pass | Automated | `cargo test --manifest-path src-tauri/Cargo.toml` |
| Frontend type-checks | Automated | `npx tsc --noEmit` |
| Release workflow builds installer | Manual | Create tag `v0.1.0`, check release draft |

### Wave 0 Gaps

- [ ] `.github/workflows/ci.yml` -- CI workflow file (does not exist yet)
- [ ] `.github/workflows/release.yml` -- Release workflow file (does not exist yet)
- [ ] `package-lock.json` -- Required for `npm ci` (may not be committed)

## Sources

### Primary (HIGH confidence)
- [Tauri v2 GitHub Actions docs](https://v2.tauri.app/distribute/pipelines/github/) - Official workflow examples
- [tauri-apps/tauri-action](https://github.com/tauri-apps/tauri-action) - Official action README, inputs, outputs
- Project source inspection - `Cargo.toml`, `lib.rs`, `#[cfg(windows)]` gates

### Secondary (MEDIUM confidence)
- [Ship Tauri v2 with GitHub Actions (DEV Community)](https://dev.to/tomtomdu73/ship-your-tauri-v2-app-like-a-pro-github-actions-and-release-automation-part-22-2ef7) - Practical walkthrough

### Tertiary (LOW confidence)
- None

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - Official Tauri docs provide exact workflow examples
- Architecture: HIGH - Simple two-workflow structure is well-documented standard
- Pitfalls: HIGH - Based on direct source code inspection of `#[cfg(windows)]` gates and project structure

**Research date:** 2026-03-16
**Valid until:** 2026-06-16 (stable domain, GitHub Actions and Tauri action are mature)
