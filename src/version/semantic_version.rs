use anyhow::{Context, Result, bail};
use cargo_metadata::semver::Version;
use clap::ValueEnum;

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct SemanticVersion(Version);

impl std::fmt::Display for SemanticVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl SemanticVersion {
    pub fn major(&self) -> u64 {
        self.0.major
    }

    pub fn minor(&self) -> u64 {
        self.0.minor
    }

    pub fn patch(&self) -> u64 {
        self.0.patch
    }

    pub fn prerelease(&self) -> Result<Option<Prerelease>> {
        if self.0.pre.is_empty() {
            return Ok(None);
        }

        match Prerelease::parse(self.0.pre.as_str()) {
            Ok(p) => Ok(Some(p)),
            Err(e) => {
                bail!("Prerelease should have been validated. More details: {}", e);
            }
        }
    }

    pub fn is_prerelease(&self) -> bool {
        !self.0.pre.is_empty()
    }

    pub fn clear_prerelease(mut self) -> Result<Self> {
        self.0.pre = cargo_metadata::semver::Prerelease::EMPTY;

        Ok(self)
    }

    pub fn with_metadata(mut self, metadata: Option<String>) -> Result<Self> {
        self.0.build = match metadata {
            Some(m) => cargo_metadata::semver::BuildMetadata::new(&m)
                .with_context(|| "metadata validated by semver")?,
            None => cargo_metadata::semver::BuildMetadata::EMPTY,
        };
        Ok(self)
    }

    pub fn with_prerelease(mut self, prerelease: Prerelease) -> Result<Self> {
        self.0.pre = prerelease.to_semver();

        Ok(self)
    }

    pub fn bump_level(mut self, level: ReleaseLevel) -> Result<Self> {
        let (major, minor, patch) = match level {
            ReleaseLevel::Major => (self.major() + 1, 0, 0),
            ReleaseLevel::Minor => (self.major(), self.minor() + 1, 0),
            ReleaseLevel::Patch => (self.major(), self.minor(), self.patch() + 1),
        };

        self.0.major = major;
        self.0.minor = minor;
        self.0.patch = patch;

        Ok(self)
    }
}

impl TryFrom<Version> for SemanticVersion {
    type Error = anyhow::Error;

    fn try_from(val: Version) -> Result<Self> {
        if !val.pre.is_empty() {
            let _ = Prerelease::parse(val.pre.as_str())?;
        }

        Ok(Self(val))
    }
}

#[derive(Debug, Clone)]
pub struct Prerelease {
    pub ident: String,
    pub iteration: u64,
}

impl Prerelease {
    pub fn parse(s: &str) -> anyhow::Result<Self> {
        let mut parts = s.split('.');

        let ident = parts
            .next()
            .ok_or_else(|| anyhow::anyhow!("Invalid prerelease `{}`", s))?
            .to_string();

        let iteration = parts
            .next()
            .ok_or_else(|| anyhow::anyhow!("Invalid prerelease `{}`: missing counter", s))?
            .parse::<u64>()
            .map_err(|_| anyhow::anyhow!("Invalid prerelease `{}`: counter must be numeric", s))?;

        if parts.next().is_some() {
            bail!("Invalid prerelease `{}`: too many components", s);
        }

        Ok(Self { ident, iteration })
    }

    pub fn increment(&self) -> Self {
        Self {
            ident: self.ident.clone(),
            iteration: self.iteration + 1,
        }
    }

    pub fn to_semver(&self) -> cargo_metadata::semver::Prerelease {
        cargo_metadata::semver::Prerelease::new(&format!("{}.{}", self.ident, self.iteration))
            .expect("always valid")
    }
}

#[derive(Debug, Clone, ValueEnum)]
pub enum ReleaseLevel {
    Patch,
    Minor,
    Major,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prerelease_semver_eq() {
        let gold = cargo_metadata::semver::Prerelease::new(&"beta.1".to_string()).unwrap();
        let pred = Prerelease {
            ident: "beta".to_string(),
            iteration: 1,
        };

        assert_eq!(gold, pred.to_semver());
    }

    #[test]
    fn test_prerelease_parse_successful() {
        let result = Prerelease::parse("beta.1");

        assert!(result.is_ok());

        assert_eq!(
            result.unwrap().to_semver(),
            Prerelease {
                ident: "beta".to_string(),
                iteration: 1
            }
            .to_semver()
        );
    }

    #[test]
    fn test_malformed_prerelease_no_iteration() {
        let result = Prerelease::parse("beta");

        assert!(result.is_err());
    }

    #[test]
    fn test_malformed_prerelease_no_identifier() {
        let result = Prerelease::parse("1");

        assert!(result.is_err());
    }

    #[test]
    fn test_prerelease_increment() {
        let pr = Prerelease::parse("beta.1").unwrap().increment();

        assert_eq!(pr.iteration, 2);
    }
}
