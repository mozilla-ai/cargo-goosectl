use anyhow::{Result, anyhow, bail};
use cargo_metadata::Package;

pub struct Metadata(cargo_metadata::Metadata);

impl From<cargo_metadata::Metadata> for Metadata {
    fn from(val: cargo_metadata::Metadata) -> Self {
        Self(val)
    }
}

impl Metadata {
    pub fn select_packages<'a>(
        &'a self,
        workspace: bool,
        packages: &[String],
    ) -> Result<Vec<&'a Package>> {
        match (workspace, packages.is_empty()) {
            (true, false) => {
                bail!("cannot use --workspace with --package");
            }

            (true, true) => {
                // all workspace members
                Ok(self
                    .0
                    .packages
                    .iter()
                    .filter(|p| self.0.workspace_members.contains(&p.id))
                    .collect())
            }

            (false, false) => {
                // specific packages
                let mut out = Vec::new();
                for name in packages {
                    let pkg = self
                        .0
                        .packages
                        .iter()
                        .find(|p| p.name == name)
                        .ok_or_else(|| anyhow!("package `{}` not found", name))?;
                    out.push(pkg);
                }
                Ok(out)
            }

            (false, true) => {
                // if there is a root package, we use that
                if let Some(pkg) = self.0.root_package() {
                    Ok(vec![pkg])
                } else {
                    // no root package → apply to all workspace members
                    Ok(self
                        .0
                        .packages
                        .iter()
                        .filter(|p| self.0.workspace_members.contains(&p.id))
                        .collect())
                }
            }
        }
    }
}
