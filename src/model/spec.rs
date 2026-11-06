use crate::model::COUNT;
use std::f64::consts::TAU;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Limits {
    pub min: f64,
    pub max: f64,
}

impl Limits {
    #[must_use]
    pub fn clamp(self, value: f64) -> f64 { value.clamp(self.min, self.max) }
    #[must_use]
    pub fn contains(self, value: f64, margin: f64) -> bool {
        (self.min - margin..=self.max + margin).contains(&value)
    }
    const fn symmetric(max: f64) -> Limits { Limits { min: -max, max } }
    const fn from_ticks(min: i32, max: i32) -> Limits {
        Limits {
            min: Limits::ticks_to_radians(min),
            max: Limits::ticks_to_radians(max),
        }
    }
    const fn ticks_to_radians(v: i32) -> f64 { (v as f64 - 2048.) * TAU / 4096.0 }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct JointSpec {
    name: &'static str,
    pub limits: Limits,
    /// The Dynamixel ID of its motor on the bus.
    pub motor_id: u8,
    // The position P gain its motor runs with.
    pub p_gain: u16,
    pub sim_name: &'static str,
    pub sim_sign: f64,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ReachyMiniSpec {
    pub body: JointSpec,
    pub head: [JointSpec; 6],
    pub right_antenna: JointSpec,
    pub left_antenna: JointSpec,
}

impl ReachyMiniSpec {
    #[must_use]
    pub fn joints(&self) -> [JointSpec; COUNT] {
        let [s1, s2, s3, s4, s5, s6] = self.head;
        [
            self.body,
            s1,
            s2,
            s3,
            s4,
            s5,
            s6,
            self.right_antenna,
            self.left_antenna,
        ]
    }
}

pub static REACHY_MINI_LITE: ReachyMiniSpec = ReachyMiniSpec {
    body: JointSpec {
        name: "body_yaw",
        limits: Limits::symmetric(2.7925),
        motor_id: 10,
        p_gain: 200,
        sim_name: "yaw_body",
        sim_sign: 1.0,
    },
    head: [
        JointSpec {
            name: "stewart_1",
            motor_id: 11,
            limits: Limits::from_ticks(1502, 2958),
            p_gain: 300,
            sim_name: "stewart_1",
            sim_sign: 1.0,
        },
        JointSpec {
            name: "stewart_2",
            motor_id: 12,
            limits: Limits::from_ticks(1138, 2844),
            p_gain: 300,
            sim_name: "stewart_2",
            sim_sign: 1.0,
        },
        JointSpec {
            name: "stewart_3",
            motor_id: 13,
            limits: Limits::from_ticks(1502, 2958),
            p_gain: 300,
            sim_name: "stewart_3",
            sim_sign: 1.0,
        },
        JointSpec {
            name: "stewart_4",
            motor_id: 14,
            limits: Limits::from_ticks(1138, 2594),
            p_gain: 300,
            sim_name: "stewart_4",
            sim_sign: 1.0,
        },
        JointSpec {
            name: "stewart_5",
            motor_id: 15,
            limits: Limits::from_ticks(1252, 2958),
            p_gain: 300,
            sim_name: "stewart_5",
            sim_sign: 1.0,
        },
        JointSpec {
            name: "stewart_6",
            motor_id: 16,
            limits: Limits::from_ticks(1138, 2594),
            p_gain: 300,
            sim_name: "stewart_6",
            sim_sign: 1.0,
        },
    ],
    right_antenna: JointSpec {
        name: "right_antenna",
        motor_id: 17,
        limits: Limits::symmetric(3.0),
        p_gain: 200,
        sim_name: "right_antenna",
        sim_sign: -1.0,
    },
    left_antenna: JointSpec {
        name: "left_antenna",
        motor_id: 18,
        limits: Limits::symmetric(3.0),
        p_gain: 200,
        sim_name: "left_antenna",
        sim_sign: -1.0,
    },
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn joints_come_in_bus_order() {
        let joints = REACHY_MINI_LITE.joints();
        assert_eq!(
            joints.map(|joint| joint.motor_id),
            [10, 11, 12, 13, 14, 15, 16, 17, 18]
        );
        assert_eq!(joints[0].name, "body_yaw");
        assert_eq!(joints[7].name, "right_antenna");
    }

    #[test]
    fn limits_come_from_the_raw_motor_values() {
        // stewart_1: 1502 to 2958 ticks is -48° to +80°.
        let limits = REACHY_MINI_LITE.head[0].limits;
        assert!((limits.min - (-48f64).to_radians()).abs() < 0.001);
        assert!((limits.max - 80f64.to_radians()).abs() < 0.001);
    }

    #[test]
    fn only_the_antennas_turn_the_other_way_in_the_simulator() {
        let signs = REACHY_MINI_LITE.joints().map(|joint| joint.sim_sign);
        assert_eq!(signs, [1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, -1.0, -1.0]);
        assert_eq!(REACHY_MINI_LITE.body.sim_name, "yaw_body");
    }

    #[test]
    fn gains_follow_the_hardware_config() {
        let gains = REACHY_MINI_LITE.joints().map(|joint| joint.p_gain);
        assert_eq!(gains, [200, 300, 300, 300, 300, 300, 300, 200, 200]);
    }

    #[test]
    fn contains_widens_the_limits_by_the_margin() {
        let limits = Limits { min: -1.0, max: 1.0 };
        assert!(limits.contains(1.05, 0.1));
        assert!(!limits.contains(1.15, 0.1));
        assert_eq!(limits.clamp(2.0), 1.0);
    }
}
