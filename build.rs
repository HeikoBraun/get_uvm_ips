use std::{error::Error, process::Command};
use vergen::{BuildBuilder, CargoBuilder, Emitter, RustcBuilder, SysinfoBuilder};

fn direct_normal_dependencies_via_cargo_tree() -> Result<String, Box<dyn Error>> {
    let pkg_name = std::env::var("CARGO_PKG_NAME")?;

    let output = Command::new("cargo")
        .args([
            "tree", "-e", "normal", "-p", &pkg_name, "--depth", "1", "--prefix", "none",
        ])
        .output()?;

    if !output.status.success() {
        return Err(format!(
            "cargo tree failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }

    let text = String::from_utf8(output.stdout)?;

    // Output format (typical):
    // mycrate v0.1.0
    // clap v4.5.49
    //
    // We skip the first line (the crate itself), then parse `name version`.
    let mut pairs = Vec::new();
    for (i, line) in text.lines().enumerate() {
        if i == 0 {
            continue;
        }
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let mut parts = line.split_whitespace();
        let name = match parts.next() {
            Some(v) => v,
            None => continue,
        };
        let ver = match parts.next() {
            Some(v) => v.trim_start_matches('v'),
            None => continue,
        };

        pairs.push(format!("{name}={ver}"));
    }

    pairs.sort();
    Ok(pairs.join(","))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Build all "cargo:" instructions (crate name/version, etc.)
    // Requires: vergen = { version = "9.1.0", features = ["cargo"] }
    let build = BuildBuilder::all_build()?;
    let cargo = CargoBuilder::all_cargo()?;
    let rustc = RustcBuilder::all_rustc()?;
    let si = SysinfoBuilder::all_sysinfo()?;

    Emitter::default()
        .add_instructions(&build)?
        .add_instructions(&cargo)?
        .add_instructions(&rustc)?
        .add_instructions(&si)?
        .emit()?;

    let direct_deps = direct_normal_dependencies_via_cargo_tree()?;
    println!("cargo:rustc-env=APP_DIRECT_DEPENDENCIES={direct_deps}");

    Ok(())
}
