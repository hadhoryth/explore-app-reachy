# Reachy Mini Lite: verified facts

Everything here was checked against Pollen's source or in MuJoCo while writing guides 02 and 03.
Re-check before relying on anything marked as a choice of the guides, and cite the source when a
guide states a fact.

## Sources

- `reachy_mini` (Python SDK and daemon): the learner's clone at
  `~/Documents/Projects/Robotics/reachy/reachy_mini`. Key files:
  `src/reachy_mini/assets/config/hardware_config.yaml`, `assets/kinematics_data.json`,
  `daemon/backend/abstract.py` (poses, wake_up, goto_sleep, reset_to_sleep),
  `daemon/backend/robot/backend.py`, `daemon/backend/mujoco/backend.py`,
  `kinematics/analytical_kinematics.py`.
- `pollen-robotics/reachy-mini-motor-controller` (Rust, under the daemon): `src/controller.rs`,
  `src/control_loop.rs`. Clone it from GitHub.
- `pollen-robotics/reachy_mini_rust_kinematics` (Rust IK/FK, `src/lib.rs`). Not usable directly
  as a Rust dependency without linking Python: it depends on pyo3 unconditionally.
- `pollen-robotics/rustypot` 1.13.1 (`src/servo/dynamixel/xl330.rs`).
- ROBOTIS e-manual, XL330: https://emanual.robotis.com/docs/en/dxl/x/xl330-m288/

## Bus and motors

| # | Joint | ID | Motor | MuJoCo name | Sim sign | Limits | P gain |
|---|---|---|---|---|---|---|---|
| 0 | body_yaw | 10 | XC330-M288 | yaw_body | + | ±160° (daemon `max_body_yaw`) | 200 |
| 1 | stewart_1 | 11 | XL330-M288 | stewart_1 | + | 1502–2958 ticks | 300 |
| 2 | stewart_2 | 12 | XL330-M288 | stewart_2 | + | 1138–2844 | 300 |
| 3 | stewart_3 | 13 | XL330-M288 | stewart_3 | + | 1502–2958 | 300 |
| 4 | stewart_4 | 14 | XL330-M288 | stewart_4 | + | 1138–2594 | 300 |
| 5 | stewart_5 | 15 | XL330-M288 | stewart_5 | + | 1252–2958 | 300 |
| 6 | stewart_6 | 16 | XL330-M288 | stewart_6 | + | 1138–2594 | 300 |
| 7 | right_antenna | 17 | XL330-M077 | right_antenna | − | guide choice ±3.0 rad | 200 |
| 8 | left_antenna | 18 | XL330-M077 | left_antenna | − | guide choice ±3.0 rad | 200 |

- This is the order Pollen's motor controller reads and writes all nine (`read_all_positions`,
  `set_all_goal_positions`). All nine are driven with rustypot's `xl330` functions; the XC330
  shares the control table.
- Serial: USB 1a86:55d3, 1,000,000 baud, 10 ms reply timeout. Operating mode 3 (position).
  Shutdown mask 52; the input-voltage error bit (0b1) is ignored for reboot decisions.
- Homing offsets (EEPROM): ±1024 on the head motors, 0 elsewhere, so present position in radians
  is already the joint angle the kinematics uses.
- rustypot conversion: `rad = 2π·raw/4096 − π`, `raw = (4096·(π + rad)/2π) as i32` (truncates;
  NaN becomes 0, i.e. −π). Converting a limit from rounded degrees can land one tick outside.
- Gains: the daemon writes P from the yaml and I = D = 0 (registers 84, 82, 80).
- Registers used so far: 64 torque enable, 70 hardware error status, 80/82/84 D/I/P gain,
  98 bus watchdog, 116 goal position, 126 present current, 132 present position.
- All nine motors silent → no power (the daemon reports `NoPowerError`).

## Kinematics

- Stewart platform: arm 40 mm, rod 85 mm, head z offset 177 mm (`kinematics_data.json`).
- Pose convention: x forward, y left, z up; `R = Rz(yaw)·Ry(pitch)·Rx(roll)` (scipy "xyz").
  Pitch +15° looks down, roll +15° tilts the right ear down, yaw +25° turns left.
- Body yaw is independent of the head joints: the platform turns with the body. Daemon limits:
  body ±160°, head-to-body relative yaw ±65°.
- `scripts/head_ik.py` reproduces the crate's IK. Poses used in guide 03 (stewart_1..6):

| Pose | Angles |
|---|---|
| NEUTRAL (identity) | 0.6265, −0.6265, 0.6265, −0.6265, 0.6265, −0.6265 |
| LOOK_UP (pitch −15°) | 0.7612, −0.6759, 0.4584, −0.4584, 0.6759, −0.7612 |
| LOOK_DOWN (pitch +15°) | 0.5129, −0.5735, 0.8124, −0.8124, 0.5735, −0.5129 |
| TILT_LEFT (roll −15°) | 0.4961, −0.4661, 0.5875, −0.6714, 0.8142, −0.7591 |
| TILT_RIGHT (roll +15°) | 0.7591, −0.8142, 0.6714, −0.5875, 0.4661, −0.4961 |
| TURN_LEFT (yaw +25°) | 0.7829, −0.5360, 0.7829, −0.5360, 0.7829, −0.5360 |
| TURN_RIGHT (yaw −25°) | 0.5361, −0.7829, 0.5360, −0.7829, 0.5360, −0.7829 |
| SLEEP (daemon `SLEEP_HEAD_POSE`) | −0.1706, 0.8365, −0.1234, 0.0876, −0.8122, 0.1785 |
| CURIOUS (pitch +10°, yaw +15°) | 0.6390, −0.5266, 0.8312, −0.6893, 0.6802, −0.4808 |

- Mirror of a head pose = reverse the six angles and negate them (to 1e-5 for symmetric poses).
- The daemon's `SLEEP_HEAD_JOINT_POSITIONS` constant is stale: it lies outside the head motor
  limits and doesn't match IK of `SLEEP_HEAD_POSE`. Use the IK value.
- Joint-space blends between the poses above are all reachable (least-squares FK, 11 points per
  pair). Path deviation from a straight line: ≤ ~1.4 mm between awake poses, ~5 mm toward SLEEP.

## Daemon behavior worth copying

- Awake antennas ±0.1745 rad (10° tilt against gearbox rattle), sleep antennas ±3.05.
- Before torque on: pin every goal to the present position (`enable_motors`).
- Sleep: move to `SLEEP_HEAD_POSE` and fold antennas, then disable torque. Cutting torque with
  the head up drops it (documented in `reset_to_sleep`). A crashed app can leave torque on.
- 50 Hz command loop; hardware error status checked about once a second.

## MuJoCo model (assets/mjcf, MuJoCo 3.17)

- Nine position actuators, named like their joints, in bus order. Body and head signs match the
  motors; both antennas are negated (Pollen's backend: `ctrl[-2:] = -antennas`).
- All joints at zero: head upright but 27 mm below neutral. SLEEP rests the head on the body
  (contacts), about 5° short of the computed pitch. Every other pose above lands within 0.2°.
- Head pose: site `head`; subtract 177 mm from z for the SDK's head pose.
- `scripts/mujoco_check.py` settles poses headless and reports them.

## Learner's setup

- macOS, rustc 1.99 (Nov 2026), edition 2024; clap 4.6.7, ctrlc 3.5.2, mujoco-rs 7.0.0
  (feature viewer-ui, glutin patched for macOS), rustypot 1.13.1, serialport 4.10.1.
- `rustfmt.toml` sets max_width 100 plus nightly-only options; stable rustfmt ignores those.
- Cloud verification: the [patch.crates-io] glutin entry must be removed on Linux; download
  mujoco-3.17.0-linux-x86_64.tar.gz from GitHub and set MUJOCO_DYNAMIC_LINK_DIR and
  LD_LIBRARY_PATH to its lib folder. The viewer can't open headless, so sim behavior is checked
  with `scripts/mujoco_check.py` on the learner's computer instead.
