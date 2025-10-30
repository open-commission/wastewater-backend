//! GPIO 控制器
//! 通过 sysfs 接口控制 GPIO 外设

use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::Path;

/// GPIO 控制错误类型
#[derive(Debug)]
pub enum GpioError {
    IoError(std::io::Error),
    ParseError(String),
}

impl From<std::io::Error> for GpioError {
    fn from(err: std::io::Error) -> Self {
        GpioError::IoError(err)
    }
}

/// GPIO 控制类
pub struct GpioController {
    pin: u32,
    exported: bool,
}

impl GpioController {
    /// 创建新的 GPIO 控制器实例
    pub fn new(pin: u32) -> Result<Self, GpioError> {
        let mut controller = GpioController {
            pin,
            exported: false,
        };
        
        // 检查 GPIO 是否已经导出
        let gpio_path = format!("/sys/class/gpio/gpio{}", pin);
        if Path::new(&gpio_path).exists() {
            controller.exported = true;
        }
        
        Ok(controller)
    }
    
    /// 导出 GPIO
    pub fn export(&mut self) -> Result<(), GpioError> {
        if !self.exported {
            let mut file = OpenOptions::new()
                .write(true)
                .open("/sys/class/gpio/export")?;
            file.write_all(self.pin.to_string().as_bytes())?;
            self.exported = true;
        }
        Ok(())
    }
    
    /// 取消导出 GPIO
    pub fn unexport(&mut self) -> Result<(), GpioError> {
        if self.exported {
            let mut file = OpenOptions::new()
                .write(true)
                .open("/sys/class/gpio/unexport")?;
            file.write_all(self.pin.to_string().as_bytes())?;
            self.exported = false;
        }
        Ok(())
    }
    
    /// 设置 GPIO 方向 (in/out)
    pub fn set_direction(&mut self, direction: &str) -> Result<(), GpioError> {
        self.export()?;
        let direction_path = format!("/sys/class/gpio/gpio{}/direction", self.pin);
        let mut file = OpenOptions::new()
            .write(true)
            .open(&direction_path)?;
        file.write_all(direction.as_bytes())?;
        Ok(())
    }
    
    /// 设置 GPIO 值 (0/1)
    pub fn set_value(&mut self, value: u8) -> Result<(), GpioError> {
        self.export()?;
        let value_path = format!("/sys/class/gpio/gpio{}/value", self.pin);
        let mut file = OpenOptions::new()
            .write(true)
            .open(&value_path)?;
        file.write_all(value.to_string().as_bytes())?;
        Ok(())
    }
    
    /// 读取 GPIO 值
    pub fn get_value(&mut self) -> Result<u8, GpioError> {
        self.export()?;
        let value_path = format!("/sys/class/gpio/gpio{}/value", self.pin);
        let mut file = OpenOptions::new()
            .read(true)
            .open(&value_path)?;
        let mut buffer = [0; 1];
        file.read_exact(&mut buffer)?;
        let value = buffer[0]
            .to_string()
            .parse::<u8>()
            .map_err(|_| GpioError::ParseError("Failed to parse GPIO value".to_string()))?;
        Ok(value)
    }
}

impl Drop for GpioController {
    fn drop(&mut self) {
        // 自动取消导出 GPIO
        let _ = self.unexport();
    }
}