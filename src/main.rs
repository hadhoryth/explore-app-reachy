#![allow(dead_code)]

mod backend;
mod motion;
mod motions;

use clap::{ArgGroup, Parser};

use crate::backend::Result;

#[derive(Parser)]
#[command(group(ArgGroup::new("target").required(true).args(["sim", "device"])))]
struct Cli {
    #[arg(long)]
    sim: bool,

    #[arg(long)]
    device: bool,

    #[arg(long, conflicts_with = "sim")]
    port: Option<String>,

    #[arg(long, conflicts_with = "sim")]
    watch: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    if cli.sim {
        run_sim()
    } else {
        run_device(cli.port.as_deref(), cli.watch)
    }
}

fn run_sim() -> Result<()> { Ok(()) }

fn run_device(port: Option<&str>, watch_only: bool) -> Result<()> { Ok(()) }

#[cfg(test)]
mod tests {
    use clap::{CommandFactory, Parser};

    use super::Cli;

    #[test]
    fn the_flags_are_well_formed() { Cli::command().debug_assert(); }

    #[test]
    fn sim_alone_works() {
        let cli = Cli::try_parse_from(["explore-app", "--sim"]).unwrap();
        assert!(cli.sim);
        assert!(!cli.device);
    }

    #[test]
    fn device_takes_a_port() {
        let cli = Cli::try_parse_from(["explore-app", "--device", "--port", "/dev/x"]).unwrap();
        assert!(cli.device);
        assert_eq!(cli.port.as_deref(), Some("/dev/x"));
    }

    #[test]
    fn a_target_is_required() {
        assert!(Cli::try_parse_from(["explore-app"]).is_err());
    }

    #[test]
    fn only_one_target_at_a_time() {
        assert!(Cli::try_parse_from(["explore-app", "--sim", "--device"]).is_err());
    }

    #[test]
    fn watch_and_port_need_the_device() {
        assert!(Cli::try_parse_from(["explore-app", "--sim", "--watch"]).is_err());
        assert!(Cli::try_parse_from(["explore-app", "--sim", "--port", "/dev/x"]).is_err());
    }
}
