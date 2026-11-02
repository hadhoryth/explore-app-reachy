use std::thread;
use std::time::Duration;

use rustypot::DynamixelProtocolHandler;
use rustypot::servo::dynamixel::xl330;
use serialport::{SerialPort, SerialPortInfo, SerialPortType};

use crate::backend::{Backend, Result};
use crate::motion::Antennas;

/// USB vendor and product IDs of the Lite's motor bus adapter.
const USB_VENDOR_ID: u16 = 0x1a86;
const USB_PRODUCT_ID: u16 = 0x55d3;

/// Bits per second on the motor bus, set in every motor's EEPROM.
const BAUD_RATE: u32 = 1_000_000;

/// The hardware error bit for input voltage, which the Lite's motors don't shut down on.
const INPUT_VOLTAGE_ERROR: u8 = 0b0000_0001;

/// How long to wait for a motor's reply before giving up.
const REPLY_TIMEOUT: Duration = Duration::from_millis(10);

/// Motor IDs on the bus, right antenna first.
const ANTENNA_IDS: [u8; 2] = [17, 18];

/// Position P gain the daemon gives the antenna motors. The factory default is 400.
const ANTENNA_P_GAIN: u16 = 200;

/// How far from upright an antenna may be when torque turns on, in radians. The
/// encoder's range ends at ±π, so starting closer than this to that seam could
/// send the antenna the long way round.
const MAX_START_ANGLE: f64 = 3.1;

pub struct Device {
    port: Box<dyn SerialPort>,
    protocol: DynamixelProtocolHandler,
    torque_on: bool,
}

impl Device {
    pub fn open(path: Option<&str>) -> Result<Device> {
        let path = match path {
            Some(path) => path.to_string(),
            None => find_port()?,
        };
        println!("Using {path}");
        let port = serialport::new(&path, BAUD_RATE)
            .timeout(REPLY_TIMEOUT)
            .open()?;
        let mut device = Device {
            port,
            protocol: DynamixelProtocolHandler::v2(),
            torque_on: false,
        };
        device.clear_errors()?;
        device.set_torque(false)?;
        Ok(device)
    }

    pub fn enable(&mut self) -> Result<()> {
        let angles = self.read()?;
        check_start(angles)?;
        self.write(angles)?;
        xl330::sync_write_position_p_gain(
            &self.protocol,
            self.port.as_mut(),
            &ANTENNA_IDS,
            &[ANTENNA_P_GAIN; 2],
        )?;
        self.set_torque(true)?;
        Ok(())
    }

    fn set_torque(&mut self, on: bool) -> Result<()> {
        xl330::sync_write_torque_enable(
            &self.protocol,
            self.port.as_mut(),
            &ANTENNA_IDS,
            &[on, on],
        )?;
        self.torque_on = on;
        Ok(())
    }

    fn clear_errors(&mut self) -> Result<()> {
        let statuses = xl330::sync_read_hardware_error_status(
            &self.protocol,
            self.port.as_mut(),
            &ANTENNA_IDS,
        )?;
        for (&id, &status) in ANTENNA_IDS.iter().zip(&statuses) {
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
    fn read(&mut self) -> Result<Antennas> {
        let angles =
            xl330::sync_read_present_position(&self.protocol, self.port.as_mut(), &ANTENNA_IDS)?;
        Ok(Antennas { right: angles[0], left: angles[1] })
    }

    fn write(&mut self, target: Antennas) -> Result<()> {
        xl330::sync_write_goal_position(
            &self.protocol,
            self.port.as_mut(),
            &ANTENNA_IDS,
            &[target.right, target.left],
        )
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        if self.torque_on
            && let Err(error) = self.set_torque(false)
        {
            eprintln!("Could not turn the antenna torque off: {error}");
        }
    }
}

fn find_port() -> Result<String> {
    serialport::available_ports()?
        .into_iter()
        .find(is_reachy_lite)
        .map(|port| port.port_name)
        .ok_or_else(|| "no Reachy Mini Lite found".into())
}
fn is_reachy_lite(port: &SerialPortInfo) -> bool {
    matches!(&port.port_type, SerialPortType::UsbPort(usb) if usb.vid == USB_VENDOR_ID && usb.pid == USB_PRODUCT_ID)
}

fn needs_reboot(status: u8) -> bool { status & !INPUT_VOLTAGE_ERROR != 0 }

fn check_start(angles: Antennas) -> Result<()> {
    for angle in [angles.left, angles.right] {
        if angle.abs() > MAX_START_ANGLE {
            return Err(format!(
                "an antenna is at {angle:+.2} rad, too close to ±π; turn it upright by hand"
            )
            .into());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use serialport::UsbPortInfo;

    use super::*;

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
        // Same vendor, different chip: a CH340, common on hobby boards.
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
        assert!(needs_reboot(0b0010_0001)); // overload and low voltage
    }

    fn pose(right: f64, left: f64) -> Antennas { Antennas { right, left } }

    #[test]
    fn starting_poses_near_upright_or_asleep_are_safe() {
        assert!(check_start(pose(0.0, 0.0)).is_ok());
        assert!(check_start(pose(-3.05, 3.05)).is_ok());
    }

    #[test]
    fn starting_near_the_seam_is_refused() {
        assert!(check_start(pose(-3.2, 0.0)).is_err());
        assert!(check_start(pose(0.0, 3.12)).is_err());
        // Turned a whole extra turn by hand while torque was off.
        assert!(check_start(pose(6.4, 0.0)).is_err());
    }
}
