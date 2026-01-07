# cargo-goose 🪿

<p align="center">
A strict, explicit SemVer CLI with first-class prerelease support.
</p>

<p align="center">
  <a href="https://github.com/mozilla-ai/cargo-goose/actions/workflows/ci.yml">
    <img src="https://github.com/mozilla-ai/cargo-goose/actions/workflows/ci.yml/badge.svg" alt="CI">
  </a>
  <a href="https://github.com/mozilla-ai/cargo-goose/actions/workflows/release.yml">
    <img src="https://github.com/mozilla-ai/cargo-goose/actions/workflows/release.yml/badge.svg" alt="Lint">
  </a>
  <a href="https://codecov.io/gh/mozilla-ai/cargo-goose" > 
    <img src="https://codecov.io/gh/mozilla-ai/cargo-goose/graph/badge.svg?token=RYYJ2XMWAR"/> 
  </a>
</p>


## Installation

```sh
cargo install cargo-goose
```

## Usage

```sh
cargo goose bump <command>
```

## Commands

### Bump a release version

Bump the current version by level:

```sh
cargo goose bump version patch
# 1.2.3 → 1.2.4
cargo goose bump version minor
# 1.2.3 → 1.3.0
cargo goose bump version major
# 1.2.3 → 2.0.0
```

Start a prerelease on the new version line:

```sh
cargo goose bump version minor rc
# 1.2.3 → 1.3.0-rc.1
```

### Prerelease management

Increment the current prerelease counter:

```sh
cargo goose bump prerelease
# alpha.1 → alpha.2
```

Transition to a new prerelease identifier:

```sh
cargo goose bump prerelease beta
# 1.2.3-alpha.3 → 1.2.3-beta.1
```

### Finalize a release

Finalize a prerelease into a stable release:

```sh
cargo goose bump release
# 1.2.0-rc.2 → 1.2.0
```

### Build metadata

All commands accept optional build metadata:

```sh
cargo goose bump version patch --metadata git.abc123
```

### Dry run

Don't want to screw up your Cargo.toml just yet? Add the `--dry-run` flag to see what cargo-goose will do without modifying any files:

```sh
cargo goose --dry-run bump ...
```

## Prerelease format

Prereleases must use the following format:

```
<identifier>.<counter>
```

Examples:

* `alpha.1`
* `beta.2`
* `rc.3`

Invalid prerelease formats are rejected.
