// Cargo.toml 依赖
// [dependencies]
// tokio = { version = "1.29", features = ["full"] }
// tokio-modbus = "0.7"
// tokio-serial = "5.6"
// anyhow = "1.0"

use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::Duration,
};

use tokio_modbus::prelude::*;
use tokio_modbus::server::{tcp::Server as TcpServer, rtu::Server as RtuServer};
use tokio_serial::{SerialStream, SerialPortBuilderExt};

/// 通用 Modbus 工具类，支持 TCP/RTU，主/从机
pub struct ModbusClient {
    tcp_addr: Option<SocketAddr>,
    rtu_path: Option<String>,
    slave: Option<Slave>,
}

impl ModbusClient {
    /// 创建 TCP 客户端
    pub fn new_tcp(addr: &str) -> Self {
        let tcp_addr = addr.parse().ok();
        Self {
            tcp_addr,
            rtu_path: None,
            slave: None,
        }
    }

    /// 创建 RTU 客户端
    pub fn new_rtu(path: &str, slave_id: u8) -> Self {
        Self {
            tcp_addr: None,
            rtu_path: Some(path.to_string()),
            slave: Some(Slave(slave_id)),
        }
    }

    /// 异步读取保持寄存器
    pub async fn read_holding(&self, addr: u16, count: u16) -> anyhow::Result<Vec<u16>> {
        if let Some(tcp) = self.tcp_addr {
            let mut ctx = tcp::connect(tcp).await?;
            let data = ctx.read_holding_registers(addr, count).await??;
            ctx.disconnect().await?;
            Ok(data)
        } else if let Some(ref path) = self.rtu_path {
            let slave = self.slave.unwrap();
            let builder = tokio_serial::new(path, 19200);
            let port = SerialStream::open(&builder)?;
            let mut ctx = rtu::attach_slave(port, slave);
            let data = ctx.read_holding_registers(addr, count).await??;
            ctx.disconnect().await?;
            Ok(data)
        } else {
            anyhow::bail!("No connection method defined");
        }
    }

    /// 异步写单个寄存器
    pub async fn write_holding(&self, addr: u16, value: u16) -> anyhow::Result<()> {
        if let Some(tcp) = self.tcp_addr {
            let mut ctx = tcp::connect(tcp).await?;
            ctx.write_single_register(addr, value).await??;
            ctx.disconnect().await?;
            Ok(())
        } else if let Some(ref path) = self.rtu_path {
            let slave = self.slave.unwrap();
            let builder = tokio_serial::new(path, 19200);
            let port = SerialStream::open(&builder)?;
            let mut ctx = rtu::attach_slave(port, slave);
            ctx.write_single_register(addr, value).await??;
            ctx.disconnect().await?;
            Ok(())
        } else {
            anyhow::bail!("No connection method defined");
        }
    }
}

/// Modbus 从机服务端实现，通用 TCP/RTU
pub struct ModbusServer {
    input_registers: Arc<Mutex<HashMap<u16, u16>>>,
    holding_registers: Arc<Mutex<HashMap<u16, u16>>>,
}

impl ModbusServer {
    pub fn new() -> Self {
        let mut input = HashMap::new();
        input.insert(0, 1234);
        input.insert(1, 5678);

        let mut holding = HashMap::new();
        holding.insert(0, 10);
        holding.insert(1, 20);
        holding.insert(2, 30);
        holding.insert(3, 40);

        Self {
            input_registers: Arc::new(Mutex::new(input)),
            holding_registers: Arc::new(Mutex::new(holding)),
        }
    }

    /// TCP 服务端启动
    pub async fn serve_tcp(&self, addr: &str) -> anyhow::Result<()> {
        let socket_addr: SocketAddr = addr.parse()?;
        let listener = tokio::net::TcpListener::bind(socket_addr).await?;
        let server = TcpServer::new(listener);

        let new_service = |_socket_addr| Ok(Some(self.clone()));
        let on_connected = |stream, socket_addr| async move {
            tokio_modbus::server::tcp::accept_tcp_connection(stream, socket_addr, new_service)
        };
        let on_error = |err| eprintln!("{err}");
        server.serve(&on_connected, on_error).await?;
        Ok(())
    }

    /// RTU 服务端启动
    pub async fn serve_rtu(&self, path: &str, slave_id: u8) -> anyhow::Result<()> {
        let builder = tokio_serial::new(path, 19200);
        let port = SerialStream::open(&builder)?;
        let server = RtuServer::new(port);
        let service = RtuService { slave_id, server: self.clone() };
        server.serve_forever(service).await?;
        Ok(())
    }
}

impl Clone for ModbusServer {
    fn clone(&self) -> Self {
        Self {
            input_registers: Arc::clone(&self.input_registers),
            holding_registers: Arc::clone(&self.holding_registers),
        }
    }
}

impl tokio_modbus::server::Service for ModbusServer {
    type Request = Request<'static>;
    type Response = Response;
    type Exception = ExceptionCode;
    type Future = std::future::Ready<Result<Self::Response, Self::Exception>>;

    fn call(&self, req: Self::Request) -> Self::Future {
        match req {
            Request::ReadInputRegisters(addr, cnt) => {
                let data = register_read(&self.input_registers.lock().unwrap(), addr, cnt);
                std::future::ready(data.map(Response::ReadInputRegisters))
            }
            Request::ReadHoldingRegisters(addr, cnt) => {
                let data = register_read(&self.holding_registers.lock().unwrap(), addr, cnt);
                std::future::ready(data.map(Response::ReadHoldingRegisters))
            }
            Request::WriteSingleRegister(addr, val) => {
                let data = register_write(&mut self.holding_registers.lock().unwrap(), addr, &[val]);
                std::future::ready(data.map(|_| Response::WriteSingleRegister(addr, val)))
            }
            Request::WriteMultipleRegisters(addr, vals) => {
                let data = register_write(&mut self.holding_registers.lock().unwrap(), addr, &vals);
                std::future::ready(data.map(|_| Response::WriteMultipleRegisters(addr, vals.len() as u16)))
            }
            _ => std::future::ready(Err(ExceptionCode::IllegalFunction)),
        }
    }
}

/// RTU 服务具体处理
struct RtuService {
    slave_id: u8,
    server: ModbusServer,
}

impl tokio_modbus::server::Service for RtuService {
    type Request = SlaveRequest<'static>;
    type Response = Response;
    type Exception = ExceptionCode;
    type Future = std::future::Ready<Result<Self::Response, Self::Exception>>;

    fn call(&self, req: Self::Request) -> Self::Future {
        use tokio_modbus::server::Service;
        if req.slave != self.slave_id {
            return std::future::ready(Err(ExceptionCode::IllegalFunction));
        }
        match req.request {
            Request::ReadInputRegisters(addr, cnt) => {
                let data = register_read(&self.server.input_registers.lock().unwrap(), addr, cnt);
                std::future::ready(data.map(Response::ReadInputRegisters))
            }
            Request::ReadHoldingRegisters(addr, cnt) => {
                let data = register_read(&self.server.holding_registers.lock().unwrap(), addr, cnt);
                std::future::ready(data.map(Response::ReadHoldingRegisters))
            }
            Request::WriteSingleRegister(addr, val) => {
                let data = register_write(&mut self.server.holding_registers.lock().unwrap(), addr, &[val]);
                std::future::ready(data.map(|_| Response::WriteSingleRegister(addr, val)))
            }
            Request::WriteMultipleRegisters(addr, vals) => {
                let data = register_write(&mut self.server.holding_registers.lock().unwrap(), addr, &vals);
                std::future::ready(data.map(|_| Response::WriteMultipleRegisters(addr, vals.len() as u16)))
            }
            _ => std::future::ready(Err(ExceptionCode::IllegalFunction)),
        }
    }
}

/// 通用寄存器读写
fn register_read(registers: &HashMap<u16, u16>, addr: u16, cnt: u16) -> Result<Vec<u16>, ExceptionCode> {
    let mut res = vec![0; cnt.into()];
    for i in 0..cnt {
        let r_addr = addr + i;
        res[i as usize] = *registers.get(&r_addr).ok_or(ExceptionCode::IllegalDataAddress)?;
    }
    Ok(res)
}

fn register_write(registers: &mut HashMap<u16, u16>, addr: u16, values: &[u16]) -> Result<(), ExceptionCode> {
    for (i, &val) in values.iter().enumerate() {
        let r_addr = addr + i as u16;
        let r = registers.get_mut(&r_addr).ok_or(ExceptionCode::IllegalDataAddress)?;
        *r = val;
    }
    Ok(())
}
