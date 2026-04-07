# update-deps.ps1: local replacement for the heavy parts of Dependabot.
#
# Bumps Cargo and npm dependencies inside the version constraints from
# the manifests, runs every test suite, and prints a diff of the lock
# files plus an outdated report so you can decide on major bumps.
#
# Major version bumps still need a manual edit of the manifest. Run
# `cargo upgrade` (cargo-edit) or `npx npm-check-updates -u` for that.
#
# Usage:  pwsh scripts/update-deps.ps1
#
$ErrorActionPreference = "Stop"
$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
Set-Location $repoRoot

function Bold([string]$msg) { Write-Host $msg -ForegroundColor White }
function Ok([string]$msg)   { Write-Host $msg -ForegroundColor Green }
function Warn([string]$msg) { Write-Host $msg -ForegroundColor Yellow }
function Err([string]$msg)  { Write-Host $msg -ForegroundColor Red }

$cargoCrates = @(
  "libs/i18n",
  "callback-popup",
  "tel-handler",
  "desktop-client/src-tauri"
)

Bold "==> Step 1/4: cargo update (within Cargo.toml constraints)"
foreach ($crate in $cargoCrates) {
  Write-Host "  - $crate"
  cargo update --manifest-path "$crate/Cargo.toml" --quiet
}
Ok "    cargo lockfiles refreshed"

Bold "==> Step 2/4: npm update (within package.json constraints)"
Push-Location desktop-client
npm update --silent
Pop-Location
Ok "    npm lockfile refreshed"

Bold "==> Step 3/4: tests"
# Tauri's build script needs the sidecar files to exist, even for
# `cargo test --lib`. Empty placeholders are enough.
$binDir = "desktop-client/src-tauri/binaries"
New-Item -ItemType Directory -Force -Path $binDir | Out-Null
New-Item -ItemType File -Force -Path "$binDir/callback-popup-x86_64-pc-windows-msvc.exe" | Out-Null
New-Item -ItemType File -Force -Path "$binDir/tel-handler-x86_64-pc-windows-msvc.exe" | Out-Null

$failed = $false
foreach ($crate in @("libs/i18n", "callback-popup", "tel-handler")) {
  Write-Host "  - cargo test ($crate)"
  cargo test --manifest-path "$crate/Cargo.toml" --quiet
  if ($LASTEXITCODE -ne 0) { $failed = $true }
}
Write-Host "  - cargo test --lib (desktop-client/src-tauri)"
cargo test --manifest-path desktop-client/src-tauri/Cargo.toml --lib --quiet
if ($LASTEXITCODE -ne 0) { $failed = $true }

Write-Host "  - npm test (desktop-client)"
Push-Location desktop-client
npm test --silent
if ($LASTEXITCODE -ne 0) { $failed = $true }
Pop-Location

Write-Host "  - cargo fmt --check"
foreach ($crate in $cargoCrates) {
  cargo fmt --manifest-path "$crate/Cargo.toml" --check
  if ($LASTEXITCODE -ne 0) { $failed = $true }
}

if ($failed) {
  Err "    one or more checks failed - review the output above"
  exit 1
}
Ok "    all tests + fmt green"

Bold "==> Step 4/4: outdated report (informational)"
Write-Host "  Cargo:"
$cargoOutdated = Get-Command cargo-outdated -ErrorAction SilentlyContinue
if ($cargoOutdated) {
  foreach ($crate in $cargoCrates) {
    $output = cargo outdated --manifest-path "$crate/Cargo.toml" --root-deps-only 2>$null
    $output | ForEach-Object { Write-Host "    [$crate] $_" }
  }
} else {
  Warn "    cargo-outdated not installed (install: cargo install cargo-outdated)"
}
Write-Host "  npm:"
Push-Location desktop-client
npm outdated
Pop-Location

Write-Host ""
Bold "==> Done"
Write-Host ""
$diff = git diff --name-only
if ($diff) {
  Write-Host "Lockfile / manifest changes ready to commit:"
  $diff | ForEach-Object { Write-Host "  $_" }
  Write-Host ""
  Write-Host "Suggested next step:"
  Write-Host "  git add -A; git commit -m 'chore(deps): bump dependencies'; git push"
} else {
  Write-Host "Nothing changed - your dependencies were already up to date."
}
