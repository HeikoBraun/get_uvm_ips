use crate::functions::{get_config, print_about, run_cmd};
use clap::Parser;
use env_logger::Env;
use log::{debug, error, info, warn};
use std::path::Path;
use std::process::{exit, Command};
use std::{env, fs};

mod cli;
mod functions;

fn checkout_uvm_ip(target_dir: &String, name: String, version: String, dry_run: bool) {
    let target_name = name.to_lowercase();
    let target_path_str = format!("{}/{}", target_dir, &target_name).replace("//", "/");
    info!("Get UVM IP '{}'", name);
    let target_path = Path::new(&target_path_str);

    // if not yet exists, clone
    if !target_path.exists() {
        let mut cmd = Command::new("git");
        cmd.args(vec![
            "clone",
            format!("ssh://git@sourcecode.socialcoding.bosch.com:7999/uvm_ips/{name}.git").as_str(),
            &target_path_str,
        ]);
        run_cmd(cmd, dry_run);
    }

    // pull
    let mut cmd = Command::new("git");
    cmd.args(vec!["-C", &target_path_str, "pull"]);
    run_cmd(cmd, dry_run);

    // checkout
    let mut cmd = Command::new("git");
    cmd.args(vec!["-C", &target_path_str, "checkout", &version]);
    run_cmd(cmd, dry_run);
}

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let cli_args = cli::Cli::parse();

    if cli_args.about {
        print_about();
        exit(0);
    }

    debug!("cli_args: {:?}", cli_args);

    // get config
    let config = get_config(&cli_args);
    debug!("Config:\n{}\n\n", config);

    // have to create UVM_IP_DIR?
    let target_dir: String;
    match env::var("UVM_IP_DIR") {
        Ok(value) => {
            if let Err(e) = fs::create_dir_all(&value) {
                error!("Failed to create UVM_IP_DIR '{}': {}", value, e);
                exit(1);
            }
            target_dir = value;
        }
        Err(_) => {
            warn!("UVM_IP_DIR is not set! => Using ./ as fallback!");
            target_dir = String::from("./");
        }
    }

    // loop over config and check out
    for (key, value) in config {
        match value {
            toml::Value::String(version) => {
                checkout_uvm_ip(&target_dir, key, version, cli_args.dry_run)
            }
            _ => (),
        }
    }
}
