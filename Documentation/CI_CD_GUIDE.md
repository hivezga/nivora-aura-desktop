# CI/CD Guide

## Overview

Aura Desktop uses GitHub Actions for automated testing, building, and releasing across all supported platforms (Windows, macOS, Linux).

## Workflows

### 1. Continuous Integration (CI)

**File:** `.github/workflows/ci.yml`

**Triggers:**
- Every push to `main` branch
- Every pull request targeting `main` branch

**Purpose:**
- Validate code compiles on all platforms
- Run Rust tests and linting
- Check code formatting
- Build complete application packages

**Jobs:**

#### Build Job (Matrix Strategy)
Runs on: `ubuntu-latest`, `macos-latest`, `windows-latest`

**Steps:**
1. Checkout code with Git LFS support
2. Install pnpm, Node.js, and Rust
3. Install platform-specific system dependencies
4. Install frontend dependencies
5. Lint Rust code with Clippy
6. Run Rust unit tests
7. Build frontend
8. Build Tauri application
9. Upload build artifacts for inspection

**Expected Duration:** 15-25 minutes per platform

#### Format Check Job
Runs on: `ubuntu-latest`

**Steps:**
1. Check Rust code formatting with `rustfmt`
2. (Optional) Check frontend formatting

**Expected Duration:** 2-3 minutes

### 2. Release Automation (CD)

**File:** `.github/workflows/release.yml`

**Triggers:**
- Push of version tag (e.g., `v0.1.0`, `v1.2.3`)

**Purpose:**
- Build production installers for all platforms
- Create GitHub release with artifacts
- Generate release notes

**Jobs:**

#### Build Tauri Job (Matrix Strategy)
Runs on: `ubuntu-latest`, `macos-latest`, `windows-latest`

**Targets:**
- **Linux:** `x86_64-unknown-linux-gnu` (AppImage, DEB, RPM)
- **macOS Intel:** `x86_64-apple-darwin` (DMG)
- **macOS Apple Silicon:** `aarch64-apple-darwin` (DMG)
- **Windows:** `x86_64-pc-windows-msvc` (NSIS, MSI)

**Steps:**
1. Checkout code with Git LFS support
2. Install dependencies
3. Build and package application using `tauri-action`
4. Upload artifacts to draft GitHub release

**Expected Duration:** 20-35 minutes per platform

**Output Artifacts:**
- **Windows:** `aura-desktop_X.Y.Z_x64-setup.exe` (~3.6 GB)
- **macOS:** `Aura Desktop_X.Y.Z_x64.dmg` (~1.8 GB)
- **Linux:**
  - `aura-desktop_X.Y.Z_amd64.AppImage` (~1.8 GB)
  - `aura-desktop_X.Y.Z_amd64.deb` (~1.7 GB)
  - `aura-desktop-X.Y.Z-1.x86_64.rpm` (~1.8 GB)

#### Finalize Release Job
Runs on: `ubuntu-latest`

**Steps:**
1. Generate release notes
2. Notify that draft release is ready for review

### 3. Dependency Management (Dependabot)

**File:** `.github/dependabot.yml`

**Schedule:** Weekly (Monday mornings)

**Purpose:**
- Automatically create PRs for dependency updates
- Keep project secure and up-to-date

**Monitored Ecosystems:**
- **Rust (Cargo):** Dependencies in `src-tauri/Cargo.toml`
- **Node.js (npm):** Dependencies in `package.json`
- **GitHub Actions:** Workflow action versions

**Configuration:**
- Grouped updates to reduce PR noise
- Maximum 5 open PRs per ecosystem
- Labeled for easy filtering

## Important Considerations

### Git LFS Requirements

**Critical:** All workflows require Git LFS to be enabled and properly configured.

The project uses Git LFS for:
- Gemma 2B model (1.7 GB)
- Ollama GPU libraries (3.4 GB)
- Piper TTS models (122 MB)

**Workflow Configuration:**
```yaml
- uses: actions/checkout@v4
  with:
    lfs: true  # Enable LFS

# LFS caching to reduce bandwidth
- uses: actions/cache@v4
  with:
    path: .git/lfs
    key: lfs-${{ runner.os }}-${{ hashFiles('.gitattributes') }}

- run: git lfs pull  # Fetch LFS objects
```

**GitHub Actions LFS Bandwidth:**
- Free tier: 1 GB/month bandwidth
- First workflow run: Downloads ~5.3 GB of LFS data
- Subsequent runs: Downloads only changed files (~250 MB with caching)

**Impact of Optimizations:**
- ✅ **LFS caching implemented:** 90-95% bandwidth reduction
- ✅ **paths-ignore implemented:** 30-40% fewer CI runs
- **Combined:** Free tier may now be sufficient for moderate development activity
- **Recommendation:** Monitor bandwidth usage and upgrade if needed

**GitHub LFS Bandwidth Tiers:**
1. **Free:** 1 GB/month (~4 cached runs or 0.2 full runs)
2. **Pro ($4/month):** 50 GB/month (~200 cached runs or 10 full runs)
3. **Team ($4/user/month):** 100 GB/month (~400 cached runs or 20 full runs)

### Build Times and Resources

**Expected CI Build Times (First Run):**
- **Linux:** 15-20 minutes
- **macOS:** 20-25 minutes
- **Windows:** 15-20 minutes
- **Total (all platforms in parallel):** ~50-65 minutes

**Expected CI Build Times (Cached Runs):**
- **Linux:** 5-8 minutes
- **macOS:** 8-10 minutes
- **Windows:** 5-8 minutes
- **Total (all platforms in parallel):** ~15-25 minutes

**Build Time Breakdown:**
- LFS checkout: ~30s (cached) vs ~5-10 minutes (uncached)
- Rust compilation: ~3-5 minutes (cached) vs ~10-15 minutes (uncached)
- Frontend build: ~1-2 minutes (consistent)
- Tauri bundling: ~2-3 minutes (consistent)

**GitHub Actions Free Tier Limits:**
- Public repos: Unlimited minutes
- Private repos: 2,000 minutes/month
- Concurrent jobs: 20 (free tier)

**Resource Usage Per Build:**
- **Disk space:** ~20 GB (LFS files + build artifacts)
- **Memory:** ~4-8 GB peak
- **CPU:** All available cores

### Platform-Specific Notes

#### Linux (ubuntu-latest)
**System Dependencies:**
- `libwebkit2gtk-4.1-dev` - WebView rendering
- `libappindicator3-dev` - System tray support
- `librsvg2-dev` - SVG icon rendering
- `patchelf` - Binary patching for AppImage
- `libasound2-dev` - Audio support (for Piper TTS)

**Build Outputs:**
- AppImage (universal, no installation)
- DEB (Debian/Ubuntu)
- RPM (Fedora/RHEL)

#### macOS (macos-latest)
**Cross-Compilation:**
- Intel builds on Intel or Apple Silicon runners
- Apple Silicon builds require `aarch64-apple-darwin` target

**Code Signing:**
- Unsigned builds work but show security warnings
- Production releases need Apple Developer certificates
- Notarization required for macOS 10.15+

**Build Outputs:**
- DMG (disk image)
- `.app` bundle (inside DMG)

#### Windows (windows-latest)
**WebView:**
- WebView2 pre-installed on GitHub Actions runners
- No additional dependencies needed

**Code Signing:**
- Unsigned NSIS installers trigger SmartScreen warnings
- Production releases need code signing certificate

**Build Outputs:**
- NSIS installer (`.exe`) - **Official format**
- MSI installer (`.msi`) - Optional, requires WiX

### Troubleshooting

#### CI Fails: "Git LFS quota exceeded"

**Symptom:**
```
Error: failed to fetch some objects from 'https://github.com/...'
```

**Solution:**
- Upgrade GitHub account for more LFS bandwidth
- Or, implement LFS caching in workflow
- Or, temporarily disable builds for non-code changes

#### CI Fails: Rust compilation errors

**Common Causes:**
1. Dependency version conflicts
2. Platform-specific code issues
3. Missing system libraries

**Solution:**
```bash
# Test locally first
cargo clippy --all-targets --all-features
cargo test --verbose
```

#### CI Fails: Frontend build errors

**Common Causes:**
1. Node version mismatch
2. Missing dependencies
3. TypeScript errors

**Solution:**
```bash
# Test locally
pnpm install
pnpm build
```

#### Release Workflow: Draft not created

**Possible Causes:**
1. Invalid tag format (must be `vX.Y.Z`)
2. `GITHUB_TOKEN` permissions issue
3. tauri-action failure

**Solution:**
- Check tag format: `git tag -l`
- Verify workflow permissions in repo settings
- Check workflow logs for errors

### Best Practices

#### Before Pushing to Main

1. **Run local checks:**
   ```bash
   # Rust
   cd src-tauri
   cargo clippy --all-targets --all-features
   cargo test
   cargo fmt --all

   # Frontend
   cd ..
   pnpm build
   ```

2. **Test on your platform:**
   ```bash
   pnpm tauri build
   ```

3. **Verify Git LFS files:**
   ```bash
   git lfs ls-files
   ```

#### Creating a Release

1. **Update version in:**
   - `src-tauri/tauri.conf.json` → `version`
   - `src-tauri/Cargo.toml` → `version`
   - `package.json` → `version`

2. **Create and push tag:**
   ```bash
   git tag v0.1.0
   git push origin v0.1.0
   ```

3. **Monitor workflow:**
   - Go to Actions tab on GitHub
   - Wait for all builds to complete (~50-65 minutes)
   - Review draft release
   - Edit release notes if needed
   - Publish release

#### Dependabot PRs

1. **Review weekly:**
   - Check for security updates (prioritize)
   - Review breaking changes in major version updates
   - Test locally before merging

2. **Merge strategy:**
   - Group related updates
   - Merge non-breaking updates quickly
   - Test breaking updates thoroughly

## Implemented Optimizations

### 1. LFS Caching ✅

**Implemented:** Reduces LFS bandwidth by caching model files across workflow runs.

```yaml
- name: Cache Git LFS objects
  uses: actions/cache@v4
  with:
    path: .git/lfs
    key: lfs-${{ runner.os }}-${{ hashFiles('.gitattributes') }}
    restore-keys: |
      lfs-${{ runner.os }}-
```

**Impact:**
- First run: Downloads full 5.3 GB from LFS
- Subsequent runs: Only downloads changed files
- **Estimated savings:** 90-95% bandwidth reduction

### 2. Rust Build Caching ✅

**Implemented:** Caches Cargo dependencies and build artifacts for faster builds.

```yaml
- name: Cache Rust dependencies
  uses: actions/cache@v4
  with:
    path: |
      ~/.cargo/bin/
      ~/.cargo/registry/index/
      ~/.cargo/registry/cache/
      ~/.cargo/git/db/
      src-tauri/target/
    key: ${{ runner.os }}-cargo-${{ hashFiles('src-tauri/Cargo.lock') }}
    restore-keys: |
      ${{ runner.os }}-cargo-
```

**Impact:**
- First build: ~15-20 minutes
- Cached builds: ~5-8 minutes
- **Estimated savings:** 60-70% build time reduction

### 3. Skip Builds for Docs ✅

**Implemented:** CI only runs for actual code changes, not documentation updates.

```yaml
on:
  push:
    branches: [main]
    paths-ignore:
      - '**.md'
      - 'Documentation/**'
      - '.github/dependabot.yml'
      - 'LICENSE'
      - '.gitignore'
```

**Impact:**
- Documentation-only PRs: No CI runs
- **Estimated savings:** 30-40% fewer CI runs

### Combined Impact

With all optimizations:
- **LFS bandwidth:** ~95% reduction (5.3 GB → ~250 MB per run after first)
- **Build time:** ~65% reduction (50-65 min → 15-25 min after first)
- **CI runs:** ~35% reduction (docs don't trigger builds)
- **GitHub Actions minutes:** Significant savings on private repos

## Additional Optimization Opportunities

These can be implemented in the future if needed:

### Future Optimizations

#### 1. Parallel Testing

Split tests across multiple jobs to run in parallel:

```yaml
jobs:
  test-unit:
    # Run unit tests only
  test-integration:
    # Run integration tests only
  build:
    needs: [test-unit, test-integration]
    # Build only after tests pass
```

**Potential benefit:** Further reduce wall-clock time if tests take >5 minutes

#### 2. Conditional Platform Builds

Only build for specific platforms on certain triggers:

```yaml
strategy:
  matrix:
    platform: [ubuntu-latest]  # Only Linux for PRs
    include:
      - platform: macos-latest
        if: github.event_name == 'push'  # Full builds on push to main
      - platform: windows-latest
        if: github.event_name == 'push'
```

**Potential benefit:** Faster feedback on PRs, full validation on merge

## Monitoring and Alerts

### GitHub Actions Dashboard

Monitor workflow runs:
1. Go to repository → Actions tab
2. View recent runs, durations, and failures
3. Download artifacts for inspection

### Notifications

Configure notifications:
1. Repository → Settings → Notifications
2. Enable email alerts for workflow failures
3. Configure Slack/Discord webhooks (optional)

## Security Considerations

### Secrets Management

**Required Secrets (for code signing):**
- `APPLE_CERTIFICATE` - macOS code signing certificate
- `APPLE_CERTIFICATE_PASSWORD` - Certificate password
- `APPLE_SIGNING_IDENTITY` - Signing identity
- `APPLE_ID` - Apple ID for notarization
- `APPLE_PASSWORD` - App-specific password
- `TAURI_PRIVATE_KEY` - Tauri updater signing key
- `TAURI_KEY_PASSWORD` - Key password

**To add secrets:**
1. Repository → Settings → Secrets and variables → Actions
2. Click "New repository secret"
3. Add secret name and value

### Permissions

**Workflow Permissions:**
- `GITHUB_TOKEN` has read/write access by default
- Used for creating releases and uploading artifacts
- Scoped to repository only

## Future Enhancements

### Planned Improvements

1. **Automated testing:**
   - E2E tests with Playwright/Tauri
   - UI component tests
   - Integration tests for voice pipeline

2. **Performance monitoring:**
   - Build time tracking
   - Bundle size analysis
   - Dependency audit

3. **Release automation:**
   - Automatic changelog generation
   - Version bump automation
   - Release notes from PR descriptions

4. **Platform-specific optimizations:**
   - macOS universal binaries (Intel + Apple Silicon)
   - Windows ARM64 support
   - Linux ARM64 builds

## Support

For issues with CI/CD workflows:
1. Check this documentation
2. Review workflow logs in Actions tab
3. Consult [Tauri CI/CD docs](https://v2.tauri.app/distribute/ci-cd/)
4. Open issue with `ci-cd` label

---

**Last Updated:** October 2025
**Workflow Version:** 1.0
**Tauri Version:** 2.x
