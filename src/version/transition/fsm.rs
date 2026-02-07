use super::TransitionInput;
use crate::version::semantic_version::SemanticVersion;
use anyhow::Result;

impl SemanticVersion {
    pub fn apply(&self, transition: TransitionInput) -> Result<SemanticVersion> {
        let from = self.state();
        let kind = transition.kind();

        validate(&from, &kind)?;

        self.apply_unchecked(transition)
    }

    fn state(&self) -> State {
        if self.is_prerelease() {
            State::Prerelease
        } else {
            State::Release
        }
    }
}

impl TransitionInput {
    pub fn kind(&self) -> TransitionKind {
        match self {
            TransitionInput::BumpRelease { .. } => TransitionKind::BumpRelease,
            TransitionInput::FinalizeRelease { .. } => TransitionKind::FinalizeRelease,
            TransitionInput::IncrementPrerelease { .. } => TransitionKind::IncrementPrerelease,
            TransitionInput::StartPrerelease { .. } => TransitionKind::StartPrerelease,
            TransitionInput::TransitionPrerelease { .. } => TransitionKind::TransitionPrerelease,
        }
    }
}

fn validate(from: &State, t: &TransitionKind) -> Result<(), TransitionError> {
    let err = match (from, t) {
        (State::Release, TransitionKind::StartPrerelease)
        | (State::Prerelease, TransitionKind::IncrementPrerelease)
        | (State::Prerelease, TransitionKind::TransitionPrerelease)
        | (State::Prerelease, TransitionKind::FinalizeRelease)
        | (State::Release, TransitionKind::BumpRelease) => return Ok(()),
        (State::Prerelease, TransitionKind::StartPrerelease) => {
            TransitionError::StartPrereleaseFromPrerelease
        }
        (State::Release, TransitionKind::IncrementPrerelease) => {
            TransitionError::IncrementPrereleaseFromRelease
        }
        (State::Release, TransitionKind::FinalizeRelease) => {
            TransitionError::FinalizeReleaseFromRelease
        }
        (State::Prerelease, TransitionKind::BumpRelease) => {
            TransitionError::BumpReleaseFromPrerelease
        }
        (State::Release, TransitionKind::TransitionPrerelease) => {
            TransitionError::TransitionPrereleaseFromRelease
        }
    };

    Err(err)
}

#[derive(Debug)]
pub enum TransitionError {
    StartPrereleaseFromPrerelease,
    IncrementPrereleaseFromRelease,
    FinalizeReleaseFromRelease,
    BumpReleaseFromPrerelease,
    TransitionPrereleaseFromRelease,
}

impl std::error::Error for TransitionError {}

impl std::fmt::Display for TransitionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            TransitionError::StartPrereleaseFromPrerelease => {
                "You can only start a new pre-release from a release-level version (e.g., 1.2.3)."
            }
            TransitionError::IncrementPrereleaseFromRelease => {
                "You can only increment a pre-release from an existing pre-release version."
            }
            TransitionError::FinalizeReleaseFromRelease => {
                "Can only finalize release from a prerelease version."
            }
            TransitionError::BumpReleaseFromPrerelease => {
                "Cannot bump version line of a pre-release version."
            }
            TransitionError::TransitionPrereleaseFromRelease => {
                "You can only transition from one prerelease to another prerelease."
            }
        };
        f.write_str(msg)
    }
}

#[derive(Clone, PartialEq)]
pub enum State {
    Release,
    Prerelease,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransitionKind {
    StartPrerelease,
    IncrementPrerelease,
    TransitionPrerelease,
    FinalizeRelease,
    BumpRelease,
}
