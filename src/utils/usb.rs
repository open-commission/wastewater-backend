//! USB 控制器
//! 通过 sysfs 接口控制 USB 外设

use std::fs;
use std::path::Path;

/// USB 控制错误类型
#[derive(Debug)]
pub enum UsbError {
    IoError(std::io::Error),
}

impl From<std::io::Error> for UsbError {
    fn from(err: std::io::Error) -> Self {
        UsbError::IoError(err)
    }
}

/// USB 控制类
pub struct UsbController {
    device_path: String,
}

impl UsbController {
    /// 创建新的 USB 控制器实例
    pub fn new(device_path: &str) -> Self {
        UsbController {
            device_path: device_path.to_string(),
        }
    }
    
    /// 检查 USB 设备是否连接
    pub fn is_connected(&self) -> Result<bool, UsbError> {
        Ok(Path::new(&self.device_path).exists())
    }
    
    /// 获取 USB 设备信息
    pub fn get_device_info(&self) -> Result<String, UsbError> {
        let info_path = format!("{}/device/product", self.device_path);
        let info = fs::read_to_string(&info_path)?;
        Ok(info.trim().to_string())
    }
}