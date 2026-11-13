use ::std::thread;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use crate::backend::{Backend, Result};
use crate::model::Joints;
use crate::motion::{Interpolate, Target, min_jerk};

pub const CONTROL_PERIOD: Duration = Duration::from_millis(20);
pub const MAX_SPEED: f64 = 2.0;

/// How long waking up and going to sleep take.
pub const POSTURE_TIME: Duration = Duration::from_secs(2);

pub struct ReachyMini<B> {
    backend: B,
    commanded: Joints,
}

impl<B: Backend> ReachyMini<B> {
    pub fn new(mut backend: B) -> Result<Self> {
        let commanded = backend.read()?;
        Ok(ReachyMini { backend, commanded })
    }

    pub fn goto(&mut self, target: Target, duration: Duration, stop: &AtomicBool) -> Result<()> {
        let from = self.commanded;
        let to = target.resolve(from).clamped(self.backend.spec());
        if !to.is_finite() {
            return Err(format!("refusing a target that isn't a number: {to}").into());
        }
        let asked = duration.div_duration_f64(CONTROL_PERIOD).round();
        let slowest = (from.max_difference(to) / MAX_SPEED / CONTROL_PERIOD.as_secs_f64()).ceil();
        let ticks = asked.max(slowest).max(1.0) as u32;
        let mut ticker = Ticker::new(CONTROL_PERIOD);
        for tick in 1..=ticks {
            if stop.load(Ordering::Relaxed) || !self.backend.is_running() {
                return Ok(());
            }
            let command = from.toward(to, min_jerk(f64::from(tick) / f64::from(ticks)));
            self.backend.write(command)?;
            self.commanded = command;
            self.backend.advance(CONTROL_PERIOD)?;
            ticker.wait();
        }
        Ok(())
    }

    pub fn hold(&mut self, stop: &AtomicBool) -> Result<()> {
        let mut ticker = Ticker::new(CONTROL_PERIOD);
        while !stop.load(Ordering::Relaxed) && self.backend.is_running() {
            self.backend.write(self.commanded)?;
            self.backend.advance(CONTROL_PERIOD);
            ticker.wait();
        }
        Ok(())
    }

    pub fn wake_up(&mut self, stop: &AtomicBool) -> Result<()> {
        self.goto(Joints::AWAKE.into(), POSTURE_TIME, stop)
    }

    pub fn sleep(&mut self, abort: &AtomicBool) -> Result<()> {
        self.goto(Joints::SLEEP.into(), POSTURE_TIME, abort)
    }
}

#[derive(Debug)]
pub struct Ticker {
    period: Duration,
    next: Instant,
}

impl Ticker {
    pub fn new(period: Duration) -> Ticker { Ticker { period, next: Instant::now() + period } }
    pub fn wait(&mut self) -> bool {
        let now = Instant::now();
        match self.next.checked_duration_since(now) {
            Some(rest) => {
                thread::sleep(rest);
                self.next += self.period;
                true
            }
            None => {
                self.next = now + self.period;
                false
            }
        }
    }
}
