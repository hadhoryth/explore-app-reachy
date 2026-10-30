use std::ops::Div;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use crate::backend::{Backend, Result};

const MAX_ANGLE: f64 = 0.8;
pub const CONTROL_PERIOD: Duration = Duration::from_millis(20);

#[derive(Clone, Copy, Debug)]
pub struct Antennas {
    pub right: f64,
    pub left: f64,
}

#[derive(Debug)]
pub struct Move {
    pub target: Antennas,
    pub duration: Duration,
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

pub fn play(backend: &mut dyn Backend, moves: &[Move], stop: &AtomicBool) -> Result<()> {
    let mut current = backend.read()?;
    for next in moves {
        let target = next.target.clamped();
        let ticks = next.duration.div_duration_f64(CONTROL_PERIOD).round() as u32;
        for tik in 1..=ticks {
            if stop.load(Ordering::Relaxed) {
                return Ok(());
            }
            let started = Instant::now();
            let progress = min_jerk(f64::from(tik / ticks));
            backend.write(current.toward(target, progress))?;
            backend.advance(CONTROL_PERIOD)?;
            if let Some(rest) = CONTROL_PERIOD.checked_sub(started.elapsed()) {
                thread::sleep(rest);
            }
        }
        current = target;
    }
    Ok(())
}

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

    struct Recorder {
        position: Antennas,
        written: Vec<Antennas>,
        advanced: u32,
    }

    impl Backend for Recorder {
        fn read(&mut self) -> Result<Antennas> { Ok(self.position) }

        fn write(&mut self, target: Antennas) -> Result<()> {
            self.position = target;
            self.written.push(target);
            Ok(())
        }

        fn advance(&mut self, _period: Duration) -> Result<()> {
            self.advanced += 1;
            Ok(())
        }
    }

    fn recorder() -> Recorder {
        Recorder { position: pose(0.3, -0.3), written: Vec::new(), advanced: 0 }
    }

    fn one_move(right: f64, left: f64) -> [Move; 1] {
        [Move {
            target: pose(right, left),
            duration: Duration::from_millis(100),
        }]
    }

    #[test]
    fn play_sends_one_target_per_tick() {
        let mut backend = recorder();
        play(&mut backend, &one_move(0.5, -0.5), &AtomicBool::new(false)).unwrap();
        // 100 ms at 20 ms per tick.
        assert_eq!(backend.written.len(), 5);
        assert_eq!(backend.advanced, 5);
    }

    #[test]
    fn play_starts_where_the_antennas_are_and_ends_on_target() {
        let mut backend = recorder();
        play(&mut backend, &one_move(0.5, -0.5), &AtomicBool::new(false)).unwrap();
        let first = backend.written[0];
        let last = backend.written[backend.written.len() - 1];
        assert!(
            (first.right - 0.3).abs() < 0.05,
            "first target {first:?} jumped"
        );
        assert_eq!(last.right, 0.5);
        assert_eq!(last.left, -0.5);
    }

    #[test]
    fn play_clamps_targets() {
        let mut backend = recorder();
        play(&mut backend, &one_move(2.0, -2.0), &AtomicBool::new(false)).unwrap();
        assert_eq!(backend.position.right, MAX_ANGLE);
        assert_eq!(backend.position.left, -MAX_ANGLE);
    }

    #[test]
    fn play_takes_real_time() {
        let started = Instant::now();
        play(
            &mut recorder(),
            &one_move(0.5, -0.5),
            &AtomicBool::new(false),
        )
        .unwrap();
        assert!(started.elapsed() >= Duration::from_millis(90));
    }

    #[test]
    fn play_does_nothing_once_stopped() {
        let mut backend = recorder();
        play(&mut backend, &one_move(0.5, -0.5), &AtomicBool::new(true)).unwrap();
        assert!(backend.written.is_empty());
    }
}
