use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about="Check out UVM IPs", long_about = None)]
pub struct Cli {
    /// toml
    #[arg(
        short = 't',
        long = "toml",
        help = "Toml file for config of UVM IPs.\nIf not set it will try to use:\n  - $UVM_DIR/admin/tool_setups/uvm_ips.toml\n  - $DB_ADMIN/admin/tool_setups/uvm_ips.toml\n  - ./uvm_ips.toml\nContent is: <UVC name> = <version>"
    )]
    pub toml_file: Option<String>,

    /// dry_run
    #[arg(
        short = 'n',
        long = "dry-run",
        default_value_t = false,
        help = "dry run, no commands will be executed"
    )]
    pub dry_run: bool,

    /// about
    #[arg(
        long = "about",
        default_value_t = false,
        help = "More details about this program"
    )]
    pub about: bool,

    // debug
    #[arg(long, hide = true, help = "debug, no envs")]
    pub debug: bool,
}
