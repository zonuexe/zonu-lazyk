---
name: zonu-lazyk-release-prep
description: Prepare a zonu-lazyk release - bump the crate version, seal the changelog, reconcile the README, verify locally, and tag so GitHub Actions publishes to crates.io and attaches pre-built binaries. Use when the user asks to prepare the next version, cut a release, or make versioned files consistent before tagging.
metadata:
  internal: true
---

# zonu-lazyk Release Prep

Follow this workflow to release a new `zonu-lazyk` version. One `vX.Y.Z` tag
ships two ways: the crate on [crates.io](https://crates.io/crates/zonu-lazyk)
(`cargo install zonu-lazyk`) and per-platform pre-built binaries on the GitHub
Release.

The tag triggers [`release.yml`](../../../.github/workflows/release.yml), which
**first compiles every shipped target** (`build-check`), then — only if all
compiled — runs `cargo publish`, creates the GitHub Release from `CHANGELOG.md`,
and builds + uploads binaries. **Publishing is irreversible** (crates.io versions
can only be yanked, never replaced), so get a human Go before pushing the tag.

Prepare the release **on a `release/x.y.z` branch and open a PR**, so CI gates
the exact release delta before it touches `master`. **Squash-merge** the green
PR, then tag the merged commit.

At a glance: branch `release/x.y.z` -> bump version + seal changelog + reconcile
README -> verify locally -> push branch + open PR -> **CI green** -> **human Go**
-> squash-merge -> tag the merged commit + push -> Actions publishes.

## One-time setup (do once, before the first release)

- Reserve the crate name on crates.io (the first `cargo publish` from the
  release does this; the account owner must hold or be free to take
  `zonu-lazyk`).
- Store a crates.io API token as the repository secret
  `CARGO_REGISTRY_TOKEN` (GitHub -> Settings -> Secrets and variables -> Actions).
- The binary job uses the built-in `GITHUB_TOKEN`; no extra secret needed.

## Update release metadata

Decide the next semantic version, then update all versioned files together:

- `Cargo.toml` — the `version` field.
- `Cargo.lock` — bump zonu-lazyk's own entry (`cargo build` refreshes it). This
  is a binary crate that **tracks `Cargo.lock`**, so it must stay in sync.
- `CHANGELOG.md` — seal `[Unreleased]` into the new version section.

### Seal the `[Unreleased]` entries

The changelog is for humans; make it read like release notes, not commit messages.
`release.yml` extracts the version's section verbatim as the GitHub Release body.

1. If `[Unreleased]` is thin, reconstruct it from `git log <last-tag>..HEAD --oneline`.
2. Rewrite each bullet to one self-contained, user-facing sentence; drop
   internal-only detail (private refactors, test-only additions).
3. Add a `## [x.y.z] - YYYY-MM-DD` section below `## [Unreleased]`, using Keep a
   Changelog headings (`Added`, `Changed`, `Fixed`, ...).
4. **Do not hard-wrap entries** — each bullet is one physical line (wrapping
   degrades the GitHub Release body).
5. Update the bottom links: point `[Unreleased]` at `compare/vx.y.z...HEAD` and
   add `[x.y.z]: https://github.com/zonuexe/zonu-lazyk/releases/tag/vx.y.z`.

## Reconcile the README

`README.md` is the crates.io page. Before tagging, check it against the sealed
changelog and the real binary:

- **Usage/CLI** — the usage block matches `src/main.rs` (the `zonu-lazyk
  <program-file>` invocation and any flags it grows).
- **Status** — reflects what actually ships this release (no stale "not yet" for
  things now done).
- **Design/ADRs** — the ADR links resolve and describe what the code does.

## Verify the release

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo check --target x86_64-pc-windows-msvc   # Windows compiles (a binary ships)
cargo doc --no-deps
cargo publish --dry-run                        # packages as crates.io will
git diff --check
```

`cargo publish --dry-run` confirms a small file count: `Cargo.toml`'s `exclude`
drops `.github`, `.claude`, agent docs, `/tests`, and `/benches`. If the package
is unexpectedly large, fix `exclude` before publishing.

zonu-lazyk has **no runtime dependencies**, so there is no third-party license
bundle to regenerate. If a runtime dependency is ever added, add a license audit
(cargo-deny) and notices (cargo-about) to `ci.yml`, mirroring how other crates do.

## Open the release PR, get CI + the Go, then tag to publish

```sh
git switch -c release/x.y.z
# ... the bump / changelog / README edits ...
git commit -am "Bump up version to x.y.z"
git push -u origin release/x.y.z
gh pr create --fill                       # CI runs on the PR (incl. Windows cross-check)
gh pr checks --watch                      # wait for green
```

If CI fails, fix on the branch and push again — nothing has touched `master` or
crates.io. When the PR is green, **stop and get a human Go** (the tag is the
irreversible publish). Only after the Go, squash-merge and tag the merged commit:

```sh
gh pr merge --squash                      # one clean commit on master
git switch master && git pull --ff-only
grep '^version' Cargo.toml                # sanity: equals x.y.z (release.yml re-checks)
git tag vx.y.z
git push origin vx.y.z                    # runs release.yml: build-check -> publish -> binaries + Release
gh run watch                              # watch the publish
```

## Verify the outcome

```sh
cargo search zonu-lazyk | head -1                            # newest = x.y.z
gh release view vx.y.z --json assets --jq '.assets[].name'  # 4 binaries attached
```

## Manual fallback (if Actions is unavailable)

```sh
cargo login                    # paste a crates.io token, once
cargo publish
git tag vx.y.z && git push origin vx.y.z
gh release create vx.y.z --title vx.y.z \
  --notes "$(awk -v v=x.y.z '$0 ~ "^## \\["v"\\]"{p=1;next} p&&/^## \\[/{exit} p' CHANGELOG.md)"
```
