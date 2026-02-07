# current-version GitHub Action

A GitHub Action that extracts the current Cargo workspace version using `cargo-goosectl` and exposes it as structured outputs.

This action is designed for CI workflows that need access to version components such as `major`, `minor`, `patch`, prerelease identifiers, or the full version string.

Internally, this action runs:

```bash
cargo goosectl current-version --format=json
```

and adapts the JSON output into GitHub Action outputs.

## Usage

Basic usage, asserting that all packages in the workspace share the same version:

```yaml
- id: version
  uses: mozilla-ai/cargo-goosectl/actions/current-version@v1
  with:
    force-single-version: true

- run: echo "Version is ${{ steps.version.outputs.version }}"
```

## Inputs

| Name                   | Description                                              | Required | Default |
| ---------------------- | -------------------------------------------------------- | -------- | ------- |
| `force-single-version` | Assert that all selected packages share the same version | No       | `false` |

If `force-single-version` is set to `true` and multiple packages have different versions, the action will fail.

## Outputs

| Name            | Description                                                 |
| --------------- | ----------------------------------------------------------- |
| `version`       | Full version string (e.g. `1.2.3-beta.4`)                   |
| `major`         | Major version number                                        |
| `minor`         | Minor version number                                        |
| `patch`         | Patch version number                                        |
| `pre`           | Prerelease identifier (e.g. `alpha`, `beta`), empty if none |
| `iteration`     | Prerelease iteration number, empty if none                  |
| `build`         | Build metadata, empty if none                               |
| `is_prerelease` | `true` if this is a prerelease                              |

All outputs are strings, as required by GitHub Actions.

## Examples

Using version information in conditional logic:

```yaml
- id: version
  uses: mozilla-ai/cargo-goosectl/actions/current-version@v1
  with:
    force-single-version: true

- run: |
    if [ "${{ steps.version.outputs.is_prerelease }}" = "true" ]; then
      echo "This is a prerelease build"
    fi
```

Tagging a release with the current version:

```yaml
- id: version
  uses: mozilla-ai/cargo-goosectl/actions/current-version@v1
  with:
    force-single-version: true

- run: git tag "v${{ steps.version.outputs.version }}"
```

## Notes

* This action installs Rust using the stable toolchain.
* `cargo-goosectl` is installed via `cargo install`.
* Version parsing logic lives entirely in `cargo-goosectl`; this action does not reimplement semver logic.
* For workspaces with multiple packages, `force-single-version` should be enabled to avoid ambiguity.

## Versioning

This action should be referenced using a major version tag:

```yaml
uses: mozilla-ai/cargo-goosectl/actions/current-version@v1
```

This allows non-breaking improvements without disrupting existing workflows.

## License

This action is licensed under the same terms as the `cargo-goosectl` project.
