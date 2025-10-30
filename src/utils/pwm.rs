//! PWM 控制器
//! 通过 sysfs 接口控制 PWM 外设

use std::fs::OpenOptions;
use std::io::Write;

/// PWM 控制错误类型
#[derive(Debug)]
pub enum PwmError {
    IoError(std::io::Error),
}

impl From<std::io::Error> for PwmError {
    fn from(err: std::io::Error) -> Self {
        PwmError::IoError(err)
    }
}

/// PWM 控制类
pub struct PwmController {
    chip: u32,
    channel: u32,
}

impl PwmController {
    /// 创建新的 PWM 控制器实例
    pub fn new(chip: u32, channel: u32) -> Self {
        PwmController { chip, channel }
    }
    
    /// 启用 PWM
    pub fn enable(&self) -> Result<(), PwmError> {
        let enable_path = format!("/sys/class/pwm/pwmchip{}/pwm{}/enable", self.chip, self.channel);
        let mut file = OpenOptions::new()
            .write(true)
            .open(&enable_path)?;
        file.write_all(b"1")?;
        Ok(())
    }
    
    /// 禁用 PWM
    pub fn disable(&self) -> Result<(), PwmError> {
        let enable_path = format!("/sys/class/pwm/pwmchip{}/pwm{}/enable", self.chip, self.channel);
        let mut file = OpenOptions::new()
            .write(true)
            .open(&enable_path)?;
        file.write_all(b"0")?;
        Ok(())
    }
    
    /// 设置 PWM 周期 (纳秒)
    pub fn set_period(&self, period: u32) -> Result<(), PwmError> {
        let period_path = format!("/sys/class/pwm/pwmchip{}/pwm{}/period", self.chip, self.channel);
        let mut file = OpenOptions::new()
            .write(true)
            .open(&period_path)?;
        file.write_all(period.to_string().as_bytes())?;
        Ok(())
    }
    
    /// 设置 PWM 占空比 (纳秒)
    pub fn set_duty_cycle(&self, duty_cycle: u32) -> Result<(), PwmError> {
        let duty_path = format!("/sys/class/pwm/pwmchip{}/pwm{}/duty_cycle", self.chip, self.channel);
        let mut file = OpenOptions::new()
            .write(true)
            .open(&duty_path)?;
        file.write_all(duty_cycle.to_string().as_bytes())?;
        Ok(())
    }
}