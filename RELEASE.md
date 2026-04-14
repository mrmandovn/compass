# Release Process

Compass releases are tag-driven: pushing a `v<X.Y.Z>` tag triggers `.github/workflows/release.yml` which builds 4 platform binaries in parallel and publishes 5 npm packages (4 platform + 1 meta).

## Prerequisites

- npm scope `@compass-m` exists on npmjs.com (free public org).
- GitHub repository secret `NPM_TOKEN` set to an automation token with publish permissions on `compass-m` + `@compass-m/cli-*` packages.

## Cutting a release

1. **Bump versions in 5 sources** (must all match):
   - `VERSION`
   - `package.json` `.version`
   - `cli/Cargo.toml` `[package] version`
   - `core/manifest.json` `.version`
   - `core/colleagues/manifest.json` `.version`

   Also update the version pin in `cli/src/cmd/version.rs`'s `all_sources_match` test.

   Then bump each `npm/cli-{platform}/package.json` `.version` to match.

2. **Bump optionalDependencies in `package.json`** to the new version (4 entries).

3. **Run `cd cli && cargo test`** — all unit + integration tests must pass.

4. **Commit + tag + push:**
   ```bash
   git add -A
   git commit -m "release: v<X.Y.Z>"
   git tag v<X.Y.Z>
   git push origin main
   git push origin v<X.Y.Z>
   ```

5. **Monitor CI:** `gh run watch` or check the Actions tab. Build matrix takes ~3-5min, publish ~1min.

6. **Verify:** `npm view compass-m version` and `npm view @compass-m/cli-darwin-arm64 version` should both show the new version.

## CI workflow overview

`.github/workflows/release.yml` triggers on `push: tags: ['v*']`:

- **build** (matrix, 4 parallel jobs):
  - `aarch64-apple-darwin` → macos-14
  - `x86_64-apple-darwin` → macos-13
  - `x86_64-unknown-linux-gnu` → ubuntu-22.04
  - `aarch64-unknown-linux-gnu` → ubuntu-22.04-arm

  Each builds `compass-cli` for its target, copies into `npm/cli-{platform}/compass-cli`, uploads as artifact.

- **publish** (depends on build):
  - Downloads all 4 platform artifacts.
  - Publishes 4 platform packages first (`@compass-m/cli-{platform}`).
  - Publishes meta `compass-m` package last.

If any platform build fails, meta is NOT published — preventing partial releases.

## Local testing before release

```bash
cd cli && cargo build --release && cargo test
node bin/install --help     # smoke check Node script
```

## Hotfixes

For patch releases, follow the same process. Bump only PATCH (1.0.0 → 1.0.1).

## Rollback

If a release ships broken:
1. Deprecate the bad version: `npm deprecate compass-m@<X.Y.Z> "broken; use <prev>"` plus same for each `@compass-m/cli-*@<X.Y.Z>`.
2. Cut a new patch release with the fix.
3. Avoid `npm unpublish` after 72h (npm forbids it).
