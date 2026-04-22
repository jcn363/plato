//! Display controller abstraction for e-ink devices
//!
//! Provides a trait abstraction for different e-ink display controllers.

use anyhow::Result;
use crate::geom::Rectangle;
use crate::eink::waveform::WaveformMode;

/// Trait for e-ink display controllers
pub trait EInkController {
    fn update(&self, region: Rectangle, data: &[u8], waveform: WaveformMode) -> Result<()>;
    fn full_refresh(&self) -> Result<()>;
    fn set_waveform_lut(&self, lut: &[u8]) -> Result<()>;
    fn get_controller_name(&self) -> &str;
}

/// Sunxi disp2 controller for Allwinner-based Kobo devices (Elipsa, Sage)
#[derive(Debug)]
pub struct SunxiController {
    device_path: String,
}

impl SunxiController {
    pub fn new(device_path: String) -> Result<Self> {
        if device_path.is_empty() {
            anyhow::bail!("Device path cannot be empty");
        }
        Ok(Self { device_path })
    }

    pub fn default() -> Result<Self> {
        Ok(Self::new("/dev/disp_eink".to_string())?)
    }
}

impl EInkController for SunxiController {
    fn update(&self, _region: Rectangle, data: &[u8], _waveform: WaveformMode) -> Result<()> {
        if data.is_empty() {
            anyhow::bail!("Data cannot be empty for update");
        }

        // TODO: Implement actual DISP_EINK_UPDATE2 ioctl
        // This is a placeholder for the actual implementation
        Ok(())
    }

    fn full_refresh(&self) -> Result<()> {
        // TODO: Implement full refresh via ioctl
        Ok(())
    }

    fn set_waveform_lut(&self, lut: &[u8]) -> Result<()> {
        if lut.is_empty() {
            anyhow::bail!("Waveform LUT cannot be empty");
        }

        // TODO: Implement waveform LUT programming
        Ok(())
    }

    fn get_controller_name(&self) -> &str {
        "sunxi-disp2"
    }
}

/// MXC EPDC controller for Freescale i.MX-based Kobo devices
#[derive(Debug)]
pub struct MxcController {
    device_path: String,
}

impl MxcController {
    pub fn new(device_path: String) -> Result<Self> {
        if device_path.is_empty() {
            anyhow::bail!("Device path cannot be empty");
        }
        Ok(Self { device_path })
    }

    pub fn default() -> Result<Self> {
        Ok(Self::new("/dev/fb0".to_string())?)
    }
}

impl EInkController for MxcController {
    fn update(&self, _region: Rectangle, data: &[u8], _waveform: WaveformMode) -> Result<()> {
        if data.is_empty() {
            anyhow::bail!("Data cannot be empty for update");
        }

        // TODO: Implement actual MXCFB_SEND_UPDATE ioctl
        // This is a placeholder for the actual implementation
        Ok(())
    }

    fn full_refresh(&self) -> Result<()> {
        // TODO: Implement full refresh via ioctl
        Ok(())
    }

    fn set_waveform_lut(&self, lut: &[u8]) -> Result<()> {
        if lut.is_empty() {
            anyhow::bail!("Waveform LUT cannot be empty");
        }

        // TODO: Implement EPDC waveform programming
        Ok(())
    }

    fn get_controller_name(&self) -> &str {
        "mxc-epdc"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sunxi_controller_creation() {
        let controller = SunxiController::new("/dev/disp_eink".to_string());
        assert!(controller.is_ok());
        assert!(controller.unwrap().device_path == "/dev/disp_eink");
    }

    #[test]
    fn test_sunxi_controller_empty_path() {
        let controller = SunxiController::new("".to_string());
        assert!(controller.is_err());
    }

    #[test]
    fn test_mxc_controller_creation() {
        let controller = MxcController::new("/dev/fb0".to_string());
        assert!(controller.is_ok());
    }

    #[test]
    fn test_controller_names() {
        let sunxi = SunxiController::default().unwrap();
        let mxc = MxcController::default().unwrap();
        assert_eq!(sunxi.get_controller_name(), "sunxi-disp2");
        assert_eq!(mxc.get_controller_name(), "mxc-epdc");
    }
}
