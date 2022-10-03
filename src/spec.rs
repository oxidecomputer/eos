use crate::ninja;
use crate::util;
use serde_derive::Deserialize;
use std::io::Result;
use std::path::PathBuf;

/// An Eos build specification
#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Spec {
    /// A genunix build spec.
    Genunix(Genunix),
    /// A kernel module build spec.
    Module(Module),
}

impl Spec {
    /// Produce a set of ninja build statements from this spec.
    pub fn to_ninja(
        &self,
        path: &PathBuf,
    ) -> Result<Vec<ninja::BuildStatement>> {
        match self {
            Spec::Genunix(x) => x.to_ninja(path),
            Spec::Module(x) => x.to_ninja(path),
        }
    }
}

/// A build specification for a kernel module.
#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub struct Module {
    /// Name of the kernel module.
    pub name: String,
    /// Source c files.
    pub src: Vec<String>,
    /// Other kernel modules this module depends on.
    #[serde(default = "Vec::new")]
    pub dependencies: Vec<String>,
}

impl Module {
    /// Produce a set of ninja build statements from this spec.
    pub fn to_ninja(
        &self,
        path: &PathBuf,
    ) -> Result<Vec<ninja::BuildStatement>> {
        let osm = util::object_source_map(path, &self.src)?;
        let mut stmts =
            util::object_build_statements(ninja::Spec::kernel_cflags(), &osm);

        let mod_deps = if !self.dependencies.is_empty() {
            vec![ninja::Variable {
                name: "mod_deps".to_owned(),
                value: self
                    .dependencies
                    .iter()
                    .map(|x| format!("-N{}", x))
                    .collect::<Vec<String>>()
                    .join(" "),
            }]
        } else {
            Vec::new()
        };

        stmts.push(ninja::BuildStatement {
            input: osm
                .iter()
                .map(|(_, obj)| obj.to_str().unwrap())
                .collect::<Vec<&str>>()
                .join(" "),
            output: format!("bld/modules/{}", self.name),
            rule: ninja::Rules::ModLink.to_string(),
            variables: mod_deps,
            implicit_deps: vec!["bld/genunix".to_owned()],
        });

        Ok(stmts)
    }
}

/// A build specification for genunix.
#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub struct Genunix {
    /// Source c files.
    pub src: Vec<String>,
}

impl Genunix {
    /// Produce a set of ninja build statements from this spec.
    pub fn to_ninja(
        &self,
        path: &PathBuf,
    ) -> Result<Vec<ninja::BuildStatement>> {
        let osm = util::object_source_map(path, &self.src)?;
        let mut stmts =
            util::object_build_statements(ninja::Spec::kernel_cflags(), &osm);
        stmts.push(ninja::BuildStatement {
            input: osm
                .iter()
                .map(|(_, obj)| obj.to_str().unwrap())
                .collect::<Vec<&str>>()
                .join(" "),
            output: "bld/genunix".to_owned(),
            rule: ninja::Rules::ModLink.to_string(),
            ..Default::default()
        });

        Ok(stmts)
    }
}
