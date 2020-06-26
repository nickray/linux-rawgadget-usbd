//! # linux-rawgadget-usbd
//!
//! This is an attempt to use the USB Raw Gadget interface in Linux to implement
//! `usb-device` for desktop.
//!
//! See: <https://github.com/xairy/raw-gadget>
//!
//! Kernel commit: <https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/commit/?id=f2c2e717642c66f7fe7e5dd69b2e8ff5849f4d10>
//!
//! Header: <https://github.com/torvalds/linux/blob/master/include/uapi/linux/usb/raw_gadget.h>
//!
//! Code: <https://github.com/torvalds/linux/blob/master/drivers/usb/gadget/legacy/raw_gadget.c>

use std::{
    ffi::{CStr, CString},
    fs::{File, OpenOptions},
    os::unix::io::AsRawFd,
};

use usb_device::{
    bus::{PollResult, UsbBusAllocator},
    endpoint::{EndpointAddress, EndpointType},
    Result, UsbDirection, UsbError,
};

pub struct UsbBus {
    file: File,
}

#[repr(u8)]
// https://git.io/JJefs
pub enum UsbSpeed {
    Unknown = 0,
    Low = 1,
    Full = 2,
    High = 3,
    Wireless = 4,
    Super = 5,
    SuperPlus = 6,
}

pub mod raw {
    use super::*;

    const UDC_NAME_LENGTH_MAX: usize = 128;

    #[repr(C)]
    pub struct Init {
        driver_name: [u8; UDC_NAME_LENGTH_MAX],
        device_name: [u8; UDC_NAME_LENGTH_MAX],
        speed: u8,
    }

    impl Init {
        pub fn new(driver: &CStr, device: &CStr, speed: UsbSpeed) -> Self {
            let driver_len = driver.to_bytes_with_nul().len();
            assert!(driver_len <= UDC_NAME_LENGTH_MAX);
            let mut driver_name = [0u8; UDC_NAME_LENGTH_MAX];
            driver_name[..driver_len].copy_from_slice(driver.to_bytes_with_nul());

            let device_len = device.to_bytes_with_nul().len();
            assert!(device_len <= UDC_NAME_LENGTH_MAX);
            let mut device_name = [0u8; UDC_NAME_LENGTH_MAX];
            device_name[..device_len].copy_from_slice(device.to_bytes_with_nul());

            Self {
                driver_name,
                device_name,
                speed: speed as u8,
            }
        }
    }

    #[repr(u8) ]
    pub enum EventType {
        Invalid,
        Connect,
        Control,
    }

    #[repr(C)]
    pub struct Event {
        event_type: u32,
        length: u32,
        first_byte: *const u8,
    }

    #[repr(C)]
    pub struct EpIo {
        ep: u16,
        flags: u16,
        length: u32,
        first_byte: *const u8,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    // TODO: double check
    pub struct EpCaps(u32);
    impl EpCaps {
        fn type_control(self) -> bool {
            self.0 & (1 << 0) != 0
        }
        fn type_iso(self) -> bool {
            self.0 & (1 << 1) != 0
        }
        fn type_bulk(self) -> bool {
            self.0 & (1 << 2) != 0
        }
        fn type_int(self) -> bool {
            self.0 & (1 << 3) != 0
        }
        fn dir_in(self) -> bool {
            self.0 & (1 << 4) != 0
        }
        fn dir_out(self) -> bool {
            self.0 & (1 << 5) != 0
        }
    }

    #[repr(C)]
    pub struct EpLimits {
        maxpacket_limit: u16,
        max_streams: u16,
        reserved: u32,
    }

    const EP_NAME_MAX: usize = 16;

    #[repr(C)]
    pub struct EpInfo {
        name: [u8; EP_NAME_MAX],
        addr: u32,
        caps: EpCaps,
        limits: EpLimits,
    }

    const EPS_NUM_MAX: usize = 30;

    #[repr(C)]
    pub struct EpsInfo {
        eps: [EpInfo; EPS_NUM_MAX],
    }

    pub type EndpointDescriptor = libusb_sys::libusb_endpoint_descriptor;


    const ADDR_ANY: u8 = 0xff;

    const MAGIC: u8 = b'U';

    nix::ioctl_write_ptr!(init, MAGIC, 0, Init);
    nix::ioctl_none!(run, MAGIC, 1);
    nix::ioctl_read!(event_fetch, MAGIC, 2, Event);
    nix::ioctl_write_ptr!(ep0_write, MAGIC, 3, EpIo);
    nix::ioctl_readwrite!(ep0_read, MAGIC, 4, EpIo);
    nix::ioctl_write_ptr!(ep_enable, MAGIC, 5, EndpointDescriptor);
    nix::ioctl_write_int!(ep_disable, MAGIC, 6);
    nix::ioctl_write_ptr!(ep_write, MAGIC, 7, EpIo);
    nix::ioctl_readwrite!(ep_read, MAGIC, 8, EpIo);
    nix::ioctl_none!(configure, MAGIC, 9);
    nix::ioctl_write_int!(vbus_draw, MAGIC, 10);
    nix::ioctl_read!(eps_info, MAGIC, 11, EpsInfo);
    nix::ioctl_none!(stall, MAGIC, 12);
    nix::ioctl_write_int!(ep_set_halt, MAGIC, 13);
    nix::ioctl_write_int!(ep_clear_halt, MAGIC, 14);
    nix::ioctl_write_int!(ep_set_wedge, MAGIC, 15);
}

impl UsbBus {
    pub fn new() -> UsbBusAllocator<Self> {
        // TODO: nicer error handling (panics if not run as sudo)
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/raw-gadget")
            .unwrap();

        let bus = Self { file };

        bus.init();
        bus.run();

        UsbBusAllocator::new(bus)
    }

    fn init(&self) {
        let driver = CString::new("dummy_udc").unwrap();
        let device = CString::new("dummy_udc.0").unwrap();

        let initializer = raw::Init::new(&driver, &device, UsbSpeed::High);

        let ioctl_result = unsafe { raw::init(self.file.as_raw_fd(), &initializer) };
        ioctl_result.unwrap();
    }

    fn run(&self) {
        let ioctl_result = unsafe { raw::run(self.file.as_raw_fd()) };
        // thread 'main' panicked at 'called `Result::unwrap()` on an `Err` value: Sys(EINVAL)'
        ioctl_result.unwrap();
    }
}

impl usb_device::bus::UsbBus for UsbBus {
    fn alloc_ep(
        &mut self,
        ep_dir: UsbDirection,
        ep_addr: Option<EndpointAddress>,
        ep_type: EndpointType,
        max_packet_size: u16,
        interval: u8,
    ) -> Result<EndpointAddress> {
        todo!();
    }

    fn enable(&mut self) {
        todo!();
    }

    fn reset(&self) {
        todo!();
    }

    fn set_device_address(&self, addr: u8) {
        todo!();
    }

    fn poll(&self) -> PollResult {
        todo!();
    }

    fn read(&self, ep_addr: EndpointAddress, buf: &mut [u8]) -> Result<usize> {
        todo!();
    }

    fn write(&self, ep_addr: EndpointAddress, buf: &[u8]) -> Result<usize> {
        todo!();
    }

    fn set_stalled(&self, ep_addr: EndpointAddress, stalled: bool) {
        todo!();
    }

    fn is_stalled(&self, ep_addr: EndpointAddress) -> bool {
        todo!();
    }

    fn suspend(&self) {}

    fn resume(&self) {}
}
