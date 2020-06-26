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
    raw_gadget_file: File,
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

const UDC_NAME_LENGTH_MAX: usize = 128;

#[repr(C)]
pub struct UsbRawInit {
    driver_name: [u8; UDC_NAME_LENGTH_MAX],
    device_name: [u8; UDC_NAME_LENGTH_MAX],
    speed: u8,
}

impl UsbRawInit {
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

const USB_RAW_INIT_MAGIC: u8 = b'U';

nix::ioctl_write_ptr!(usb_raw_init, USB_RAW_INIT_MAGIC, 0, UsbRawInit);
nix::ioctl_none!(usb_raw_run, USB_RAW_INIT_MAGIC, 1);

impl UsbBus {
    pub fn new() -> UsbBusAllocator<Self> {
        // TODO: nicer error handling (panics if not run as sudo)
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/raw-gadget")
            .unwrap();

        let bus = Self {
            raw_gadget_file: file,
        };

        bus.init();
        bus.run();

        UsbBusAllocator::new(bus)
    }

    fn init(&self) {
        let driver = CString::new("dummy_udc").unwrap();
        let device = CString::new("dummy_udc.0").unwrap();

        let initializer = UsbRawInit::new(&driver, &device, UsbSpeed::High);

        let ioctl_result = unsafe { usb_raw_init(self.raw_gadget_file.as_raw_fd(), &initializer) };
        ioctl_result.unwrap();
    }

    fn run(&self) {
        let ioctl_result = unsafe { usb_raw_run(self.raw_gadget_file.as_raw_fd()) };
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
