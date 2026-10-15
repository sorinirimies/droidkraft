//! Blocking fastboot USB transport on top of `nusb` 0.1.
//!
//! (nusb 0.2 needs `core-foundation 0.10.1`, which conflicts with gpui's
//! `=0.10.0` pin on macOS, so we stay on 0.1.)

use nusb::transfer::{Direction, EndpointType, RequestBuffer};
use nusb::{DeviceInfo, Interface};
use pollster::block_on;

/// Fastboot USB interface descriptor: class 0xff, subclass 0x42, protocol 0x03.
fn fastboot_interface(info: &DeviceInfo) -> Option<u8> {
    info.interfaces()
        .find(|i| i.class() == 0xff && i.subclass() == 0x42 && i.protocol() == 0x03)
        .map(|i| i.interface_number())
}

/// Serial numbers of all attached fastboot devices.
pub fn list() -> Result<Vec<String>, String> {
    let devs = nusb::list_devices().map_err(|e| e.to_string())?;
    Ok(devs
        .filter(|d| fastboot_interface(d).is_some())
        .map(|d| d.serial_number().unwrap_or_default().to_string())
        .collect())
}

pub struct Usb {
    iface: Interface,
    out_ep: u8,
    max_out: usize,
    in_ep: u8,
    max_in: usize,
}

impl Usb {
    /// Open the first fastboot device (or the one matching `serial`).
    pub fn open(serial: Option<&str>) -> Result<Self, String> {
        let mut devs = nusb::list_devices().map_err(|e| e.to_string())?;
        let info = devs
            .find(|d| {
                fastboot_interface(d).is_some()
                    && serial.is_none_or(|s| d.serial_number() == Some(s))
            })
            .ok_or_else(|| "no fastboot device".to_string())?;
        let number = fastboot_interface(&info).ok_or("no fastboot interface")?;
        let device = info.open().map_err(|e| format!("open: {e}"))?;
        let iface = device
            .claim_interface(number)
            .map_err(|e| format!("claim interface: {e}"))?;

        let (out_ep, max_out, in_ep, max_in) = iface
            .descriptors()
            .find_map(|alt| {
                let find = |dir| {
                    alt.endpoints().find_map(|e| {
                        (e.transfer_type() == EndpointType::Bulk && e.direction() == dir)
                            .then(|| (e.address(), e.max_packet_size()))
                    })
                };
                let (o, mo) = find(Direction::Out)?;
                let (i, mi) = find(Direction::In)?;
                Some((o, mo, i, mi))
            })
            .ok_or("fastboot bulk endpoints not found")?;

        Ok(Self {
            iface,
            out_ep,
            max_out,
            in_ep,
            max_in,
        })
    }

    fn send_impl(&mut self, data: &[u8]) -> Result<(), String> {
        block_on(self.iface.bulk_out(self.out_ep, data.to_vec()))
            .into_result()
            .map(|_| ())
            .map_err(|e| format!("usb write: {e}"))
    }

    fn chunk_impl(&self) -> usize {
        (1024 * 1024usize).next_multiple_of(self.max_out)
    }

    fn recv_impl(&mut self) -> Result<Vec<u8>, String> {
        let len = self.max_in.max(512);
        block_on(self.iface.bulk_in(self.in_ep, RequestBuffer::new(len)))
            .into_result()
            .map_err(|e| format!("usb read: {e}"))
    }
}

impl crate::transport::Transport for Usb {
    fn send(&mut self, data: &[u8]) -> Result<(), String> {
        self.send_impl(data)
    }
    fn recv(&mut self) -> Result<Vec<u8>, String> {
        self.recv_impl()
    }
    fn chunk_size(&self) -> usize {
        self.chunk_impl()
    }
}
