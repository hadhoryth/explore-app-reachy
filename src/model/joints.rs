use core::fmt;

use crate::model::ReachyMiniSpec;
use crate::motion::Interpolate;

pub const COUNT: usize = 9;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Antennas {
    right: f64,
    left: f64,
}
impl Antennas {
    /// Upright, tilted 10° so gravity takes up the gearbox play.
    pub const REST: Antennas = Antennas { right: -0.1745, left: 0.1745 };

    /// Folded back along the head, as the robot sleeps.
    pub const FOLDED: Antennas = Antennas { right: -3.0, left: 3.0 };
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Head([f64; 6]);
impl Head {
    /// Upright and looking straight ahead.
    pub const NEUTRAL: Head = Head([0.6265, -0.6265, 0.6265, -0.6265, 0.6265, -0.6265]);
    /// Chin up 15°.
    pub const LOOK_UP: Head = Head([0.7612, -0.6759, 0.4584, -0.4584, 0.6759, -0.7612]);
    /// Chin down 15°.
    pub const LOOK_DOWN: Head = Head([0.5129, -0.5735, 0.8124, -0.8124, 0.5735, -0.5129]);
    /// Left ear 15° toward the left shoulder.
    pub const TILT_LEFT: Head = Head([0.4961, -0.4661, 0.5875, -0.6714, 0.8142, -0.7591]);
    /// Right ear 15° toward the right shoulder.
    pub const TILT_RIGHT: Head = Head([0.7591, -0.8142, 0.6714, -0.5875, 0.4661, -0.4961]);
    /// Turned 25° to its left.
    pub const TURN_LEFT: Head = Head([0.7829, -0.5360, 0.7829, -0.5360, 0.7829, -0.5360]);
    /// Turned 25° to its right.
    pub const TURN_RIGHT: Head = Head([0.5361, -0.7829, 0.5360, -0.7829, 0.5360, -0.7829]);
    /// Lowered 44 mm and pitched forward 24°: the daemon's sleep pose.
    pub const SLEEP: Head = Head([-0.1706, 0.8365, -0.1234, 0.0876, -0.8122, 0.1785]);
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Joints {
    pub body_yaw: f64,
    pub head: Head,
    pub antennas: Antennas,
}

impl Joints {
    pub const AWAKE: Joints =
        Joints { body_yaw: 0.0, head: Head::NEUTRAL, antennas: Antennas::REST };
    pub const SLEEP: Joints =
        Joints { body_yaw: 0.0, head: Head::SLEEP, antennas: Antennas::FOLDED };

    #[must_use]
    pub fn to_array(self) -> [f64; COUNT] {
        let [s1, s2, s3, s4, s5, s6] = self.head.0;
        let Antennas { right, left } = self.antennas;
        [self.body_yaw, s1, s2, s3, s4, s5, s6, right, left]
    }

    #[must_use]
    pub fn from_array(angles: [f64; COUNT]) -> Joints {
        let [body_yaw, s1, s2, s3, s4, s5, s6, right, left] = angles;
        Joints {
            body_yaw,
            head: Head([s1, s2, s3, s4, s5, s6]),
            antennas: Antennas { right, left },
        }
    }

    #[must_use]
    pub fn clamped(self, spec: &ReachyMiniSpec) -> Joints {
        let angles = self.to_array();
        let joints = spec.joints();
        Joints::from_array(std::array::from_fn(|i| joints[i].limits.clamp(angles[i])))
    }

    #[must_use]
    pub fn is_finite(self) -> bool { self.to_array().iter().all(|angle| angle.is_finite()) }

    #[must_use]
    pub fn max_difference(self, other: Joints) -> f64 {
        self.to_array()
            .into_iter()
            .zip(other.to_array())
            .map(|(a, b)| (a - b).abs())
            .fold(0.0, f64::max)
    }
}

impl Interpolate for Antennas {
    fn toward(self, target: Antennas, progress: f64) -> Antennas {
        Antennas {
            right: self.right.toward(target.right, progress),
            left: self.left.toward(target.left, progress),
        }
    }
}

impl Interpolate for Head {
    fn toward(self, target: Head, progress: f64) -> Self { Head(self.0.toward(target.0, progress)) }
}

impl Interpolate for Joints {
    fn toward(self, target: Joints, progress: f64) -> Joints {
        Joints {
            body_yaw: self.body_yaw.toward(target.body_yaw, progress),
            head: self.head.toward(target.head, progress),
            antennas: self.antennas.toward(target.antennas, progress),
        }
    }
}
impl fmt::Display for Joints {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "body {:+.3} | head", self.body_yaw)?;
        for angle in self.head.0 {
            write!(f, " {angle:+.3}")?;
        }
        write!(
            f,
            " | antennas {:+.3} {:+.3}",
            self.antennas.right, self.antennas.left
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::REACHY_MINI_LITE;

    const NAMED_HEADS: [Head; 8] = [
        Head::NEUTRAL,
        Head::LOOK_UP,
        Head::LOOK_DOWN,
        Head::TILT_LEFT,
        Head::TILT_RIGHT,
        Head::TURN_LEFT,
        Head::TURN_RIGHT,
        Head::SLEEP,
    ];

    #[test]
    fn the_array_is_in_bus_order() {
        let joints = Joints {
            body_yaw: 0.1,
            head: Head([1.0, 2.0, 3.0, 4.0, 5.0, 6.0]),
            antennas: Antennas { right: 7.0, left: 8.0 },
        };
        assert_eq!(
            joints.to_array(),
            [0.1, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0]
        );
    }

    #[test]
    fn to_array_and_from_array_round_trip() {
        let angles = [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9];
        assert_eq!(Joints::from_array(angles).to_array(), angles);
    }

    #[test]
    fn clamped_limits_each_joint_to_its_own_range() {
        let wild = Joints::from_array([9.0, -9.0, 9.0, 9.0, -9.0, 9.0, -9.0, 9.0, -9.0]);
        let clamped = wild.clamped(&REACHY_MINI_LITE).to_array();
        let joints = REACHY_MINI_LITE.joints();
        assert_eq!(clamped[0], joints[0].limits.max);
        assert_eq!(clamped[1], joints[1].limits.min);
        assert_eq!(clamped[3], joints[3].limits.max);
        assert_eq!(clamped[8], joints[8].limits.min);
    }

    #[test]
    fn clamped_leaves_good_joints_alone() {
        assert_eq!(Joints::AWAKE.clamped(&REACHY_MINI_LITE), Joints::AWAKE);
        assert_eq!(Joints::SLEEP.clamped(&REACHY_MINI_LITE), Joints::SLEEP);
    }

    #[test]
    fn every_named_head_is_within_the_motor_limits() {
        for head in NAMED_HEADS {
            let joints = Joints { head, ..Joints::AWAKE };
            assert_eq!(joints.clamped(&REACHY_MINI_LITE), joints, "{head:?}");
        }
    }

    #[test]
    fn nan_is_not_finite() {
        assert!(Joints::AWAKE.is_finite());
        assert!(!Joints { body_yaw: f64::NAN, ..Joints::AWAKE }.is_finite());
    }

    #[test]
    fn max_difference_finds_the_joint_that_differs_most() {
        let moved = Joints { body_yaw: 0.25, ..Joints::AWAKE };
        assert_eq!(Joints::AWAKE.max_difference(moved), 0.25);
        assert_eq!(Joints::AWAKE.max_difference(Joints::AWAKE), 0.0);
    }

    #[test]
    fn display_shows_all_nine_angles() {
        let joints = Joints::from_array([0.0, 0.5, -0.5, 0.25, -0.25, 1.0, -1.0, -0.125, 0.125]);
        assert_eq!(
            joints.to_string(),
            "body +0.000 | head +0.500 -0.500 +0.250 -0.250 +1.000 -1.000 | antennas -0.125 +0.125"
        );
    }
}
