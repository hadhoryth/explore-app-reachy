use std::thread;
use std::time::Duration;

use rustypot::DynamixelProtocolHandler;
use rustypot::servo::dynamixel::xl330;
use serialport::{SerialPort, SerialPortInfo, SerialPortType};

use crate::backend::{Backend, Result};
use crate::model::{COUNT, Joints, ReachyMiniSpec};

struct UsbId {
    vendor: u16,
    product: u16,
}

const BOARD_USB: UsbId = UsbId { vendor: 0x1a86, product: 0x55d3 };
const BAUD_RATE: u32 = 1_000_000;
const REPLY_TIMEOUT: Duration = Duration::from_millis(10);
const INPUT_VOLTAGE_ERROR: u8 = 0b0000_0001;
const START_MARGIN: f64 = 0.1;
const RELEASE_TOLERANCE: f64 = 0.3;

pub struct Device {
    spec: &'static ReachyMiniSpec,
    ids: [u8; COUNT],
    port: Box<dyn SerialPort>,
    protocol: DynamixelProtocolHandler,
    torque_on: bool,
    owns_torque: bool,
}

impl Device {
    pub fn open(path: Option<&str>, spec: &'static ReachyMiniSpec) -> Result<Device> {
        let path = match path {
            Some(path) => path.to_string(),
            None => find_port()?,
        };
        println!("Using {path}");
        let port = serialport::new(&path, BAUD_RATE)
            .timeout(REPLY_TIMEOUT)
            .open()?;
        let mut device = Device {
            spec,
            ids: spec.joints().map(|joint| joint.motor_id),
            port,
            protocol: DynamixelProtocolHandler::v2(),
            torque_on: false,
            owns_torque: false,
        };
        device.check_all_answer()?;
        device.clear_errors()?;
        let torque =
            xl330::sync_read_torque_enable(&device.protocol, device.port.as_mut(), &device.ids)?;
        device.torque_on = torque.contains(&true);
        Ok(device)
    }

    pub fn enable(&mut self) -> Result<()> {
        let joints = self.read()?;
        check_start(joints, self.spec)?;
        self.write(joints)?;
        let gains = self.spec.joints().map(|joint| joint.p_gain);
        let port = self.port.as_mut();

        xl330::sync_write_position_p_gain(&self.protocol, port, &self.ids, &gains)?;
        xl330::sync_write_position_i_gain(&self.protocol, port, &self.ids, &[0; COUNT])?;
        xl330::sync_write_position_d_gain(&self.protocol, port, &self.ids, &[0; COUNT])?;
        self.set_torque(true)
    }

    pub fn release(&mut self) -> Result<()> {
        let joints = self.read()?;
        check_release(self.torque_on, joints)?;
        self.set_torque(false)
    }

    fn set_torque(&mut self, on: bool) -> Result<()> {
        xl330::sync_write_torque_enable(
            &self.protocol,
            self.port.as_mut(),
            &self.ids,
            &[on; COUNT],
        );
        self.torque_on = on;
        self.owns_torque = on;
        Ok(())
    }

    fn check_all_answer(&mut self) -> Result<()> {
        let mut missing = Vec::new();
        for joint in self.spec.joints() {
            let (id, name) = (joint.motor_id, joint.name);
            if !self.protocol.ping(self.port.as_mut(), id).unwrap_or(false) {
                missing.push(format!("{name} ({id})"))
            }
        }
        match missing.len() {
            0 => Ok(()),
            COUNT => Err("no motor answered; is the power supply plugged in?".into()),
            _ => Err(format!("these motors didn't answer: {}", missing.join(", ")).into()),
        }
    }

    fn clear_errors(&mut self) -> Result<()> {
        let statuses =
            xl330::sync_read_hardware_error_status(&self.protocol, self.port.as_mut(), &self.ids)?;
        for (id, status) in self.ids.into_iter().zip(statuses) {
            if !needs_reboot(status) {
                continue;
            }
            println!("Motor {id} reports hardware error {status:#010b}; rebooting it.");
            self.protocol.reboot(self.port.as_mut(), id)?;
            thread::sleep(Duration::from_millis(500));
            if !self.protocol.ping(self.port.as_mut(), id).unwrap_or(false) {
                return Err(format!("motor {id} didn't answer after rebooting").into());
            }
        }
        Ok(())
    }
}

impl Backend for Device {
    fn spec(&self) -> &'static ReachyMiniSpec { &self.spec }

    fn read(&mut self) -> Result<Joints> {
        let angles =
            xl330::sync_read_present_position(&self.protocol, self.port.as_mut(), &self.ids)?;
        let angles: [f64; COUNT] = angles
            .try_into()
            .map_err(|angles: Vec<f64>| format!("expected {COUNT} angles, got {}", angles.len()))?;
        Ok(Joints::from_array(angles))
    }

    fn write(&mut self, command: Joints) -> Result<()> {
        let goals = command.clamped(self.spec).to_array();
        xl330::sync_write_goal_position(&self.protocol, self.port.as_mut(), &self.ids, &goals)
    }
}

impl Drop for Device {
    /// The last line of defense: torque this program turned on goes off. A normal run has
    /// already gone to sleep by now. Torque found on is left alone, as `open` found it.
    fn drop(&mut self) {
        if self.owns_torque
            && let Err(error) = self.set_torque(false)
        {
            eprintln!("Could not turn the torque off: {error}");
        }
    }
}

fn find_port() -> Result<String> {
    serialport::available_ports()?
        .into_iter()
        .find(is_reachy_lite)
        .map(|port| port.port_name)
        .ok_or_else(|| "no Reacy Mini Light found.".into())
}

fn is_reachy_lite(port: &SerialPortInfo) -> bool {
    matches!(&port.port_type, SerialPortType::UsbPort(usb) if usb.vid == BOARD_USB.vendor && usb.pid == BOARD_USB.product)
}

fn check_start(joints: Joints, spec: &ReachyMiniSpec) -> Result<()> {
    for (angle, joint) in joints.to_array().into_iter().zip(spec.joints()) {
        let (name, limits) = (joint.name, joint.limits);
        if !limits.contains(angle, START_MARGIN) {
            return Err(format!(
                "{name} is at {angle:+.2} rad, outside {:+.2} to {:+.2}; move it back by hand",
                limits.min, limits.max
            )
            .into());
        }
    }
    Ok(())
}

fn check_release(torque_on: bool, joints: Joints) -> Result<()> {
    let head_only = Joints { head: joints.head, ..Joints::SLEEP };
    if torque_on && head_only.max_difference(Joints::SLEEP) > RELEASE_TOLERANCE {
        return Err(
            "the motors are holding the head up; run a routine first, it ends asleep".into(),
        );
    }
    Ok(())
}

fn needs_reboot(status: u8) -> bool { status & !INPUT_VOLTAGE_ERROR != 0 }
#[cfg(test)]
mod tests {
    use serialport::UsbPortInfo;

    use super::*;
    use crate::model::{Antennas, REACHY_MINI_LITE};

    fn usb_port(vid: u16, pid: u16) -> SerialPortInfo {
        SerialPortInfo {
            port_name: "/dev/test".to_string(),
            port_type: SerialPortType::UsbPort(UsbPortInfo {
                vid,
                pid,
                serial_number: None,
                manufacturer: None,
                product: None,
            }),
        }
    }

    #[test]
    fn finds_the_lite() {
        assert!(is_reachy_lite(&usb_port(0x1a86, 0x55d3)));
    }

    #[test]
    fn ignores_other_usb_serial_chips() {
        assert!(!is_reachy_lite(&usb_port(0x1a86, 0x7523)));
    }

    #[test]
    fn ignores_ports_that_are_not_usb() {
        let port = SerialPortInfo {
            port_name: "/dev/ttyS0".to_string(),
            port_type: SerialPortType::Unknown,
        };
        assert!(!is_reachy_lite(&port));
    }

    #[test]
    fn healthy_and_low_voltage_need_no_reboot() {
        assert!(!needs_reboot(0b0000_0000));
        assert!(!needs_reboot(0b0000_0001));
    }

    #[test]
    fn real_errors_need_a_reboot() {
        assert!(needs_reboot(0b0010_0000)); // overload
        assert!(needs_reboot(0b0000_0100)); // overheating
    }

    #[test]
    fn awake_and_asleep_are_safe_starts() {
        assert!(check_start(Joints::AWAKE, &REACHY_MINI_LITE).is_ok());
        assert!(check_start(Joints::SLEEP, &REACHY_MINI_LITE).is_ok());
        // The daemon folds the antennas to ±3.05, a little past our ±3.0.
        let daemon_sleep = Joints {
            antennas: Antennas { right: -3.05, left: 3.05 },
            ..Joints::SLEEP
        };
        assert!(check_start(daemon_sleep, &REACHY_MINI_LITE).is_ok());
    }

    #[test]
    fn torque_found_on_is_only_cut_near_sleep() {
        assert!(check_release(false, Joints::AWAKE).is_ok());
        assert!(check_release(true, Joints::SLEEP).is_ok());
        assert!(check_release(true, Joints::AWAKE).is_err());
    }

    #[test]
    fn a_joint_far_outside_its_limits_is_refused() {
        let mut angles = Joints::AWAKE.to_array();
        angles[3] = 2.0; // stewart_3 stops at +1.40
        let error = check_start(Joints::from_array(angles), &REACHY_MINI_LITE).unwrap_err();
        assert!(error.to_string().starts_with("stewart_3"), "{error}");
        // Near the ±π seam, as in guide 02.
        let seam = Joints {
            antennas: Antennas { right: -3.2, left: 0.0 },
            ..Joints::AWAKE
        };
        assert!(check_start(seam, &REACHY_MINI_LITE).is_err());
    }
}
