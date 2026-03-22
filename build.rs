use std::{error::Error, process::Command};

fn git_remote_name() -> Result<String, Box<dyn Error>> {
    let output = Command::new("git").args(["remote"]).output()?;

    if !output.status.success() {
        return Err(format!(
            "git remote failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }

    let text = String::from_utf8(output.stdout)?;
    Ok(text.lines().next().unwrap_or("").trim().to_string())
}

fn git_branch_name() -> Result<String, Box<dyn Error>> {
    let output = Command::new("git")
        .args(["branch", "--show-current"])
        .output()?;

    if !output.status.success() {
        return Err(format!(
            "git branch failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }

    let text = String::from_utf8(output.stdout)?;
    Ok(text.trim().to_string())
}

fn deps_via_cargo_tree(kind: &str) -> Result<String, Box<dyn Error>> {
    let pkg_name = std::env::var("CARGO_PKG_NAME")?;

    let output = Command::new("cargo")
        .args([
            "tree", "-e", kind, "-p", &pkg_name, "--prefix", "none", "--depth", "1",
        ])
        .output()?;

    if !output.status.success() {
        return Err(format!(
            "cargo tree ({kind}) failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }

    let text = String::from_utf8(output.stdout)?;

    let mut pairs = Vec::new();
    for (i, line) in text.lines().enumerate() {
        if i == 0 {
            continue; // skip the package itself
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

fn git_remote_url(remote: &str) -> Option<String> {
    let output = Command::new("git")
        .args(["remote", "get-url", remote])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let text = String::from_utf8(output.stdout).ok()?;
    let url = text.trim();
    if url.is_empty() {
        None
    } else {
        Some(url.to_string())
    }
}

fn git_commit_hash() -> Option<String> {
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let text = String::from_utf8(output.stdout).ok()?;
    Some(text.trim().to_string())
}

fn rustc_version() -> Option<String> {
    let output = Command::new("rustc").args(["--version"]).output().ok()?;

    if !output.status.success() {
        return None;
    }

    let text = String::from_utf8(output.stdout).ok()?;
    Some(text.trim().to_string())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let normal_deps = deps_via_cargo_tree("normal")?;
    let build_deps = deps_via_cargo_tree("build")?;

    println!("cargo:rustc-env=APP_NORMAL_DEPS={normal_deps}");
    println!("cargo:rustc-env=APP_BUILD_DEPS={build_deps}");

    // Git
    let git_remote = git_remote_name()?;
    let git_remote_url = git_remote_url("origin").unwrap_or_else(|| "<unknown>".to_string());
    let git_branch = git_branch_name()?;
    let git_commit = git_commit_hash().unwrap_or_else(|| "<unknown>".to_string());

    println!("cargo:rustc-env=APP_GIT_REMOTE={git_remote}");
    println!("cargo:rustc-env=APP_GIT_REMOTE_URL={git_remote_url}");
    println!("cargo:rustc-env=APP_GIT_BRANCH={git_branch}");
    println!("cargo:rustc-env=APP_GIT_COMMIT={git_commit}");

    let rustc_version = rustc_version().unwrap_or_else(|| "<unknown>".to_string());
    println!("cargo:rustc-env=APP_RUSTC_VERSION={rustc_version}");

    Ok(())
}
