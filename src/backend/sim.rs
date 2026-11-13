use std::time::Duration;

use mujoco_rs::prelude::{MjActuatorDataInfo, MjData, MjJointDataInfo, MjModel};
use mujoco_rs::viewer::MjViewer;

use crate::backend::{Backend, Result};
use crate::model::{COUNT, Joints, ReachyMiniSpec};

pub const SCENE: &str = "assets/mjcf/scenes/empty.xml";

pub struct Sim {
    spec: &'static ReachyMiniSpec,
    data: MjData<Box<MjModel>>,
    viewer: MjViewer,
    motors: Vec<MjActuatorDataInfo>,
    joints: Vec<MjJointDataInfo>,
    timestep: f64,
}

impl<'model> Sim {
    pub fn new(spec: &'static ReachyMiniSpec, scene: Option<&str>) -> Result<Self> {
        let scene = scene.unwrap_or(SCENE);
        let model = Box::new(MjModel::from_xml(&scene)?);
        let timestep = model.opt().timestep;
        let viewer = MjViewer::builder().build_passive(&model)?;
        let data = MjData::new(model);

        let mut motors = Vec::new();
        let mut joints = Vec::new();
        for joint in spec.joints().iter() {
            let name = joint.sim_name;
            let actuator = data
                .actuator(name)
                .ok_or(format!("model has no {name} actuator"))?;
            let j = data
                .joint(name)
                .ok_or(format!("model has no {name} joint"))?;
            motors.push(actuator);
            joints.push(j);
        }
        Ok(Sim { spec, data, viewer, motors, joints, timestep })
    }
}

impl Backend for Sim {
    fn spec(&self) -> &'static ReachyMiniSpec { self.spec }
    fn read(&mut self) -> Result<Joints> {
        let mut angles = [0.0; COUNT];
        let joint_specs = self.spec.joints();
        for ((angle, joint), spec) in angles.iter_mut().zip(&self.joints).zip(joint_specs) {
            *angle = spec.sim_sign * joint.view(&self.data).qpos[0];
        }
        Ok(Joints::from_array(angles))
    }

    fn write(&mut self, command: Joints) -> Result<()> {
        let joint_specs = self.spec.joints();
        for ((angle, motor), spec) in command
            .to_array()
            .into_iter()
            .zip(&self.motors)
            .zip(joint_specs)
        {
            motor.view_mut(&mut self.data).ctrl[0] = spec.sim_sign * angle;
        }
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
