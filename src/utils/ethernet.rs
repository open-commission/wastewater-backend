//! Ethernet 控制器
//! 通过 sysfs 接口控制以太网外设

use std::fs;

/// Ethernet 控制错误类型
#[derive(Debug)]
pub enum EthernetError {
    IoError(std::io::Error),
    ParseError(String),
}

impl From<std::io::Error> for EthernetError {
    fn from(err: std::io::Error) -> Self {
        EthernetError::IoError(err)
    }
}

/// Ethernet 控制类
pub struct EthernetController {
    interface: String,
}

impl EthernetController {
    /// 创建新的以太网控制器实例
    pub fn new(interface: &str) -> Self {
        EthernetController {
            interface: interface.to_string(),
        }
    }
    
    /// 获取网络接口状态
    pub fn get_status(&self) -> Result<String, EthernetError> {
        let status_path = format!("/sys/class/net/{}/operstate", self.interface);
        let status = fs::read_to_string(&status_path)?;
        Ok(status.trim().to_string())
    }
    
    /// 获取接收字节数
    pub fn get_rx_bytes(&self) -> Result<u64, EthernetError> {
        let stat_path = format!("/sys/class/net/{}/statistics/rx_bytes", self.interface);
        let stat_str = fs::read_to_string(&stat_path)?;
        let stat = stat_str
            .trim()
            .parse::<u64>()
            .map_err(|_| EthernetError::ParseError("Failed to parse rx_bytes".to_string()))?;
        Ok(stat)
    }
    
    /// 获取发送字节数
    pub fn get_tx_bytes(&self) -> Result<u64, EthernetError> {
        let stat_path = format!("/sys/class/net/{}/statistics/tx_bytes", self.interface);
        let stat_str = fs::read_to_string(&stat_path)?;
        let stat = stat_str
            .trim()
            .parse::<u64>()
            .map_err(|_| EthernetError::ParseError("Failed to parse tx_bytes".to_string()))?;
        Ok(stat)
    }
}