//! UART 控制器
//! 通过串口设备文件控制 UART 外设

use std::fs::OpenOptions;
use std::io::{Read, Write};

/// UART 控制错误类型
#[derive(Debug)]
pub enum UartError {
    IoError(std::io::Error),
}

impl From<std::io::Error> for UartError {
    fn from(err: std::io::Error) -> Self {
        UartError::IoError(err)
    }
}

/// UART 控制类
pub struct UartController {
    device: String,
}

impl UartController {
    /// 创建新的 UART 控制器实例
    pub fn new(device: &str) -> Self {
        UartController {
            device: device.to_string(),
        }
    }
    
    /// 通过 UART 发送数据
    pub fn write(&self, data: &[u8]) -> Result<(), UartError> {
        let mut file = OpenOptions::new()
            .write(true)
            .open(&self.device)?;
        file.write_all(data)?;
        Ok(())
    }
    
    /// 通过 UART 读取数据
    pub fn read(&self, buffer_size: usize) -> Result<Vec<u8>, UartError> {
        let mut file = OpenOptions::new()
            .read(true)
            .open(&self.device)?;
        let mut buffer = vec![0; buffer_size];
        let bytes_read = file.read(&mut buffer)?;
        buffer.truncate(bytes_read);
        Ok(buffer)
    }
}