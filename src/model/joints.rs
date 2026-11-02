pub const COUNT: usize = 9;

struct Head([f64; 6]);
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

struct Antennas {
    right: f64,
    left: f64,
}
impl Antennas {
    /// Upright, tilted 10° so gravity takes up the gearbox play.
    pub const REST: Antennas = Antennas { right: -0.1745, left: 0.1745 };

    /// Folded back along the head, as the robot sleeps.
    pub const FOLDED: Antennas = Antennas { right: -3.0, left: 3.0 };
}

struct Joints {
    body_yaw: f64,
    head: Head,
    antennas: Antennas,
}
impl Joints {}
