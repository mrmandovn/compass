#!/usr/bin/env bash
#
# bump-version.sh — sync all 11 version references across the repo.
#
# Usage:
#   ./scripts/bump-version.sh 1.0.18       # bump to 1.0.18
#   ./scripts/bump-version.sh --dry-run 1.0.18   # preview, don't write
#   ./scripts/bump-version.sh --check      # verify current sync state
#
# Files updated (11 references):
#   - VERSION                                  (source of truth)
#   - package.json                             (.version + 4× .optionalDependencies)
#   - core/manifest.json                       (.version)
#   - core/colleagues/manifest.json            (.version)
#   - cli/Cargo.toml                           (version = "...")
#   - cli/src/cmd/version.rs                   (pin literal in all_sources_match test)
#   - npm/cli-darwin-arm64/package.json        (.version)
#   - npm/cli-darwin-x64/package.json          (.version)
#   - npm/cli-linux-arm64/package.json         (.version)
#   - npm/cli-linux-x64/package.json           (.version)
#
# After running, review `git diff`, then commit + tag:
#   git add -A && git commit -m "chore: bump to $NEW — sync all platform packages"
#   git tag v$NEW
#   git push origin main v$NEW

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

# ───── Portable in-place sed (GNU vs BSD) ─────
if sed --version >/dev/null 2>&1; then
  SED_INPLACE=(-i)    # GNU sed
else
  SED_INPLACE=(-i '')  # BSD sed (macOS)
fi

# ───── Parse args ─────
DRY_RUN=false
CHECK_ONLY=false
NEW=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --dry-run) DRY_RUN=true; shift ;;
    --check) CHECK_ONLY=true; shift ;;
    -h|--help)
      sed -n '2,/^$/p' "$0" | sed 's/^# \?//'
      exit 0
      ;;
    -*) echo "Unknown flag: $1"; exit 1 ;;
    *)  NEW="$1"; shift ;;
  esac
done

# ───── Read current state ─────
OLD=$(cat VERSION | tr -d '[:space:]')

# ───── Helper: read each ref ─────
read_refs() {
  local v_file v_pkg v_pkg_dep v_manifest v_coll_manifest v_cargo v_pin v_plat
  v_file=$(cat VERSION | tr -d '[:space:]')
  v_pkg=$(jq -r '.version' package.json)
  v_pkg_dep=$(jq -r '.optionalDependencies["@compass-m/cli-darwin-arm64"] // ""' package.json)
  v_manifest=$(jq -r '.version' core/manifest.json)
  v_coll_manifest=$(jq -r '.version' core/colleagues/manifest.json)
  v_cargo=$(grep -E '^version = ' cli/Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
  v_pin=$(grep -oE 'version_txt, "[0-9]+\.[0-9]+\.[0-9]+"' cli/src/cmd/version.rs | head -1 | sed 's/.*"\(.*\)"/\1/')

  echo "  VERSION                                  $v_file"
  echo "  package.json .version                    $v_pkg"
  echo "  package.json .optionalDependencies.*     $v_pkg_dep"
  echo "  core/manifest.json .version              $v_manifest"
  echo "  core/colleagues/manifest.json .version   $v_coll_manifest"
  echo "  cli/Cargo.toml version                   $v_cargo"
  echo "  cli/src/cmd/version.rs pin               $v_pin"

  for p in darwin-arm64 darwin-x64 linux-arm64 linux-x64; do
    v_plat=$(jq -r '.version' "npm/cli-$p/package.json")
    echo "  npm/cli-$p/package.json         $v_plat"
  done
}

# ───── --check mode: verify sync ─────
if $CHECK_ONLY; then
  echo "Version reference check (target: $OLD)"
  echo ""
  read_refs
  echo ""

  EXPECTED="$OLD"
  FAILED=0

  check() {
    local label="$1" actual="$2"
    if [[ "$actual" != "$EXPECTED" ]]; then
      echo "  ✗ $label = $actual (expected $EXPECTED)"
      FAILED=$((FAILED+1))
    fi
  }

  check "package.json"              "$(jq -r '.version' package.json)"
  check "optionalDep darwin-arm64"  "$(jq -r '.optionalDependencies["@compass-m/cli-darwin-arm64"]' package.json)"
  check "optionalDep darwin-x64"    "$(jq -r '.optionalDependencies["@compass-m/cli-darwin-x64"]' package.json)"
  check "optionalDep linux-arm64"   "$(jq -r '.optionalDependencies["@compass-m/cli-linux-arm64"]' package.json)"
  check "optionalDep linux-x64"     "$(jq -r '.optionalDependencies["@compass-m/cli-linux-x64"]' package.json)"
  check "core/manifest.json"        "$(jq -r '.version' core/manifest.json)"
  check "core/colleagues/manifest.json" "$(jq -r '.version' core/colleagues/manifest.json)"
  check "cli/Cargo.toml"            "$(grep -E '^version = ' cli/Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')"
  check "cli/src/cmd/version.rs pin" "$(grep -oE 'version_txt, "[0-9]+\.[0-9]+\.[0-9]+"' cli/src/cmd/version.rs | head -1 | sed 's/.*"\(.*\)"/\1/')"

  for p in darwin-arm64 darwin-x64 linux-arm64 linux-x64; do
    check "npm/cli-$p"               "$(jq -r '.version' "npm/cli-$p/package.json")"
  done

  if [[ "$FAILED" -eq 0 ]]; then
    echo ""
    echo "✓ All 11 version references are in sync at $EXPECTED"
    exit 0
  else
    echo ""
    echo "✗ $FAILED reference(s) out of sync — run ./scripts/bump-version.sh $EXPECTED to fix."
    exit 1
  fi
fi

# ───── Bump mode: need NEW version ─────
if [[ -z "$NEW" ]]; then
  echo "Usage: $0 <new-version> [--dry-run]"
  echo "       $0 --check"
  echo ""
  echo "Current version: $OLD"
  exit 1
fi

# ───── Validate NEW format ─────
if ! [[ "$NEW" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.]+)?$ ]]; then
  echo "Error: '$NEW' is not a valid semver (expected x.y.z or x.y.z-prerelease)"
  exit 1
fi

if [[ "$NEW" == "$OLD" ]]; then
  echo "Already at $OLD — no change needed."
  exit 0
fi

echo "Bumping $OLD → $NEW"
$DRY_RUN && echo "  (dry-run — no files will be written)"
echo ""

# ───── Apply bumps ─────
apply() {
  local file="$1" pattern="$2" replacement="$3"
  if $DRY_RUN; then
    if grep -qE "$pattern" "$file"; then
      echo "  would update $file"
    else
      echo "  NO MATCH in $file for pattern: $pattern" >&2
      return 1
    fi
  else
    sed "${SED_INPLACE[@]}" -E "s/$pattern/$replacement/g" "$file"
    echo "  ✓ $file"
  fi
}

# Root files
if $DRY_RUN; then
  echo "  would write $NEW to VERSION"
else
  echo "$NEW" > VERSION
  echo "  ✓ VERSION"
fi

apply package.json                    "\"version\": \"$OLD\""  "\"version\": \"$NEW\""
apply package.json                    "\"$OLD\""               "\"$NEW\""     # catches the 4 optionalDeps
apply core/manifest.json              "\"version\": \"$OLD\""  "\"version\": \"$NEW\""
apply core/colleagues/manifest.json   "\"version\": \"$OLD\""  "\"version\": \"$NEW\""
apply cli/Cargo.toml                  "version = \"$OLD\""     "version = \"$NEW\""
apply cli/src/cmd/version.rs          "version_txt, \"$OLD\""  "version_txt, \"$NEW\""

for p in darwin-arm64 darwin-x64 linux-arm64 linux-x64; do
  apply "npm/cli-$p/package.json" "\"version\": \"$OLD\"" "\"version\": \"$NEW\""
done

echo ""

if $DRY_RUN; then
  echo "Dry-run complete. Remove --dry-run to apply."
else
  # Verify after write
  if "$0" --check >/dev/null 2>&1; then
    echo "✓ All 11 version references updated to $NEW"
    echo ""
    echo "Next:"
    echo "  git diff            # review"
    echo "  git add -A && git commit -m \"chore: bump to $NEW — sync all platform packages\""
    echo "  git tag v$NEW"
    echo "  git push origin main v$NEW"
  else
    echo "⚠ Post-bump check failed. Run ./scripts/bump-version.sh --check for details."
    exit 1
  fi
fi
