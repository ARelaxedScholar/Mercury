# Contributing to Orichalcum

## Development environment

Orichalcum pins its Rust development environment through Nix:

```bash
nix develop
```

Run the complete Rust release gate from the repository root:

```bash
nix develop -c ./scripts/verify.sh
```

The gate checks formatting, default and all-feature targets across the workspace,
procedural-macro compile-pass/fail fixtures, doctests, strict Clippy, and package file
sets. It fully assembles and verifies the lowest-level `orichalcum-definition` crate.
Cargo cannot assemble a dependent package until its exact versioned dependency exists
on crates.io, so the workspace build verifies the unpublished dependency chain. Tests
and package checks use `Cargo.lock` and fail rather than silently updating it.

## Release procedure

Release all three crates at the same version and in dependency order:

1. `orichalcum-definition`
2. `orichalcum-macros`
3. `orichalcum`

Prepare a release from a clean, committed tree. Do not use `--allow-dirty` for an actual
upload. Confirm that `CHANGELOG.md` has the intended version and release date, then run:

```bash
nix develop -c ./scripts/verify.sh
nix flake check
nix build
git status --short
```

The final command must produce no output. Publish and verify each dependency before
moving to its consumer:

```bash
cargo publish --locked --dry-run -p orichalcum-definition
cargo publish --locked -p orichalcum-definition

# After crates.io has indexed orichalcum-definition:
cargo publish --locked --dry-run -p orichalcum-macros
cargo publish --locked -p orichalcum-macros

# After crates.io has indexed orichalcum-macros:
cargo publish --locked --dry-run -p orichalcum
cargo publish --locked -p orichalcum
```

Create and push the release tag from the exact published commit:

```bash
git tag -a v0.5.0 -m "Release 0.5.0"
git push origin HEAD
git push origin v0.5.0
```

Never paste or commit the crates.io API token into the repository or command history.

Verify the Nix outputs separately:

```bash
nix flake check
nix build
```

Nix flakes only include tracked files. A newly added but untracked source or example
will therefore be absent from `nix build` until it is added to Git.

## Dependency updates

`Cargo.lock` and `Cargo.nix` are one reproducibility unit. Whenever dependency
resolution changes, regenerate the Nix snapshot with the cargo2nix version pinned by
the flake:

```bash
nix run github:cargo2nix/cargo2nix/release-0.12 -- -o -l .
```

The `-l` flag requires the existing lockfile to be used, and `-o` replaces the previous
generated snapshot without an interactive prompt. Commit `Cargo.lock` and `Cargo.nix`
together.
