use crate::version::semantic_version::{Prerelease, SemanticVersion};

use anyhow::{Result, bail};

mod fsm;

use super::semantic_version::ReleaseLevel;

#[derive(Debug)]
pub enum TransitionInput {
    StartPrerelease {
        level: ReleaseLevel,
        pre: String,
        metadata: Option<String>,
    },
    IncrementPrerelease {
        metadata: Option<String>,
    },
    TransitionPrerelease {
        pre: String,
        metadata: Option<String>,
    },
    FinalizeRelease {
        metadata: Option<String>,
    },
    BumpRelease {
        level: ReleaseLevel,
        metadata: Option<String>,
    },
}

impl SemanticVersion {
    pub fn apply_unchecked(&self, transition: TransitionInput) -> Result<Self> {
        match transition {
            TransitionInput::StartPrerelease {
                level,
                pre,
                metadata,
            } => self.start_prerelease(level, pre, metadata),
            TransitionInput::IncrementPrerelease { metadata } => {
                self.increment_prerelease(metadata)
            }
            TransitionInput::TransitionPrerelease { pre, metadata } => {
                self.transition_prerelease(pre, metadata)
            }
            TransitionInput::FinalizeRelease { metadata } => self.finalize_release(metadata),
            TransitionInput::BumpRelease { level, metadata } => self.bump_release(level, metadata),
        }
    }

    fn start_prerelease(
        &self,
        level: ReleaseLevel,
        pre: String,
        metadata: Option<String>,
    ) -> Result<Self> {
        self.clone()
            .bump_level(level)?
            .with_prerelease(Prerelease {
                ident: pre,
                iteration: 1,
            })?
            .with_build(metadata)
    }

    fn increment_prerelease(&self, metadata: Option<String>) -> Result<Self> {
        let prerelease = match self.prerelease()? {
            Some(p) => p.increment(),
            None => {
                bail!("You can only increment a pre-release from an existing pre-release version.")
            }
        };

        self.clone()
            .with_prerelease(prerelease)?
            .with_build(metadata)
    }

    fn transition_prerelease(&self, pre: String, metadata: Option<String>) -> Result<Self> {
        let new_prerelease = Prerelease {
            ident: pre,
            iteration: 1,
        };

        let old_prerelease = self.prerelease()?.expect("illegal state");

        if new_prerelease.to_semver() <= old_prerelease.to_semver() {
            bail!("New prerelease must be further than old prerelease.")
        }

        self.clone()
            .with_prerelease(new_prerelease)?
            .with_build(metadata)
    }

    fn finalize_release(&self, metadata: Option<String>) -> Result<Self> {
        self.clone().clear_prerelease()?.with_build(metadata)
    }

    fn bump_release(&self, level: ReleaseLevel, metadata: Option<String>) -> Result<Self> {
        self.clone().bump_level(level)?.with_build(metadata)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use cargo_metadata::semver::Version;

    fn sv(s: &str) -> SemanticVersion {
        SemanticVersion::try_from(Version::parse(s).unwrap()).unwrap()
    }

    #[test]
    fn start_prerelease_from_release() {
        let v = sv("1.2.3");

        let next = v
            .apply(TransitionInput::StartPrerelease {
                level: ReleaseLevel::Minor,
                pre: "alpha".into(),
                metadata: None,
            })
            .unwrap();

        assert_eq!(next.to_string(), "1.3.0-alpha.1");
    }

    #[test]
    fn start_prerelease_fails_from_prerelease() {
        let v = sv("1.2.3-beta.1");

        let result = v.apply(TransitionInput::StartPrerelease {
            level: ReleaseLevel::Patch,
            pre: "alpha".into(),
            metadata: None,
        });

        assert!(result.is_err());
    }

    #[test]
    fn increment_prerelease_success() {
        let v = sv("1.2.3-alpha.1");

        let next = v
            .apply(TransitionInput::IncrementPrerelease { metadata: None })
            .unwrap();

        assert_eq!(next.to_string(), "1.2.3-alpha.2");
    }

    #[test]
    fn increment_prerelease_updates_metadata() {
        let v = sv("1.2.3-alpha.1");

        let next = v
            .apply(TransitionInput::IncrementPrerelease {
                metadata: Some("build.9".into()),
            })
            .unwrap();

        assert_eq!(next.to_string(), "1.2.3-alpha.2+build.9");
    }

    #[test]
    fn increment_prerelease_fails_on_release() {
        let v = sv("1.2.3");

        let result = v.apply(TransitionInput::IncrementPrerelease { metadata: None });

        assert!(result.is_err());
    }

    #[test]
    fn transition_prerelease_forward() {
        let v = sv("1.2.3-alpha.3");

        let next = v
            .apply(TransitionInput::TransitionPrerelease {
                pre: "beta".into(),
                metadata: None,
            })
            .unwrap();

        assert_eq!(next.to_string(), "1.2.3-beta.1");
    }

    #[test]
    fn transition_prerelease_rejects_same_or_lower() {
        let v = sv("1.2.3-beta.2");

        let result = v.apply(TransitionInput::TransitionPrerelease {
            pre: "beta".into(),
            metadata: None,
        });

        assert!(result.is_err());
    }

    #[test]
    fn transition_prerelease_fails_on_release() {
        let v = sv("1.2.3");

        let result = v.apply(TransitionInput::TransitionPrerelease {
            pre: "beta".into(),
            metadata: None,
        });

        assert!(result.is_err());
    }

    #[test]
    fn finalize_prerelease_success() {
        let v = sv("1.2.3-rc.4");

        let next = v
            .apply(TransitionInput::FinalizeRelease { metadata: None })
            .unwrap();

        assert_eq!(next.to_string(), "1.2.3");
    }

    #[test]
    fn finalize_prerelease_with_build() {
        let v = sv("1.2.3-rc.4");

        let next = v
            .apply(TransitionInput::FinalizeRelease {
                metadata: Some("build.1".into()),
            })
            .unwrap();

        assert_eq!(next.to_string(), "1.2.3+build.1");
    }

    #[test]
    fn finalize_release_fails_on_release() {
        let v = sv("1.2.3");

        let result = v.apply(TransitionInput::FinalizeRelease { metadata: None });

        assert!(result.is_err());
    }

    #[test]
    fn bump_release_success() {
        let v = sv("1.2.3");

        let next = v
            .apply(TransitionInput::BumpRelease {
                level: ReleaseLevel::Major,
                metadata: None,
            })
            .unwrap();

        assert_eq!(next.to_string(), "2.0.0");
    }

    #[test]
    fn bump_release_with_build() {
        let v = sv("1.2.3");

        let next = v
            .apply(TransitionInput::BumpRelease {
                level: ReleaseLevel::Patch,
                metadata: Some("build.7".into()),
            })
            .unwrap();

        assert_eq!(next.to_string(), "1.2.4+build.7");
    }

    #[test]
    fn bump_release_fails_on_prerelease() {
        let v = sv("1.2.3-alpha.1");

        let result = v.apply(TransitionInput::BumpRelease {
            level: ReleaseLevel::Minor,
            metadata: None,
        });

        assert!(result.is_err());
    }
}
