//! Display controller abstraction for e-ink devices
//!
//! Provides a trait abstraction for different e-ink display controllers.

use anyhow::Result;
use crate::geom::Rectangle;
use crate::eink::waveform::WaveformMode;
use crate::eink::damage_tracker::RefreshStrategy;

/// Trait for e-ink display controllers
pub trait EInkController {
    fn update(&self, region: Rectangle, data: &[u8], waveform: WaveformMode) -> Result<()>;
    fn full_refresh(&self) -> Result<()>;
    fn set_waveform_lut(&self, lut: &[u8]) -> Result<()>;
    fn get_controller_name(&self) -> &str;

    /// Perform optimized refresh based on damage strategy
    fn optimized_refresh(
        &self,
        strategy: &RefreshStrategy,
        full_data: &[u8],
        waveform: WaveformMode,
    ) -> Result<()> {
        match strategy {
            RefreshStrategy::None => Ok(()),
            RefreshStrategy::Full => self.full_refresh(),
            RefreshStrategy::Partial(regions) => {
                self.partial_refresh(regions, full_data, waveform)
            }
        }
    }

    /// Perform partial refresh for multiple regions
    fn partial_refresh(
        &self,
        regions: &[Rectangle],
        full_data: &[u8],
        waveform: WaveformMode,
    ) -> Result<()> {
        if regions.is_empty() {
            return Ok(());
        }

        // Validate regions before processing
        for region in regions {
            if region.min.x < 0 || region.min.y < 0 {
                anyhow::bail!("Invalid region: negative coordinates");
            }
            if region.max.x <= region.min.x || region.max.y <= region.min.y {
                anyhow::bail!("Invalid region: zero or negative size");
            }
        }

        // Extract data for each region and update
        for region in regions {
            let region_data = self.extract_region_data(region, full_data)?;
            self.update(*region, &region_data, waveform)?;
        }

        Ok(())
    }

    /// Extract pixel data for a specific region from full framebuffer
    fn extract_region_data(&self, region: &Rectangle, full_data: &[u8]) -> Result<Vec<u8>> {
        let width = (region.max.x - region.min.x) as u32;
        let height = (region.max.y - region.min.y) as u32;
        let region_size = (width * height * 4) as usize;

        if region_size > full_data.len() {
            anyhow::bail!(
                "Region size {} exceeds full data size {}",
                region_size,
                full_data.len()
            );
        }

        let mut region_data = Vec::with_capacity(region_size);
        let fb_width = (region.max.x as f32).ceil() as u32;

        for y in 0..height {
            let y_offset = (region.min.y as u32 + y) * fb_width * 4;
            let x_offset = region.min.x as u32 * 4;
            let start = (y_offset + x_offset) as usize;
            let end = start + (width * 4) as usize;

            if end > full_data.len() {
                anyhow::bail!("Region data extraction out of bounds");
            }

            region_data.extend_from_slice(&full_data[start..end]);
        }

        Ok(region_data)
    }
}

/// Sunxi disp2 controller for Allwinner-based Kobo devices (Elipsa, Sage)
#[derive(Debug)]
pub struct SunxiController {
    _device_path: String,
}

impl SunxiController {
    pub fn new(device_path: String) -> Result<Self> {
        if device_path.is_empty() {
            anyhow::bail!("Device path cannot be empty");
        }
        Ok(Self { _device_path: device_path })
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
    _device_path: String,
}

impl MxcController {
    pub fn new(device_path: String) -> Result<Self> {
        if device_path.is_empty() {
            anyhow::bail!("Device path cannot be empty");
        }
        Ok(Self { _device_path: device_path })
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
        // Controller name is verified via get_controller_name
        assert_eq!(controller.unwrap().get_controller_name(), "sunxi-disp2");
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
