# cargo-goosectl GitHub Action

A GitHub Action that installs and runs [`cargo-goosectl`](https://github.com/mozilla-ai/cargo-goosectl), a semver-aware release helper for Rust workspaces.

This action is a thin wrapper around the `cargo-goosectl` CLI. It installs the tool and optionally runs it with provided arguments.

## Usage

### Install `cargo-goosectl` only

If you just want `cargo-goosectl` available in your workflow:

```yaml
- uses: mozilla-ai/cargo-goosectl/actions/cargo-goosectl@v1
```

After this step, `cargo goosectl` is available for the rest of the job.

### Install and run `cargo-goosectl`

You can also pass arguments to run `cargo-goosectl` directly:

```yaml
- uses: mozilla-ai/cargo-goosectl/actions/cargo-goosectl@v1
  with:
    args: bump patch
```

This is equivalent to running:

```bash
cargo goosectl bump patch
```

## Inputs

| Name   | Description                       | Required | Default |
| ------ | --------------------------------- | -------- | ------- |
| `args` | Arguments passed to `cargo goosectl` | No       | `""`    |

If `args` is not provided, the action will only install `cargo-goosectl` and do nothing else.

## Examples

### Bump the patch version in CI

```yaml
- uses: actions/checkout@v4

- uses: mozilla-ai/cargo-goosectl/actions/cargo-goosectl@v1
  with:
    args: bump patch
```

### Install once, run multiple commands

```yaml
- uses: mozilla-ai/cargo-goosectl/actions/cargo-goosectl@v1

- run: cargo goosectl current-version
- run: cargo goosectl bump minor
```

## Notes

* This action installs Rust using the stable toolchain.
* `cargo-goosectl` is installed via `cargo install`.
* The installation is scoped to the current job.
* Calling this action multiple times will reinstall `cargo-goosectl` unless cached by the workflow.

## Versioning

This action should be referenced using a major version tag:

```yaml
uses: mozilla-ai/cargo-goosectl/actions/cargo-goosectl@v1
```

This allows bugfixes and improvements without breaking workflows.

## License

This action is licensed under the same terms as the `cargo-goosectl` project.
