// use std::thread;
// use std::time::Duration;

mod model;
mod motion;

use clap::{ArgGroup, Parser};
use std::sync::atomic::{AtomicBool, Ordering};

// use crate::device::Device;
// use crate::motion::{Antennas, play};
// use crate::motions::antenna_wiggle;
// use crate::sim::{SCENE, Sim};
//
//
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

fn main() -> Result<()> {
    let _cli = Cli::parse();
    ctrlc::set_handler(|| STOP.store(true, Ordering::Relaxed))?;
    Ok(())
    // if cli.sim {
    //     run_sim()
    // } else {
    //     run_device(cli.port.as_deref(), cli.watch)
    // }
}

// fn run_sim() -> Result<()> {
//     !todo!("Later")
//     // let mut backend = Sim::new(&SCENE)?;
//     // play(&mut backend, &antenna_wiggle(), &STOP)?;
//     // backend.hold(&STOP)?;
//     // Ok(())
// }
//
// fn run_device(port: Option<&str>, watch_only: bool) -> Result<()> {
//     !todo!("Later")
//     // let mut device = Device::open(port)?;
//     // if watch_only {
//     //     return watch(&mut device);
//     // }
//     // device.enable()?;
//     // play(&mut device, &antenna_wiggle(), &STOP)?;
//     // Ok(())
// }
//
// fn watch(backend: &mut dyn Backend) -> Result<()> {
//     println!("Torque is off move antennas manually");
//     let mut angles = Antennas::REST;
//     while !STOP.load(Ordering::Relaxed) {
//         let next = backend.read()?;
//         if next.right != angles.right || next.left != angles.left {
//             angles = next;
//             println!(
//                 "right {:+.3} rad   left {:+.3} rad",
//                 angles.right, angles.left
//             );
//         }
//         thread::sleep(Duration::from_millis(200));
//     }
//     Ok(())
// }

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
