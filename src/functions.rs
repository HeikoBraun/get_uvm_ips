use crate::cli::Cli;
use log::{debug, error, info, warn};
use std::process::{exit, Command};
use std::{env, fs};
use toml::Table;

pub fn print_about() {
    println!("Authors: {}", env!("CARGO_PKG_AUTHORS"));
    println!("Version: {}", env!("CARGO_PKG_VERSION"));
    println!("Compiled with rustc {}", env!("VERGEN_RUSTC_SEMVER"));
    println!("Build Timestamp: {}", env!("VERGEN_BUILD_TIMESTAMP"));
    println!("Compiled dependencies:");

    let deps = option_env!("APP_DIRECT_DEPENDENCIES").unwrap_or("");
    if deps.is_empty() {
        println!("  <not available>");
        return;
    }

    for pair in deps.split(',').filter(|s| !s.is_empty()) {
        let (name, version) = pair.split_once('=').unwrap_or((pair, "<unknown>"));
        println!("    - {name}: {version}");
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

pub fn print_help_toml() {
    println!(
        "===================================================================
 databases.toml
===================================================================
# Defaults for databases, so it not has to be defined everywhere.
# Can be empty.
[general]
repository_type = \"<git or projadm>\"
repository = \"ssh::git@github.com:User/project.git\"
build = \"command to build files\"
doc = \"command to generate documentation\"
versions = {{ latest = \"dev\", stable = \"release\" }}

# This is not a database.
# It is a virtual, reserved top database for top build or doc commands.
[top]
build = \"command to build top files\"
doc = \"command to build top documentation\"

# All other sections define databases
[database_dig]
# mandatory if not defined by \"general\"
repository_type = \"<git or projadm>\"
# not necessary/will be ignored for \"projadm\"
repository = \"ssh::git@github.com:User/project_dig.git\"
# optional
build = \"command to build files\"\
# optional
doc = \"command to generate documentation\"\
# optional
versions = {{ latest = \"dev\", stable = \"release\" }}
# hierarchy in which databases are updated, the higher, the better
# optional, default is 0, top has a fix value of 1^31
hierarchy = 2
# optional
skip = true # skip this database
# optional
labels = [\"analog\",\"digital\"]
"
    )
}
