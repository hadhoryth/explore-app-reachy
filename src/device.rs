use std::thread;
use std::time::Duration;

use rustypot::DynamixelProtocolHandler;
use rustypot::servo::dynamixel::xl330;
use serialport::{SerialPort, SerialPortInfo, SerialPortType};

use crate::backend::{self, Backend, Result};
use crate::motion::Antennas;

/// USB vendor and product IDs of the Lite's motor bus adapter.
const USB_VENDOR_ID: u16 = 0x1a86;
const USB_PRODUCT_ID: u16 = 0x55d3;

/// Bits per second on the motor bus, set in every motor's EEPROM.
const BAUD_RATE: u32 = 1_000_000;

/// How long to wait for a motor's reply before giving up.
const REPLY_TIMEOUT: Duration = Duration::from_millis(10);

/// Motor IDs on the bus, right antenna first.
const ANTENNA_IDS: [u8; 2] = [17, 18];

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
        device.set_torque(false)?;
        Ok(device)
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
}

impl Backend for Device {
    fn read(&mut self) -> Result<Antennas> {
        let angles =
            xl330::sync_read_present_position(&self.protocol, self.port.as_mut(), &ANTENNA_IDS)?;
        Ok(Antennas { right: angles[0], left: angles[1] })
    }

    fn write(&mut self, target: Antennas) -> Result<()> { todo!() }
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
}
