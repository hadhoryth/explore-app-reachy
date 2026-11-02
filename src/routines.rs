use crate::motion::{Antennas, Move};
use std::time::Duration;

pub fn antenna_wiggle() -> [Move; 4] {
    let duration = Duration::from_millis(500);
    [
        Move { target: Antennas::REST, duration: Duration::from_secs(2) },
        Move {
            target: Antennas { right: 0.5, left: -0.5 },
            duration: duration,
        },
        Move {
            target: Antennas { right: -0.5, left: 0.5 },
            duration: duration,
        },
        Move { target: Antennas::REST, duration },
    ]
}
