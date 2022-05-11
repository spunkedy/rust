use std::path::PathBuf;
use std::process::Command;

use crate::cache::INTERNER;
use crate::util::output;
use crate::{Build, Crate};

fn parse_metadata(s: &str) -> Option<Output> {
    let s = smoljson::Value::from_str(s).ok()?;
    let packages: Vec<Package> = s["packages"]
        .as_array()?
        .iter()
        .map(|p| {
            let name = p["name"].as_str()?.to_owned();
            let source = p["source"].as_str().map(ToOwned::to_owned);
            let manifest_path = p["manifest_path"].as_str()?.to_owned();
            let dependencies = p["dependencies"]
                .as_array()?
                .iter()
                .map(|dep| {
                    Some(Dependency {
                        name: dep["name"].as_str()?.to_owned(),
                        source: dep["source"].as_str().map(ToOwned::to_owned),
                    })
                })
                .collect::<Option<Vec<_>>>()?;
            Some(Package { name, source, manifest_path, dependencies })
        })
        .collect::<Option<_>>()?;
    Some(Output { packages })
}

struct Output {
    packages: Vec<Package>,
}

struct Package {
    name: String,
    source: Option<String>,
    manifest_path: String,
    dependencies: Vec<Dependency>,
}

struct Dependency {
    name: String,
    source: Option<String>,
}

pub fn build(build: &mut Build) {
    // Run `cargo metadata` to figure out what crates we're testing.
    let mut cargo = Command::new(&build.initial_cargo);
    cargo
        .arg("metadata")
        .arg("--format-version")
        .arg("1")
        .arg("--no-deps")
        .arg("--manifest-path")
        .arg(build.src.join("Cargo.toml"));

    let output = output(&mut cargo);
    let output: Output =
        parse_metadata(&output).expect("failed to parse metadata (probably `smoljson` bug, tbh)");
    for package in output.packages {
        if package.source.is_none() {
            let name = INTERNER.intern_string(package.name);
            let mut path = PathBuf::from(package.manifest_path);
            path.pop();
            let deps = package
                .dependencies
                .into_iter()
                .filter(|dep| dep.source.is_none())
                .map(|dep| INTERNER.intern_string(dep.name))
                .collect();
            let krate = Crate { name, deps, path };
            let relative_path = krate.local_path(build);
            build.crates.insert(name, krate);
            let existing_path = build.crate_paths.insert(relative_path, name);
            assert!(existing_path.is_none(), "multiple crates with the same path");
        }
    }
}
