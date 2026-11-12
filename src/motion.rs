use std::time::Duration;

use crate::model::{Antennas, Head, Joints};
pub trait Interpolate: Copy {
    #[must_use]
    fn toward(self, target: Self, progress: f64) -> Self;
}

impl Interpolate for f64 {
    fn toward(self, target: f64, progress: f64) -> Self {
        if progress >= 1.0 {
            return target;
        }
        self + (target - self) * progress
    }
}

impl<const N: usize> Interpolate for [f64; N] {
    fn toward(self, target: [f64; N], progress: f64) -> [f64; N] {
        std::array::from_fn(|i| self[i].toward(target[i], progress))
    }
}

#[must_use]
pub fn min_jerk(progress: f64) -> f64 {
    let p = progress.clamp(0., 1.);
    p * p * p * (10. - 15. * p + 6. * p * p)
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Target {
    body_yaw: Option<f64>,
    head: Option<Head>,
    antennas: Option<Antennas>,
}

impl Target {
    #[must_use]
    pub fn new() -> Target { Target::default() }

    #[must_use]
    pub fn body_yaw(self, body_yaw: f64) -> Target { Target { body_yaw: Some(body_yaw), ..self } }

    #[must_use]
    pub fn head(self, head: Head) -> Target { Target { head: Some(head), ..self } }

    #[must_use]
    pub fn antennas(self, antennas: Antennas) -> Target {
        Target { antennas: Some(antennas), ..self }
    }

    #[must_use]
    pub fn resolve(self, from: Joints) -> Joints {
        Joints {
            body_yaw: self.body_yaw.unwrap_or(from.body_yaw),
            head: self.head.unwrap_or(from.head),
            antennas: self.antennas.unwrap_or(from.antennas),
        }
    }
}

impl From<Joints> for Target {
    fn from(joints: Joints) -> Target {
        Target::new()
            .body_yaw(joints.body_yaw)
            .head(joints.head)
            .antennas(joints.antennas)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Move {
    pub target: Target,
    pub duration: Duration,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_interpolate_from_start_to_end() {
        assert_eq!(1.0.toward(3.0, 0.0), 1.0);
        assert_eq!(1.0.toward(3.0, 0.5), 2.0);
        assert_eq!(1.0.toward(3.0, 1.0), 3.0);
    }

    #[test]
    fn progress_1_lands_exactly_on_the_target() {
        assert_eq!(0.6265.toward(-0.1706, 1.0), -0.1706);
        assert_eq!(0.6265.toward(-0.1706, 0.0), 0.6265);
        assert_eq!(0.6265.toward(0.6265, 0.3), 0.6265);
    }

    #[test]
    fn arrays_interpolate_element_by_element() {
        assert_eq!([0.0, 10.0].toward([1.0, 20.0], 0.5), [0.5, 15.0]);
    }

    #[test]
    fn min_jerk_starts_at_0_and_ends_at_1() {
        assert_eq!(min_jerk(0.0), 0.0);
        assert_eq!(min_jerk(0.5), 0.5);
        assert_eq!(min_jerk(1.0), 1.0);
        assert_eq!(min_jerk(2.0), 1.0);
    }

    #[test]
    fn an_empty_target_keeps_everything() {
        assert_eq!(Target::new().resolve(Joints::SLEEP), Joints::SLEEP);
    }

    #[test]
    fn a_target_replaces_only_the_parts_it_names() {
        let target = Target::new().antennas(Antennas::REST).body_yaw(0.5);
        let joints = target.resolve(Joints::SLEEP);
        assert_eq!(joints.antennas, Antennas::REST);
        assert_eq!(joints.body_yaw, 0.5);
        assert_eq!(joints.head, Head::SLEEP);
    }

    #[test]
    fn a_target_from_joints_names_every_part() {
        assert_eq!(
            Target::from(Joints::AWAKE).resolve(Joints::SLEEP),
            Joints::AWAKE
        );
    }
}
