use crate::version::semantic_version::{Prerelease, SemanticVersion};

use anyhow::{Result, bail};

use super::semantic_version::ReleaseLevel;

#[derive(Debug)]
pub enum SemverTransition {
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
    pub fn apply(&self, transition: SemverTransition) -> Result<Self> {
        match transition {
            SemverTransition::StartPrerelease {
                level,
                pre,
                metadata,
            } => self.start_prerelease(level, pre, metadata),
            SemverTransition::IncrementPrerelease { metadata } => {
                self.increment_prerelease(metadata)
            }
            SemverTransition::TransitionPrerelease { pre, metadata } => {
                self.transition_prerelease(pre, metadata)
            }
            SemverTransition::FinalizeRelease { metadata } => self.finalize_release(metadata),
            SemverTransition::BumpRelease { level, metadata } => self.bump_release(level, metadata),
        }
    }

    fn start_prerelease(
        &self,
        level: ReleaseLevel,
        pre: String,
        metadata: Option<String>,
    ) -> Result<Self> {
        if self.is_prerelease() {
            bail!(
                "You can only start a new pre-release from a release-level version (e.g., 1.2.3)."
            )
        }

        self.clone()
            .bump_level(level)?
            .with_prerelease(Prerelease {
                ident: pre,
                iteration: 1,
            })?
            .with_metadata(metadata)
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
            .with_metadata(metadata)
    }

    fn transition_prerelease(&self, pre: String, metadata: Option<String>) -> Result<Self> {
        let new_prerelease = Prerelease {
            ident: pre,
            iteration: 1,
        };

        let old_prerelease = match self.prerelease()? {
            Some(p) => p,
            None => bail!("You can only transition from one prerelease to another prerelease."),
        };

        if new_prerelease.to_semver() <= old_prerelease.to_semver() {
            bail!("New prerelease must be further than old prerelease.")
        }

        self.clone()
            .with_prerelease(new_prerelease)?
            .with_metadata(metadata)
    }

    fn finalize_release(&self, metadata: Option<String>) -> Result<Self> {
        if !self.is_prerelease() {
            bail!("Can only finalize release from a prerelease version.");
        }

        self.clone().clear_prerelease()?.with_metadata(metadata)
    }

    fn bump_release(&self, level: ReleaseLevel, metadata: Option<String>) -> Result<Self> {
        if self.is_prerelease() {
            bail!("Cannot bump version line of a pre-release version.");
        }

        self.clone().bump_level(level)?.with_metadata(metadata)
    }
}
