#!/usr/bin/env bash
#
# update-deps.sh: local replacement for the heavy parts of Dependabot.
#
# Bumps Cargo and npm dependencies inside the version constraints from
# the manifests, runs every test suite, and prints a diff of the lock
# files plus an outdated report so you can decide on major bumps.
#
# Major version bumps still need a manual edit of the manifest. Run
# `cargo upgrade` (cargo-edit) or `npx npm-check-updates -u` for that.
#
# Usage:  bash scripts/update-deps.sh
#         (or just `./scripts/update-deps.sh` after `chmod +x`)
#
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$repo_root"

bold() { printf '\033[1m%s\033[0m\n' "$*"; }
ok()   { printf '\033[32m%s\033[0m\n' "$*"; }
warn() { printf '\033[33m%s\033[0m\n' "$*"; }
err()  { printf '\033[31m%s\033[0m\n' "$*"; }

cargo_crates=(
  "libs/i18n"
  "callback-popup"
  "tel-handler"
  "desktop-client/src-tauri"
)

bold "==> Step 1/4: cargo update (within Cargo.toml constraints)"
for crate in "${cargo_crates[@]}"; do
  echo "  • $crate"
  cargo update --manifest-path "$crate/Cargo.toml" --quiet
done
ok "    cargo lockfiles refreshed"

bold "==> Step 2/4: npm update (within package.json constraints)"
(cd desktop-client && npm update --silent)
ok "    npm lockfile refreshed"

bold "==> Step 3/4: tests"
# Tauri's build script needs the sidecar files to exist, even for
# `cargo test --lib`. Empty placeholders are enough.
mkdir -p desktop-client/src-tauri/binaries
: > desktop-client/src-tauri/binaries/callback-popup-x86_64-pc-windows-msvc.exe
: > desktop-client/src-tauri/binaries/tel-handler-x86_64-pc-windows-msvc.exe

failed=0
for crate in libs/i18n callback-popup tel-handler; do
  echo "  • cargo test ($crate)"
  cargo test --manifest-path "$crate/Cargo.toml" --quiet || failed=1
done
echo "  • cargo test --lib (desktop-client/src-tauri)"
cargo test --manifest-path desktop-client/src-tauri/Cargo.toml --lib --quiet || failed=1

echo "  • npm test (desktop-client)"
(cd desktop-client && npm test --silent) || failed=1

echo "  • cargo fmt --check"
for crate in "${cargo_crates[@]}"; do
  cargo fmt --manifest-path "$crate/Cargo.toml" --check || failed=1
done

if [ $failed -ne 0 ]; then
  err "    one or more checks failed, review the output above"
  exit 1
fi
ok "    all tests + fmt green"

bold "==> Step 4/4: outdated report (informational)"
echo "  Cargo:"
if command -v cargo-outdated >/dev/null 2>&1; then
  for crate in "${cargo_crates[@]}"; do
    cargo outdated --manifest-path "$crate/Cargo.toml" --root-deps-only 2>/dev/null \
      | sed "s|^|    [$crate] |" || true
  done
else
  warn "    cargo-outdated not installed (install: cargo install cargo-outdated)"
fi
echo "  npm:"
(cd desktop-client && npm outdated || true)

echo
bold "==> Done"
echo
diff_files=$(git diff --name-only 2>/dev/null || true)
if [ -n "$diff_files" ]; then
  echo "Lockfile / manifest changes ready to commit:"
  echo "$diff_files" | sed 's/^/  /'
  echo
  echo "Suggested next step:"
  echo "  git add -A && git commit -m 'chore(deps): bump dependencies' && git push"
else
  echo "Nothing changed. Your dependencies were already up to date."
fi
