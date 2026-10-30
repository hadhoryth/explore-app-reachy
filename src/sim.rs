use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use mujoco_rs::prelude::{MjActuatorDataInfo, MjData, MjJointDataInfo, MjModel};
use mujoco_rs::viewer::MjViewer;

use crate::backend::{Backend, Result};
use crate::motion::{Antennas, CONTROL_PERIOD};

pub const SCENE: &str = "assets/mjcf/scenes/empty.xml";

pub struct Sim {
    data: MjData<Box<MjModel>>,
    viewer: MjViewer,
    right_motor: MjActuatorDataInfo,
    left_motor: MjActuatorDataInfo,
    right_joint: MjJointDataInfo,
    left_joint: MjJointDataInfo,

    /// Seconds of physics per step: 0.002 for this model.
    timestep: f64,
}

impl<'model> Sim {
    pub fn new(scene: &str) -> Result<Self> {
        let model = Box::new(MjModel::from_xml(scene)?);
        let viewer = MjViewer::builder().build_passive(&model)?;
        let timestep = model.opt().timestep;
        let data = MjData::new(model);
        Ok(Sim {
            right_motor: data
                .actuator("right_antenna")
                .ok_or("no right_antenna motor")?,
            left_motor: data
                .actuator("left_antenna")
                .ok_or("no left_antenna motor")?,
            right_joint: data
                .joint("right_antenna")
                .ok_or("no right_antenna joint")?,
            left_joint: data.joint("left_antenna").ok_or("no left_antenna joint")?,
            viewer: viewer,
            timestep: timestep,
            data,
        })
    }

    pub fn hold(&mut self, stop: &AtomicBool) -> Result<()> {
        while self.is_running() && !stop.load(Ordering::Relaxed) {
            self.advance(CONTROL_PERIOD)?;
            thread::sleep(CONTROL_PERIOD);
        }
        Ok(())
    }
}

impl Backend for Sim {
    fn read(&mut self) -> Result<Antennas> {
        Ok(Antennas {
            right: -self.right_joint.view(&self.data).qpos[0],
            left: self.left_joint.view(&self.data).qpos[0],
        })
    }

    fn write(&mut self, target: Antennas) -> Result<()> {
        self.right_motor.view_mut(&mut self.data).ctrl[0] = -target.right;
        self.left_motor.view_mut(&mut self.data).ctrl[0] = target.left;
        Ok(())
    }

    fn advance(&mut self, period: Duration) -> Result<()> {
        let steps = (period.as_secs_f64() / self.timestep).round() as u32;
        for _ in 0..steps {
            self.data.step();
        }
        self.viewer.sync_data(&mut self.data);
        self.viewer.render().ok();
        Ok(())
    }

    fn is_running(&self) -> bool { self.viewer.running() }
}
