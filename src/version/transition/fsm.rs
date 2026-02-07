use super::TransitionInput;
use crate::version::semantic_version::SemanticVersion;
use anyhow::Result;

macro_rules! grammar {
    {
        $from:ident : $t:ident
        @legal {
            $($legal_state:ident : $legal_transition:ident)+
        }
        @illegal {
            $($illegal_state:ident : $illegal_transition:ident => $error:expr)+
        }
    } => {
        match (&$from, &$t) {
            $((State::$legal_state, TransitionInput::$legal_transition { .. }) => Ok(()),)+
            $((State::$illegal_state, TransitionInput::$illegal_transition { .. }) => Err(anyhow::anyhow!($error)),)+
        }
    };
}

impl SemanticVersion {
    pub fn apply(&self, transition: TransitionInput) -> Result<SemanticVersion> {
        let from = self.state();

        grammar! {
            from : transition
            @legal {
                Release : StartPrerelease
                Prerelease : IncrementPrerelease
                Prerelease : TransitionPrerelease
                Prerelease : FinalizeRelease
                Release : BumpRelease
            }
            @illegal {
                Prerelease : StartPrerelease
                    => "You can only start a new pre-release from a release-level version (e.g., 1.2.3)."
                Release : IncrementPrerelease
                    => "You can only increment a pre-release from an existing pre-release version."
                Release : FinalizeRelease
                    => "Can only finalize release from a prerelease version."
                Prerelease : BumpRelease
                    => "Cannot bump version line of a pre-release version."
                Release : TransitionPrerelease
                    => "You can only transition from one prerelease to another prerelease."
            }
        }?;

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

enum State {
    Release,
    Prerelease,
}
