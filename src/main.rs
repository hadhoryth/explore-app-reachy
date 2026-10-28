use std::thread;
use std::time::{Duration, Instant};

use mujoco_rs::prelude::MjModel;
use mujoco_rs::viewer::MjViewer;

const SCENE: &str = "assets/mjcf/scenes/empty.xml";
const STEPS_PER_FRAME: u32 = 10;

#[derive(Clone, Copy)]
struct Antennas {
    right: f64,
    left: f64,
}

struct Move {
    target: Antennas,
    duration: f64,
}

fn min_jerk(progress: f64) -> f64 {
    // From The coordination of arm movements: an experimentally confirmed mathematical model”.
    let p = progress.clamp(0.0, 1.0);
    p * p * p * (10.0 - 15.0 * p + 6.0 * p * p)
}

fn interpolate(from: f64, to: f64, progress: f64) -> f64 {
    from + (to - from) * progress
}

fn main() {
    let model = MjModel::from_xml(SCENE).expect("Could not load the Reachy Mini model");

    let mut data = model.make_data();
    let timestep = model.opt().timestep;

    let right_motor = data
        .actuator("right_antenna")
        .expect("no right_antenna motor");
    let left_motor = data
        .actuator("left_antenna")
        .expect("no left_antenna found.");

    let mut viewer = MjViewer::builder()
        .build_passive(&model)
        .expect("Could not open the viewer");

    #[rustfmt::skip]
    let moves = [
        Move { target: Antennas { right: 0.5, left: -0.5 }, duration: 0.5 },
        Move { target: Antennas { right: -0.5, left: 0.5 }, duration: 0.5 },
        Move { target: Antennas { right: 0.0, left: 0.0 }, duration: 0.5 },
    ];

    #[rustfmt::skip]
    let mut current = Antennas { right: 0.0, left: 0.0 };

    for next in &moves {
        let start = current;
        let steps = (next.duration / timestep).round() as u32;
        for step in 1..=steps {
            let tick = Instant::now();
            let progress = min_jerk(f64::from(step) * timestep / next.duration);
            right_motor.view_mut(&mut data).ctrl[0] =
                interpolate(start.right, next.target.right, progress);
            left_motor.view_mut(&mut data).ctrl[0] =
                interpolate(start.left, next.target.left, progress);
            data.step();

            if step % STEPS_PER_FRAME == 0 {
                viewer.sync_data(&mut data);
                viewer.render().ok();
            }
            if let Some(rest) = Duration::from_secs_f64(timestep).checked_sub(tick.elapsed()) {
                thread::sleep(rest);
            }

            current = next.target;
        }
    }

    while viewer.running() {
        data.step();
        viewer.sync_data(&mut data);
        viewer.render().ok();
        thread::sleep(Duration::from_secs_f64(timestep));
    }
}
