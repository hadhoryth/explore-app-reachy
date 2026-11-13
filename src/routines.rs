use std::time::Duration;

use crate::model::{Antennas, Head};
use crate::motion::{Move, Target};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Routine {
    Wiggle,
    Nod,
}

impl Routine {
    pub fn moves(self) -> Vec<Move> {
        match self {
            Routine::Wiggle => wiggle(),
            Routine::Nod => nod(),
        }
    }
}

fn go(target: Target, millis: u64) -> Move {
    Move { target, duration: Duration::from_millis(millis) }
}

pub fn wiggle() -> Vec<Move> {
    vec![
        go(
            Target::new().antennas(Antennas { right: 0.5, left: -0.5 }),
            500,
        ),
        go(
            Target::new().antennas(Antennas { right: -0.5, left: 0.5 }),
            500,
        ),
        go(Target::new().antennas(Antennas::REST), 500),
    ]
}

pub fn nod() -> Vec<Move> {
    vec![
        go(Target::new().head(Head::LOOK_DOWN), 400),
        go(Target::new().head(Head::LOOK_UP), 500),
        go(Target::new().head(Head::LOOK_DOWN), 500),
        go(Target::new().head(Head::NEUTRAL), 400),
    ]
}
