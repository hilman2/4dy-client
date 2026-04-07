# Contributing

Thanks for taking a look. This is a small Windows-only Tauri/Rust project. The goal here is to keep it lean and easy to maintain, not to grow a framework.

## Ground rules

- Keep changes focused. One bug or one feature per PR.
- Don't refactor code you're not actually touching.
- Don't add dependencies unless you really need them. The whole point of this project is to be lighter than the official client.
- Windows-only is on purpose. Don't add cross-platform shims for things you can't actually test.

## Local setup

You'll need:

- Rust stable (via [rustup](https://rustup.rs/))
- Node.js 20+
- Visual Studio Build Tools with the "Desktop development with C++" workload
- WebView2 (already on Windows 10/11)

Build instructions live in [`README.md`](README.md#building-from-source). The TL;DR is: build the two sidecar binaries first, copy them into `desktop-client/src-tauri/binaries/` with the platform suffix, then `npx tauri build`.

## Running tests

```bash
# Rust
cargo test --manifest-path libs/i18n/Cargo.toml
cargo test --manifest-path callback-popup/Cargo.toml
cargo test --manifest-path tel-handler/Cargo.toml
cargo test --manifest-path desktop-client/src-tauri/Cargo.toml --lib

# Frontend
cd desktop-client && npm install && npm test
```

CI runs all of the above, plus `cargo fmt --check`. Run them locally before pushing to avoid the round-trip.

## Updating dependencies

Dependency bumps are handled locally, not through Dependabot, to avoid a pile of cascading PRs. Run:

```bash
bash scripts/update-deps.sh    # or pwsh scripts/update-deps.ps1 on Windows
```

This refreshes both `Cargo.lock` files and `desktop-client/package-lock.json` within the version constraints in the manifests, runs every test suite, and prints an outdated report so you can decide on major bumps. If everything is green it tells you what to commit.

For major version bumps (anything beyond what `cargo update` / `npm update` will do on their own) edit the manifest manually, or use `cargo upgrade` (from `cargo-edit`) and `npx npm-check-updates -u`. The only thing Dependabot still watches is GitHub Actions, on a monthly schedule.

## Code style

- Rust: `cargo fmt` (default rustfmt config) and clippy must pass with `-D warnings`.
- TypeScript/Svelte: keep it simple, no enforced linter beyond what Vite/Svelte do out of the box.
- Comments in code are in English. Existing files have a few German comments, that's fine, leave them, but new comments should be English.

## Pull requests

- Open against `main`.
- Describe *why* the change is needed, not just *what* it does.
- If you're touching pure logic (phone normalisation, config parsing, i18n, validators), add or update the relevant unit tests.
- If you're touching the web-client integration layer (`desktop-client/src/inject/`), please mention which 3CX web client version you tested against.

## Reporting bugs

Open an issue with:

- Windows version
- 3CX web client version (the upstream system)
- Steps to reproduce
- Expected vs actual behaviour
- Anything from the desktop client log if relevant

That's it. Thanks!
