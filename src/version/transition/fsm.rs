use super::TransitionInput;
use crate::version::semantic_version::SemanticVersion;
use anyhow::Result;

macro_rules! grammar {
    {
        $from:ident@$from_enum:ident : $t:ident@$t_enum:ident
        @legal {
            $($legal_state:ident : $legal_transition:ident;)+
        }
        @illegal {
            $($illegal_state:ident : $illegal_transition:ident => $error:expr;)+
        }
    } => {
        match (&$from, &$t) {
            $(($from_enum::$legal_state, $t_enum::$legal_transition { .. }) => Ok(()),)+
            $(($from_enum::$illegal_state, $t_enum::$illegal_transition { .. }) => Err(anyhow::anyhow!($error)),)+
        }
    };
}

impl SemanticVersion {
    pub fn apply(&self, transition: TransitionInput) -> Result<SemanticVersion> {
        let from = match self.is_prerelease() {
            true => State::Prerelease,
            false => State::Release,
        };

        grammar! {
            from@State : transition@TransitionInput
            @legal {
                Release : StartPrerelease;
                Prerelease : IncrementPrerelease;
                Prerelease : TransitionPrerelease;
                Prerelease : FinalizeRelease;
                Release : BumpRelease;
            }
            @illegal {
                Prerelease : StartPrerelease
                    => "You can only start a new pre-release from a release-level version (e.g., 1.2.3).";
                Release : IncrementPrerelease
                    => "You can only increment a pre-release from an existing pre-release version.";
                Release : FinalizeRelease
                    => "Can only finalize release from a prerelease version.";
                Prerelease : BumpRelease
                    => "Cannot bump version line of a pre-release version.";
                Release : TransitionPrerelease
                    => "You can only transition from one prerelease to another prerelease.";
            }
        }?;

        self.apply_unchecked(transition)
    }
}

enum State {
    Release,
    Prerelease,
}
