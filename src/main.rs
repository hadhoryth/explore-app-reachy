use std::thread;
use std::time::Duration;

use mujoco_rs::prelude::MjModel;
use mujoco_rs::viewer::MjViewer;

fn main() {
    let model = MjModel::from_xml("assets/mjcf/scenes/empty.xml")
        .expect("Could not load the Reachy Mini model");

    let mut data = model.make_data();
    let timestep = model.opt().timestep;
    println!("Initial timestep is {}", timestep);

    let right_motor = data
        .actuator("right_antenna")
        .expect("no right_antenna motor");

    right_motor.view_mut(&mut data).ctrl[0] = 0.5;

    let mut viewer = MjViewer::builder()
        .build_passive(&model)
        .expect("Could not open the viewer");

    while viewer.running(){
        viewer.sync_data(&mut data);
        viewer.render().unwrap();

        data.step();
        thread::sleep(Duration::from_secs_f64(timestep));
    }
}

