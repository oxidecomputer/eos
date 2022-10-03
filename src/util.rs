use crate::ninja;
use crate::spec;
use rayon::prelude::*;
use std::io::{Error, ErrorKind, Result};
use std::os::unix::fs::DirEntryExt2;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Given a list of source files, return a mapping of source file -> object
/// file.
pub fn object_source_map(
    base_path: &PathBuf,
    src: &Vec<String>,
) -> Result<Vec<(PathBuf, PathBuf)>> {
    let mut objs = Vec::new();
    for src in src {
        let out = match src.strip_suffix(".c") {
            Some(prefix) => prefix.to_owned() + ".o",
            None => {
                return Err(Error::new(
                    ErrorKind::Other,
                    format!("{}: expected c source file", src),
                ));
            }
        };

        let mut in_path = base_path.clone();
        in_path.pop();
        in_path.push(src);

        let mut out_path = PathBuf::new();
        out_path.push("bld");
        out_path.push(base_path);
        out_path.pop();
        out_path.push(&out);
        objs.push((in_path, out_path));
    }
    Ok(objs)
}

/// Create a vector of build statements from a source-object map.
pub fn object_build_statements(
    cflags: Vec<&'static str>,
    obj_src_map: &[(PathBuf, PathBuf)],
) -> Vec<ninja::BuildStatement> {
    // we launch a gcc -H search per object file which is not cheap, so do this
    // over a parallel iterator. On my dev machine with 64 cores this takes
    // the time needed to construct build.ninja from ~30 seconds to ~4 seconds.
    obj_src_map
        .par_iter()
        .map(|(src, obj)| ninja::BuildStatement {
            input: src.to_str().unwrap().to_owned(),
            output: obj.to_str().unwrap().to_owned(),
            rule: ninja::Rules::ModCompile.to_string(),
            implicit_deps: header_deps(&cflags, src.as_path())
                .unwrap()
                .iter()
                .map(|x| x.to_str().unwrap().to_owned())
                .collect::<Vec<String>>(),
            ..Default::default()
        })
        .collect()
}

/// Find all the build files at the given path. This will search the path
/// recursively for any file named `build.toml`.
pub fn find_build_files(path: &Path) -> Result<Vec<PathBuf>> {
    let mut result = Vec::new();
    find_build_files_rec(path, &mut result)?;
    Ok(result)
}

/// Search the file system recursively for all build files.
fn find_build_files_rec(path: &Path, result: &mut Vec<PathBuf>) -> Result<()> {
    for entry in std::fs::read_dir(path)? {
        let e = entry?;
        let ft = e.file_type()?;
        if ft.is_symlink() {
            continue;
        } else if ft.is_dir() {
            find_build_files_rec(&e.path(), result)?;
        } else if e.file_name_ref() == "build.toml" {
            result.push(e.path());
        }
    }

    Ok(())
}

/// Read the given file into a build spec.
pub fn read_spec(path: &Path) -> Result<spec::Spec> {
    let data = std::fs::read_to_string(path)?;
    match toml::from_str(&data) {
        Ok(spec) => Ok(spec),
        Err(e) => Err(Error::new(
            ErrorKind::Other,
            format!("{}: {}", path.display(), e),
        )),
    }
}

/// given a c file, use gcc to find all the headers it depends on
pub fn header_deps(
    compiler_flags: &Vec<&'static str>,
    path: &Path,
) -> Result<Vec<PathBuf>> {
    let mut args = vec!["-H", "-fsyntax-only"];
    args.extend(compiler_flags);
    args.push(path.to_str().unwrap());

    let result = Command::new("gcc-10")
        .args(args)
        .output()
        .expect("failed to execute gcc-10");

    if !result.status.success() {
        return Err(Error::new(
            ErrorKind::Other,
            format!(
                "using gcc to determine header deps failed: {}",
                std::str::from_utf8(&result.stderr).unwrap(),
            ),
        ));
    }

    // for whatever reason gcc puts the output of this command on success ....
    // on stderr - brilliant.
    let out = std::str::from_utf8(&result.stderr).unwrap();

    let mut deps = Vec::new();
    for line in out.lines() {
        if !line.starts_with('.') {
            continue;
        }
        deps.push(Path::new(line.trim_start_matches('.')).to_owned())
    }

    Ok(deps)
}
