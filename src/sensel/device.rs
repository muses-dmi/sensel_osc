use std::mem;
use std::cell::RefCell;

use super::bindings::*;
use super::result::*;
use super::frame;
use super::contact;

enum LEDArray {
    Char(Vec<u8>),
    Short(Vec<u16>),
}

pub struct DeviceList(SenselDeviceList);

impl SenselSensorInfo {
    pub fn get_num_sensors(&self) -> usize {
        self.num_rows as usize * self.num_cols as usize
    }
}

#[derive(Copy, Clone, Debug)]
pub struct DeviceInfo {
    pub sensor_info: SenselSensorInfo,
    pub fw_info: SenselFirmwareInfo,
    pub supported_frame_content: frame::Mask,
    pub num_leds: usize,
    pub max_led_brightness: u16,
}

pub struct ScanningDevice(BaseDevice);

pub struct BaseDevice {
    handle: *mut ::std::os::raw::c_void,
    frame_data: *mut SenselFrameData,
    led_array_buf: RefCell<LEDArray>,
    info: DeviceInfo,
}

pub fn get_device_list() -> Result<DeviceList, SenselError> {
    unsafe {
        let mut list = mem::zeroed();
        sensel_result(senselGetDeviceList(&mut list))
            .and(Ok(list.into()))
    }
}

impl DeviceList {
    pub fn as_slice(&self) -> &[SenselDeviceID] {
        &self.0.devices[..self.0.num_devices as usize]
    }
}

impl Into<DeviceList> for SenselDeviceList {
    fn into(self) -> DeviceList {
        DeviceList(self)
    }
}

impl SenselDeviceID {
    pub fn get_serial_num(&self) -> &str {
        ::std::str::from_utf8(&self.serial_num).unwrap()
    }
    pub fn get_com_port(&self) -> &str {
        ::std::str::from_utf8(&self.com_port).unwrap()
    }
    pub fn open(self) -> Result<BaseDevice, SenselError> {
        BaseDevice::new(self)
    }
}

impl BaseDevice {
    fn new(id: SenselDeviceID) -> Result<Self, SenselError> {
        unsafe {
            let mut handle = mem::zeroed();
            let mut fw_info = mem::zeroed();
            let mut sensor_info = mem::zeroed();
            let mut frame_data = mem::zeroed();
            let mut supported_frame_content = 0;
            let mut num_leds = 0;
            let mut max_led_brightness = 0;
            let mut led_reg_size = 0;

            sensel_result(senselOpenDeviceByID(&mut handle, id.idx))
                .and_then(|_| sensel_result(senselGetFirmwareInfo(handle, &mut fw_info)))
                .and_then(|_| sensel_result(senselGetSensorInfo(handle, &mut sensor_info)))
                .and_then(|_| sensel_result(senselGetSupportedFrameContent(handle, &mut supported_frame_content)))
                .and_then(|_| sensel_result(senselGetNumAvailableLEDs(handle, &mut num_leds)))
                .and_then(|_| sensel_result(senselGetMaxLEDBrightness(handle, &mut max_led_brightness)))
                .and_then(|_| sensel_result(senselReadReg(handle, SENSEL_REG_LED_BRIGHTNESS_SIZE as u8, SENSEL_REG_SIZE_LED_BRIGHTNESS_SIZE as u8, &mut led_reg_size)))
                .and_then(|_| sensel_result(senselAllocateFrameData(handle, &mut frame_data)))
                .and(Ok(BaseDevice {
                    handle,
                    frame_data,
                    led_array_buf: RefCell::new(match led_reg_size {
                        1 => LEDArray::Char(vec![0; num_leds as usize]),
                        2 => LEDArray::Short(vec![0; num_leds as usize]),
                        _ => unimplemented!()
                    }),
                    info: DeviceInfo {
                        sensor_info,
                        fw_info,
                        supported_frame_content: frame::Mask::from_bits_truncate(supported_frame_content),
                        num_leds: num_leds as usize,
                        max_led_brightness,
                    }
                }))
        }
    }

    pub fn start_scanning(self) -> Result<ScanningDevice, SenselError> {
        unsafe {
            sensel_result(senselStartScanning(self.handle))
                .and(Ok(ScanningDevice(self)))
        }
    }

    pub fn soft_reset(&self) -> Result<(), SenselError> {
        unsafe {
            sensel_result(senselSoftReset(self.handle))
        }
    }
}

impl Drop for BaseDevice {
    fn drop(&mut self) {
        unsafe {
            // always close on drop to prevent memory leaks
            // never throws error
            senselClose(self.handle);
        }
    }
}

impl Device for BaseDevice {
    fn _get_device(&self) -> &BaseDevice {
        self
    }
}

impl ScanningDevice {
    pub fn stop_scanning(self) -> Result<BaseDevice, SenselError> {
        unsafe {
            sensel_result(senselStopScanning(self.0.handle))
                .and(Ok(self.0))
        }
    }

    pub fn read_sensor(&self) -> Result<(), SenselError> {
        unsafe {
            sensel_result(senselReadSensor(self.0.handle))
                .and(Ok(()))
        }
    }
    pub fn get_num_available_frames(&self) -> Result<usize, SenselError> {
        let mut num_frames = 0;
        unsafe {
            sensel_result(senselGetNumAvailableFrames(self.0.handle, &mut num_frames))
                .and(Ok(num_frames as usize))
        }
    }
    pub fn get_frame(&self) -> Result<frame::Frame, SenselError> {
        unsafe {
            sensel_result(senselGetFrame(self.0.handle, self.0.frame_data))
                .and(Ok(frame::from_frame_data(*self.0.frame_data, self.get_info().sensor_info)))
        }
    }
}

impl Device for ScanningDevice {
    fn _get_device(&self) -> &BaseDevice {
        &self.0
    }
}

pub trait Device where Self: ::std::marker::Sized {
    // this should be private but not sure if possible
    fn _get_device(&self) -> &BaseDevice;

    fn get_info(&self) -> DeviceInfo {
        self._get_device().info
    }

    fn get_power_button_pressed(&self) -> Result<bool, SenselError> {
        let dev = self._get_device();
        let mut pressed = 0;
        unsafe {
            sensel_result(senselGetPowerButtonPressed(dev.handle, &mut pressed))
                .and(Ok(pressed != 0))
        }
    }

    fn set_led_array(&self, led_array: &[u16]) -> Result<(), SenselError> {
        let dev = self._get_device();
        let info = self.get_info();
        if led_array.len() != info.num_leds {
            Err(SenselError)
        } else if led_array.iter().any(|&brightness| brightness > info.max_led_brightness) {
            Err(SenselError)
        } else {
            let (buf_ptr, buf_size) = match *dev.led_array_buf.borrow_mut() {
                LEDArray::Char(ref mut buf) => {
                    for (buf, led) in buf.iter_mut().zip(led_array) {
                        *buf = *led as u8;
                    }
                    (buf.as_mut_ptr(), buf.len())
                },
                LEDArray::Short(ref mut buf) => {
                    buf.copy_from_slice(led_array);
                    (buf.as_mut_ptr() as *mut u8, buf.len())
                }
            };
            unsafe {
                let write_size = mem::zeroed();
                sensel_result(senselWriteRegVS(dev.handle, SENSEL_REG_LED_BRIGHTNESS as u8, buf_size as u32, buf_ptr, write_size))
                    .and(Ok(()))
            }
        }
    }

    fn set_led_brightness(&self, led_id: u8, brightness: u16) -> Result<(), SenselError> {
        let dev = self._get_device();
        unsafe {
            sensel_result(senselSetLEDBrightness(dev.handle, led_id, brightness))
        }
    }
    fn get_led_brightness(&self, led_id: u8) -> Result<u16, SenselError> {
        let dev = self._get_device();
        let mut brightness = 0;
        unsafe {
            sensel_result(senselGetLEDBrightness(dev.handle, led_id, &mut brightness))
                .and(Ok(brightness))
        }
    }

    fn set_scan_mode(&self, mode: SenselScanMode) -> Result<(), SenselError> {
        let dev = self._get_device();
        unsafe {
            sensel_result(senselSetScanMode(dev.handle, mode))
        }
    }
    fn get_scan_mode(&self) -> Result<SenselScanMode, SenselError> {
        let dev = self._get_device();
        let mut mode = SenselScanMode::SCAN_MODE_DISABLE;
        unsafe {
            sensel_result(senselGetScanMode(dev.handle, &mut mode))
                .and(Ok(mode))
        }
    }

    fn set_scan_detail(&self, detail: SenselScanDetail) -> Result<(), SenselError> {
        let dev = self._get_device();
        unsafe {
            sensel_result(senselSetScanDetail(dev.handle, detail))
        }
    }
    fn get_scan_detail(&self) -> Result<SenselScanDetail, SenselError> {
        let dev = self._get_device();
        let mut detail = SenselScanDetail::SCAN_DETAIL_UNKNOWN;
        unsafe {
            sensel_result(senselGetScanDetail(dev.handle, &mut detail))
                .and(Ok(detail))
        }
    }

    fn set_buffer_control(&self, num: u8) -> Result<(), SenselError> {
        let dev = self._get_device();
        unsafe {
            sensel_result(senselSetBufferControl(dev.handle, num))
        }
    }
    fn get_buffer_control(&self) -> Result<u8, SenselError> {
        let dev = self._get_device();
        let mut num = 0;
        unsafe {
            sensel_result(senselGetBufferControl(dev.handle, &mut num))
                .and(Ok(num))
        }
    }

    fn set_max_frame_rate(&self, val: u16) -> Result<(), SenselError> {
        let dev = self._get_device();
        unsafe {
            sensel_result(senselSetMaxFrameRate(dev.handle, val))
        }
    }
    fn get_max_frame_rate(&self) -> Result<u16, SenselError> {
        let dev = self._get_device();
        let mut val = 0;
        unsafe {
            sensel_result(senselGetMaxFrameRate(dev.handle, &mut val))
                .and(Ok(val))
        }
    }

    fn set_frame_content(&self, mask: frame::Mask) -> Result<(), SenselError> {
        let dev = self._get_device();
        unsafe {
            sensel_result(senselSetFrameContent(dev.handle, mask.bits()))
        }
    }
    fn get_frame_content(&self) -> Result<frame::Mask, SenselError> {
        let dev = self._get_device();
        let mut mask = 0;
        unsafe {
            sensel_result(senselGetFrameContent(dev.handle, &mut mask))
                .and(Ok(frame::Mask::from_bits_truncate(mask)))
        }
    }

    fn set_contacts_mask(&self, mask: contact::Mask) -> Result<(), SenselError> {
        let dev = self._get_device();
        unsafe {
            sensel_result(senselSetContactsMask(dev.handle, mask.bits()))
        }
    }
    fn get_contacts_mask(&self) -> Result<contact::Mask, SenselError> {
        let dev = self._get_device();
        let mut mask = 0;
        unsafe {
            sensel_result(senselGetContactsMask(dev.handle, &mut mask))
                .and(Ok(contact::Mask::from_bits_truncate(mask)))
        }
    }

    fn set_contacts_min_force(&self, val: u16) -> Result<(), SenselError> {
        let dev = self._get_device();
        unsafe {
            sensel_result(senselSetContactsMinForce(dev.handle, val))
        }
    }
    fn get_contacts_min_force(&self) -> Result<u16, SenselError> {
        let dev = self._get_device();
        let mut val = 0;
        unsafe {
            sensel_result(senselGetContactsMinForce(dev.handle, &mut val))
                .and(Ok(val))
        }
    }

    fn set_contacts_enable_blob_merge(&self, val: bool) -> Result<(), SenselError> {
        let dev = self._get_device();
        unsafe {
            sensel_result(senselSetContactsEnableBlobMerge(dev.handle, val as u8))
        }
    }
    fn get_contacts_enable_blob_merge(&self) -> Result<bool, SenselError> {
        let dev = self._get_device();
        let mut val = 0;
        unsafe {
            sensel_result(senselGetContactsEnableBlobMerge(dev.handle, &mut val))
                .and(Ok(val != 0))
        }
    }

    fn set_dynamic_baseline_enabled(&self, val: bool) -> Result<(), SenselError> {
        let dev = self._get_device();
        unsafe {
            sensel_result(senselSetDynamicBaselineEnabled(dev.handle, val as u8))
        }
    }
    fn get_dynamic_baseline_enabled(&self) -> Result<bool, SenselError> {
        let dev = self._get_device();
        let mut val = 0;
        unsafe {
            sensel_result(senselGetDynamicBaselineEnabled(dev.handle, &mut val))
                .and(Ok(val != 0))
        }
    }

    fn close(self) -> () {
        // move and drop the device
    }
}
