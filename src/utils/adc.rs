//! ADC 控制器
//! 通过 IIO 子系统控制 ADC 外设

use std::fs;

/// ADC 控制错误类型
#[derive(Debug)]
pub enum AdcError {
    IoError(std::io::Error),
    ParseError(String),
}

impl From<std::io::Error> for AdcError {
    fn from(err: std::io::Error) -> Self {
        AdcError::IoError(err)
    }
}

/// ADC 控制类
pub struct AdcController {
    channel: u32,
}

impl AdcController {
    /// 创建新的 ADC 控制器实例
    pub fn new(channel: u32) -> Self {
        AdcController { channel }
    }
    
    /// 读取 ADC 值
    pub fn read_value(&self) -> Result<u32, AdcError> {
        let value_path = format!("/sys/bus/iio/devices/iio:device0/in_voltage{}_raw", self.channel);
        let value_str = fs::read_to_string(&value_path)?;
        let value = value_str
            .trim()
            .parse::<u32>()
            .map_err(|_| AdcError::ParseError("Failed to parse ADC value".to_string()))?;
        Ok(value)
    }
    
    /// 读取 ADC 电压 (毫伏)
    pub fn read_voltage(&self) -> Result<u32, AdcError> {
        let voltage_path = format!("/sys/bus/iio/devices/iio:device0/in_voltage{}_scale", self.channel);
        let scale_str = fs::read_to_string(&voltage_path)?;
        let scale = scale_str
            .trim()
            .parse::<f32>()
            .map_err(|_| AdcError::ParseError("Failed to parse ADC scale".to_string()))?;
        
        let raw_value = self.read_value()? as f32;
        let voltage = raw_value * scale;
        Ok(voltage as u32)
    }
}