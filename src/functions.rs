use crate::cli::Cli;
use log::{debug, error, info, warn};
use std::process::{exit, Command};
use std::{env, fs};
use toml::Table;

pub fn print_about() {
    println!("Authors: {}", env!("CARGO_PKG_AUTHORS"));
    println!("Version: {}", env!("CARGO_PKG_VERSION"));
    println!("Git:");
    println!(
        "    Remote URL: {}",
        option_env!("APP_GIT_REMOTE_URL").unwrap_or("local git repository")
    );
    println!(
        "    Branch: {}",
        option_env!("APP_GIT_BRANCH").unwrap_or("<unknown>")
    );
    println!(
        "    Commit: {}",
        option_env!("APP_GIT_COMMIT").unwrap_or("<unknown>")
    );
    println!(
        "Compiler: {}",
        option_env!("APP_RUSTC_VERSION").unwrap_or("<unknown>")
    );

    let normal_deps = option_env!("APP_NORMAL_DEPS").unwrap_or("");
    let build_deps = option_env!("APP_BUILD_DEPS").unwrap_or("");

    if !normal_deps.is_empty() {
        println!("Dependencies:");
        for pair in normal_deps.split(',').filter(|s| !s.is_empty()) {
            let (name, version) = pair.split_once('=').unwrap_or((pair, "<unknown>"));
            println!("  - {name}: {version}");
        }
    }

    if !build_deps.is_empty() {
        println!("Build dependencies:");
        for pair in build_deps.split(',').filter(|s| !s.is_empty()) {
            let (name, version) = pair.split_once('=').unwrap_or((pair, "<unknown>"));
            println!("  - {name}: {version}");
        }
    }
}

pub fn get_config(cli_args: &Cli) -> Table {
    if let Some(filename) = &cli_args.toml_file {
        let config = read_toml(&*filename, false);
        if !config.is_empty() {
            info!("Using config file '{}'", filename);
            return config;
        } else {
            warn!("Config file '{}' not found or not valid!", filename);
            exit(1);
        }
    }

    match env::var("UVM_DIR") {
        Ok(value) => {
            let filepath = value + "/admin/tool_setups/uvm_ips.toml";
            let config = read_toml(&filepath, true);
            if !config.is_empty() {
                info!("Using config file '{}'", filepath);
                return config;
            } else {
                info!("Config file '{filepath}' not found or not valid!");
            }
        }
        Err(_) => {
            info!("UVM_DIR is not set!");
        }
    }

    match env::var("DB_ADMIN") {
        Ok(value) => {
            let filepath = value + "/admin/tool_setups/uvm_ips.toml";
            let config = read_toml(&filepath, true);
            if !config.is_empty() {
                info!("Using config file '{}'", filepath);
                return config;
            } else {
                info!("Config file '{filepath}' not found or not valid!");
            }
        }
        Err(_) => {
            info!("DB_ADMIN is not set!");
        }
    }

    let filepath = "uvm_ips.toml";
    let config = read_toml(&filepath, true);
    if !config.is_empty() {
        info!("Using config file '{}'", filepath);
        return config;
    } else {
        info!("Config file '{filepath}' not found or not valid!");
    }
    error!("No config file found!");
    exit(1)
}

fn read_file(filename: &str, ignore_not_exists: bool) -> String {
    let content = match fs::read_to_string(filename) {
        Ok(contents) => {
            debug!("Read file '{}'", filename);
            contents
        }
        Err(e) => {
            if ignore_not_exists {
                debug!("File does not exist: '{}'", filename);
                return "".to_string();
            } else {
                error!("Error reading file '{}': {}", filename, e);
                exit(1)
            }
        }
    };
    content
}

pub fn read_toml(filename: &str, ignore_not_exists: bool) -> Table {
    let contents = read_file(filename, ignore_not_exists);
    contents.parse::<Table>().unwrap_or_else(|err| {
        error!("{}", err);
        exit(1)
    })
}

pub(crate) fn run_cmd(mut cmd: Command, dry_run: bool) {
    if dry_run {
        let program_str = cmd.get_program().to_string_lossy();
        let args_str = cmd
            .get_args()
            .map(|arg| arg.to_string_lossy())
            .collect::<Vec<_>>()
            .join(" ");
        info!("Dry-Run: {program_str} {args_str}");
        return;
    }
    match cmd.status() {
        Ok(_exit_status) => {}
        Err(e) => {
            eprintln!("Error: {}", e);
            eprintln!("       cmd was:   {:?}", cmd.get_program());
            eprintln!("       args were: {:?}", cmd.get_args());
            eprintln!(
                "       cwd:       {:?}",
                cmd.get_current_dir().unwrap_or("./".as_ref())
            );
            exit(1);
        }
    };
}
