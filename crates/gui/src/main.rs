#![windows_subsystem = "windows"]
use cli_interface::{arguments::args, entrypoint, Code};
use pretty_env_logger;

fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();
    let args = args();
    
    match entrypoint(args)? {
        Code::Success => Ok(()),
        Code::Failure => std::process::exit(1),
    }
}
