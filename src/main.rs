mod backend;
mod model;
mod motion;
mod reachy;
mod routines;

use clap::{ArgGroup, Parser};
use std::sync::atomic::{AtomicBool, Ordering};

use crate::backend::{Backend, Sim};
use crate::model::REACHY_MINI_LITE;
use crate::reachy::ReachyMini;
use crate::routines::Routine;

use std::error::Error;
use std::result;
pub type Result<T> = result::Result<T, Box<dyn Error>>;

#[derive(Parser)]
#[command(group(ArgGroup::new("target").required(true).args(["sim", "device"])))]
struct Cli {
    /// Run in the MuJoCo simulator.
    #[arg(long)]
    sim: bool,

    /// Drive a Reachy Mini Lite connected over USB.
    #[arg(long)]
    device: bool,

    /// Serial port to use. Found from the robot's USB IDs if left out.
    #[arg(long, conflicts_with = "sim")]
    port: Option<String>,

    /// Only print the antenna angles, with torque off.
    #[arg(long, conflicts_with = "sim")]
    watch: bool,
}

static STOP: AtomicBool = AtomicBool::new(false);
static ABORT: AtomicBool = AtomicBool::new(false);

fn main() -> Result<()> {
    let cli = Cli::parse();
    ctrlc::set_handler(|| STOP.store(true, Ordering::Relaxed))?;
    if cli.sim {
        run_sim();
    } else {
        todo!();
        // run_device(cli.port.as_deref(), cli.watch)
    }
    Ok(())
}

fn run_sim() -> Result<()> {
    let mut reachy = ReachyMini::new(Sim::new(&REACHY_MINI_LITE, None)?)?;
    perform(&mut reachy, Routine::Wiggle, &STOP, &ABORT)?;
    reachy.hold(&STOP)?;
    Ok(())
}

fn perform<B: Backend>(
    reachy: &mut ReachyMini<B>,
    routine: Routine,
    stop: &AtomicBool,
    abort: &AtomicBool,
) -> Result<()> {
    let performed = reachy.wake_up(stop);
    if stop.load(Ordering::Relaxed) {
        println!("Going to sleep. Press Ctrl+C again to cut the torque now.");
    }
    reachy.act(&routine.moves(), &stop)?;
    let slept = reachy.sleep(abort);
    performed.and(slept)
}

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
