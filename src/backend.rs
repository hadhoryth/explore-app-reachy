use std::error::Error;
use std::time::Duration;

use crate::model::{Joints, ReachyMiniSpec};

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

pub trait Backend {
    fn spec(&self) -> &'static ReachyMiniSpec;
    fn is_running(&self) -> bool { true }

    fn read(&mut self) -> Result<Joints>;
    fn write(&mut self, command: Joints) -> Result<()>;

    fn advance(&mut self, period: Duration) -> Result<()> { Ok(()) }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Still;

    impl Backend for Still {
        fn spec(&self) -> &'static ReachyMiniSpec { &crate::model::REACHY_MINI_LITE }

        fn read(&mut self) -> Result<Joints> { Ok(Joints::AWAKE) }

        fn write(&mut self, _command: Joints) -> Result<()> { Ok(()) }
    }

    #[test]
    fn advance_and_is_running_have_defaults() {
        let mut still = Still;
        assert!(still.advance(Duration::from_millis(20)).is_ok());
        assert!(still.is_running());
    }
}
