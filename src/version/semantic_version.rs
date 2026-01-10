use anyhow::{Result, bail};
use cargo_metadata::semver::Version;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct SemanticVersion {
    major: u64,
    minor: u64,
    patch: u64,
    pre: Option<Prerelease>,
    build: Option<String>,
}

impl std::fmt::Display for SemanticVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl SemanticVersion {
    pub fn new_release(major: u64, minor: u64, patch: u64) -> Self {
        let pre = None;
        let build = None;

        Self {
            major,
            minor,
            patch,
            pre,
            build
        }
    }

    pub fn major(&self) -> u64 {
        self.major
    }

    pub fn minor(&self) -> u64 {
        self.minor
    }

    pub fn patch(&self) -> u64 {
        self.patch
    }

    pub fn prerelease(&self) -> &Option<Prerelease> {
        &self.pre
    }

    pub fn is_prerelease(&self) -> bool {
        self.pre.is_some()
    }

    pub fn clear_prerelease(mut self) -> Result<Self> {
        self.pre = None;

        Ok(self)
    }

    pub fn build(&self) -> &Option<String> {
        &self.build
    }

    pub fn with_build(mut self, build: Option<String>) -> Result<Self> {
        self.build = build;

        Ok(self)
    }

    pub fn with_prerelease(mut self, prerelease: Prerelease) -> Result<Self> {
        self.pre = Some(prerelease);

        Ok(self)
    }
}

impl TryFrom<Version> for SemanticVersion {
    type Error = anyhow::Error;

    fn try_from(val: Version) -> Result<Self> {
        let pre = match val.pre.is_empty() {
            true => None,
            false => Some(Prerelease::parse(val.pre.as_str())?)
        };

        let build = match val.build.is_empty() {
            true => None,
            false => Some(val.build.to_string())
        };
        

        Ok(Self {
            major: val.major,
            minor: val.minor,
            patch: val.patch,
            pre,
            build
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone)]
pub enum ReleaseLevel {
    Patch,
    Minor,
    Major,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_empty() {
        let v: SemanticVersion = Version::parse("1.2.3").unwrap().try_into().unwrap();
        let build = v.build();

        assert!(build.is_none());
    }

    #[test]
    fn test_build_not_empty() {
        let v: SemanticVersion = Version::parse("1.2.3+asdf").unwrap().try_into().unwrap();
        let build = v.build();

        assert_eq!(build, &Some("asdf".to_string()))
    }

    #[test]
    fn test_semantic_version_display() {
        let v = Version::parse("1.2.3").unwrap();
        let sv = SemanticVersion::try_from(v).unwrap();

        assert_eq!(sv.to_string(), "1.2.3");
    }

    #[test]
    fn test_semantic_version_is_prerelease() {
        let v = Version::parse("1.2.3-beta.1").unwrap();
        let sv = SemanticVersion::try_from(v).unwrap();

        assert!(sv.is_prerelease());
    }

    #[test]
    fn test_semantic_version_not_prerelease() {
        let v = Version::parse("1.2.3").unwrap();
        let sv = SemanticVersion::try_from(v).unwrap();

        assert!(!sv.is_prerelease());
    }

    #[test]
    fn test_semantic_version_clear_prerelease() {
        let v = Version::parse("1.2.3-beta.1").unwrap();
        let sv = SemanticVersion::try_from(v).unwrap();

        let cleared = sv.clear_prerelease().unwrap();

        assert!(!cleared.is_prerelease());
        assert_eq!(cleared.to_string(), "1.2.3");
    }

    #[test]
    fn test_semantic_version_with_build() {
        let v = Version::parse("1.2.3").unwrap();
        let sv = SemanticVersion::try_from(v).unwrap();

        let with_meta = sv.with_build(Some("build.42".to_string())).unwrap();

        assert_eq!(with_meta.to_string(), "1.2.3+build.42");
    }

    #[test]
    fn test_semantic_version_with_invalid_metadata_fails() {
        let v = Version::parse("1.2.3").unwrap();
        let sv = SemanticVersion::try_from(v).unwrap();

        let result = sv.with_build(Some("invalid metadata".to_string()));

        assert!(result.is_err());
    }

    #[test]
    fn test_semantic_version_with_prerelease() {
        let v = Version::parse("1.2.3").unwrap();
        let sv = SemanticVersion::try_from(v).unwrap();

        let pr = Prerelease::parse("alpha.7").unwrap();
        let with_pr = sv.with_prerelease(pr).unwrap();

        assert_eq!(with_pr.to_string(), "1.2.3-alpha.7");
    }

    #[test]
    fn test_bump_patch() {
        let v = Version::parse("1.2.3").unwrap();
        let sv = SemanticVersion::try_from(v).unwrap();

        let bumped = sv.bump_level(ReleaseLevel::Patch).unwrap();

        assert_eq!(bumped.to_string(), "1.2.4");
    }

    #[test]
    fn test_bump_minor() {
        let v = Version::parse("1.2.3").unwrap();
        let sv = SemanticVersion::try_from(v).unwrap();

        let bumped = sv.bump_level(ReleaseLevel::Minor).unwrap();

        assert_eq!(bumped.to_string(), "1.3.0");
    }

    #[test]
    fn test_bump_major() {
        let v = Version::parse("1.2.3").unwrap();
        let sv = SemanticVersion::try_from(v).unwrap();

        let bumped = sv.bump_level(ReleaseLevel::Major).unwrap();

        assert_eq!(bumped.to_string(), "2.0.0");
    }

    #[test]
    fn test_try_from_rejects_invalid_prerelease_format() {
        // semver allows this syntactically, but your wrapper explicitly rejects it
        let v = Version::parse("1.2.3-beta").unwrap();

        let result = SemanticVersion::try_from(v);

        assert!(result.is_err());
    }

    #[test]
    fn test_prerelease_parse_rejects_extra_components() {
        let result = Prerelease::parse("beta.1.extra");

        assert!(result.is_err());
    }

    #[test]
    fn test_prerelease_parse_rejects_non_numeric_iteration() {
        let result = Prerelease::parse("beta.one");

        assert!(result.is_err());
    }

    #[test]
    fn test_prerelease_semver_eq() {
        let gold = cargo_metadata::semver::Prerelease::new("beta.1").unwrap();
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
