const MAX_ANGLE: f64 = 0.8;

#[derive(Clone, Copy, Debug)]
pub struct Antennas {
    pub right: f64,
    pub left: f64,
}

impl Antennas {
    pub const REST: Antennas = Antennas { right: -0.1745, left: 0.1745 };

    fn clamped(self) -> Antennas {
        Antennas {
            right: self.right.clamp(-MAX_ANGLE, MAX_ANGLE),
            left: self.left.clamp(-MAX_ANGLE, MAX_ANGLE),
        }
    }

    fn toward(self, target: Antennas, progress: f64) -> Antennas {
        Antennas {
            right: interpolate(self.right, target.right, progress),
            left: interpolate(self.left, target.left, progress),
        }
    }
}

fn min_jerk(progress: f64) -> f64 {
    let p = progress.clamp(0., 1.);
    p * p * p * (10. - 15. * p + 6. * p * p)
}

fn interpolate(from: f64, to: f64, progress: f64) -> f64 { from + (to - from) * progress }

#[cfg(test)]
mod tests {
    use super::*;

    fn pose(right: f64, left: f64) -> Antennas { Antennas { right, left } }

    #[test]
    fn interpolate_goes_from_start_to_end() {
        assert_eq!(interpolate(1.0, 3.0, 0.0), 1.0);
        assert_eq!(interpolate(1.0, 3.0, 0.5), 2.0);
        assert_eq!(interpolate(1.0, 3.0, 1.0), 3.0);
    }

    #[test]
    fn toward_moves_both_antennas() {
        let halfway = pose(0.0, 1.0).toward(pose(1.0, -1.0), 0.5);
        assert_eq!(halfway.right, 0.5);
        assert_eq!(halfway.left, 0.0);
    }

    #[test]
    fn clamped_limits_only_angles_that_are_too_big() {
        let wild = pose(2.0, -3.0).clamped();
        assert_eq!(wild.right, MAX_ANGLE);
        assert_eq!(wild.left, -MAX_ANGLE);
        let fine = pose(0.3, -0.3).clamped();
        assert_eq!(fine.right, 0.3);
        assert_eq!(fine.left, -0.3);
    }

    #[test]
    fn min_jerk_starts_at_0_and_ends_at_1() {
        assert_eq!(min_jerk(0.0), 0.0);
        assert_eq!(min_jerk(0.5), 0.5);
        assert_eq!(min_jerk(1.0), 1.0);
        assert_eq!(min_jerk(2.0), 1.0);
    }
}
