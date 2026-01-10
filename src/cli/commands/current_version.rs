use anyhow::Result;
use clap::{Args, ValueEnum};

use crate::{
    cli::global_args::GlobalArgs, utils::select_single_version,
    version::semantic_version::SemanticVersion,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Args)]
pub struct CurrentVersionArgs {
    #[arg(short = 'f', long = "format", help = "Output format")]
    format: Option<CurrentVersionOutput>,
    #[arg(long, help = "Assert all selected packages share the same version")]
    force_single_version: bool,
}

impl CurrentVersionArgs {
    pub fn execute(&self, metadata: &crate::metadata::Metadata, global: &GlobalArgs) -> Result<()> {
        let packages = metadata.select_packages(global.workspace, global.package.as_slice())?;

        let format = self
            .format
            .as_ref()
            .unwrap_or(&CurrentVersionOutput::Plaintext);

        if self.force_single_version {
            let version = select_single_version(packages.iter().map(|p| p.version.clone()))?;

            match format {
                CurrentVersionOutput::Plaintext => println!("{version}"),
                CurrentVersionOutput::Json => {
                    let repr = CurrentVersionRepr::try_from(version)?;
                    println!("{}", serde_json::to_string(&repr)?);
                }
            };

            return Ok(());
        }

        match format {
            CurrentVersionOutput::Plaintext => {
                // plaintext stays strict
                let version = select_single_version(packages.iter().map(|p| p.version.clone()))?;
                println!("{version}");
            }

            CurrentVersionOutput::Json => {
                let mut out = Vec::new();

                for pkg in &packages {
                    let version: SemanticVersion = pkg.version.clone().try_into()?;
                    out.push(PackageVersionRepr {
                        package: pkg.name.to_string(),
                        version: CurrentVersionRepr::try_from(version)?,
                    });
                }

                let json = serde_json::to_string(&CurrentVersionJson { packages: out })?;
                println!("{json}");
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, ValueEnum)]
pub enum CurrentVersionOutput {
    Plaintext,
    Json,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentVersionJson {
    packages: Vec<PackageVersionRepr>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageVersionRepr {
    package: String,

    #[serde(flatten)]
    version: CurrentVersionRepr,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentVersionRepr {
    version: String,
    major: u64,
    minor: u64,
    patch: u64,
    pre: Option<String>,
    iteration: Option<u64>,
    build: Option<String>,
    is_prerelease: bool,
}

impl TryFrom<SemanticVersion> for CurrentVersionRepr {
    type Error = anyhow::Error;

    fn try_from(val: SemanticVersion) -> Result<Self> {
        let pre = val.prerelease().clone();
        let (pre, iteration) = match pre {
            Some(p) => (Some(p.ident), Some(p.iteration)),
            None => (None, None),
        };

        Ok(CurrentVersionRepr {
            version: val.to_string(),
            major: val.major(),
            minor: val.minor(),
            patch: val.patch(),
            pre,
            iteration,
            build: val.build().clone(),
            is_prerelease: val.is_prerelease(),
        })
    }
}

#[cfg(test)]
mod force_single_tests {
    use super::*;
    use crate::version::semantic_version::SemanticVersion;
    use cargo_metadata::semver::Version;

    fn resolve_version_for_output(
        versions: impl IntoIterator<Item = SemanticVersion>,
        force_single: bool,
    ) -> Result<Option<SemanticVersion>> {
        if force_single {
            Ok(Some(select_single_version(versions)?))
        } else {
            Ok(None)
        }
    }

    fn sv(s: &str) -> SemanticVersion {
        SemanticVersion::try_from(Version::parse(s).unwrap()).unwrap()
    }

    #[test]
    fn force_single_ok_when_versions_match() {
        let versions = vec![sv("1.0.0"), sv("1.0.0")];

        let v = resolve_version_for_output(versions, true).unwrap().unwrap();

        assert_eq!(v.to_string(), "1.0.0");
    }

    #[test]
    fn force_single_errors_on_conflict() {
        let versions = vec![sv("1.0.0"), sv("1.1.0")];

        let err = resolve_version_for_output(versions, true).unwrap_err();
        assert!(err.to_string().contains("different versions"));
    }

    #[test]
    fn force_single_errors_on_empty() {
        let versions: Vec<SemanticVersion> = vec![];

        let err = resolve_version_for_output(versions, true).unwrap_err();
        assert!(err.to_string().contains("No packages found"));
    }

    #[test]
    fn no_force_single_returns_none() {
        let versions = vec![sv("1.0.0"), sv("1.1.0")];

        let result = resolve_version_for_output(versions, false).unwrap();
        assert!(result.is_none());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::version::semantic_version::SemanticVersion;
    use cargo_metadata::semver::Version;

    fn sv(s: &str) -> SemanticVersion {
        SemanticVersion::try_from(Version::parse(s).unwrap()).unwrap()
    }

    #[test]
    fn repr_plain_version() {
        let repr = CurrentVersionRepr::try_from(sv("1.2.3")).unwrap();

        assert_eq!(repr.version, "1.2.3");
        assert_eq!(repr.major, 1);
        assert_eq!(repr.minor, 2);
        assert_eq!(repr.patch, 3);
        assert_eq!(repr.pre, None);
        assert_eq!(repr.iteration, None);
        assert_eq!(repr.build, None);
        assert!(!repr.is_prerelease);
    }

    #[test]
    fn repr_prerelease() {
        let repr = CurrentVersionRepr::try_from(sv("1.2.3-beta.7")).unwrap();

        assert_eq!(repr.pre.as_deref(), Some("beta"));
        assert_eq!(repr.iteration, Some(7));
        assert!(repr.is_prerelease);
    }

    #[test]
    fn json_multiple_packages() {
        let data = CurrentVersionJson {
            packages: vec![
                PackageVersionRepr {
                    package: "foo".into(),
                    version: CurrentVersionRepr::try_from(sv("1.0.0")).unwrap(),
                },
                PackageVersionRepr {
                    package: "bar".into(),
                    version: CurrentVersionRepr::try_from(sv("1.0.0-beta.1")).unwrap(),
                },
            ],
        };

        let json = serde_json::to_string(&data).unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(value["packages"].as_array().unwrap().len(), 2);
        assert_eq!(value["packages"][0]["package"], "foo");
        assert_eq!(value["packages"][1]["pre"], "beta");
    }
}
