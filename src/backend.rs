use std::error::Error;
use std::time::Duration;

use crate::motion::Antennas;

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

pub trait Backend {
    fn is_running(&self) -> bool { true }

    fn read(&mut self) -> Result<Antennas>;
    fn write(&mut self, target: Antennas) -> Result<()>;

    fn advance(&mut self, period: Duration) -> Result<()> { Ok(()) }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The smallest possible backend: only the two required methods.
    struct Still;

    impl Backend for Still {
        fn read(&mut self) -> Result<Antennas> { Ok(Antennas { right: 0.0, left: 0.0 }) }

        fn write(&mut self, _target: Antennas) -> Result<()> { Ok(()) }
    }

    #[test]
    fn advance_and_is_running_have_defaults() {
        let mut still = Still;
        assert!(still.advance(Duration::from_millis(20)).is_ok());
        assert!(still.is_running());
    }
}
