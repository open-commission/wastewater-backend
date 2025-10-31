//! Modbus 使用示例
//!
//! 本示例演示了如何使用 Modbus 客户端和服务器功能

use crate::utils::modbus::{ModbusClient, ModbusServer};
use std::time::Duration;
use tokio::time::sleep;
use anyhow::Result;

/// 启动 Modbus TCP 服务器的示例
pub async fn run_tcp_server() -> Result<()> {
    println!("启动 Modbus TCP 服务器...");

    // 创建服务器实例
    let server = ModbusServer::new();

    // 在后台任务中启动服务器
    let server_task = tokio::spawn(async move {
        if let Err(e) = server.serve_tcp("127.0.0.1:5020").await {
            eprintln!("TCP 服务器错误: {}", e);
        }
    });

    // 等待服务器启动
    sleep(Duration::from_millis(100)).await;

    // 创建客户端并连接到服务器
    let client = ModbusClient::new_tcp("127.0.0.1:5020");

    // 读取保持寄存器
    println!("读取保持寄存器...");
    match client.read_holding(0, 4).await {
        Ok(values) => {
            println!("读取到的保持寄存器值: {:?}", values);
        }
        Err(e) => {
            eprintln!("读取保持寄存器失败: {}", e);
        }
    }

    // 写入单个保持寄存器
    println!("写入保持寄存器地址 0 的值为 100...");
    match client.write_holding(0, 100).await {
        Ok(()) => {
            println!("写入成功");
        }
        Err(e) => {
            eprintln!("写入失败: {}", e);
        }
    }

    // 再次读取以验证写入
    println!("再次读取保持寄存器以验证写入...");
    match client.read_holding(0, 4).await {
        Ok(values) => {
            println!("写入后读取到的保持寄存器值: {:?}", values);
        }
        Err(e) => {
            eprintln!("读取保持寄存器失败: {}", e);
        }
    }

    // 取消服务器任务
    server_task.abort();

    Ok(())
}

/// 启动 Modbus RTU 服务器的示例（需要实际串口设备）
/// 
/// 注意：此示例需要实际的串口设备才能运行
#[allow(dead_code)]
pub async fn run_rtu_server_example() -> Result<()> {
    println!("启动 Modbus RTU 服务器示例...");
    println!("注意：此示例需要实际的串口设备才能运行");
    println!("例如，在 Windows 上可能是 \"COM1\"，在 Linux 上可能是 \"/dev/ttyUSB0\"");

    // 创建服务器实例
    let server = ModbusServer::new();

    // 启动 RTU 服务器（这里使用示例端口，实际使用时需要根据系统情况修改）
    // 注意：在实际使用时，需要将 "COM2" 或 "/dev/ttyUSB1" 替换为实际可用的串口设备
    println!("尝试在 COM2 上启动 RTU 服务器（如需运行请取消注释并修改端口）...");
    
    // 为了演示目的，我们注释掉实际的服务器启动代码，因为需要真实的串口设备
    // 如果你有可用的串口设备，可以取消注释并修改端口名称
    let server_task = tokio::spawn(async move {
        if let Err(e) = server.serve_rtu("COM2", 1).await {  // 根据实际情况修改端口
            eprintln!("RTU 服务器错误: {}", e);
        }
    });
    
    // 等待服务器启动
    sleep(Duration::from_millis(100)).await;
    println!("RTU 服务器已启动");
    
    // 创建 RTU 客户端并连接到服务器
    let client = ModbusClient::new_rtu("COM2", 1);  // 使用相同的端口和从机地址
    
    // 读取保持寄存器
    println!("读取 RTU 保持寄存器...");
    match client.read_holding(0, 4).await {
        Ok(values) => {
            println!("RTU 读取到的保持寄存器值: {:?}", values);
        }
        Err(e) => {
            eprintln!("RTU 读取保持寄存器失败: {}", e);
        }
    }
    
    // 写入单个保持寄存器
    println!("RTU 写入保持寄存器地址 0 的值为 150...");
    match client.write_holding(0, 150).await {
        Ok(()) => {
            println!("RTU 写入成功");
        }
        Err(e) => {
            eprintln!("RTU 写入失败: {}", e);
        }
    }
    
    // 再次读取以验证写入
    println!("RTU 再次读取保持寄存器以验证写入...");
    match client.read_holding(0, 4).await {
        Ok(values) => {
            println!("RTU 写入后读取到的保持寄存器值: {:?}", values);
        }
        Err(e) => {
            eprintln!("RTU 读取保持寄存器失败: {}", e);
        }
    }
    
    // 取消服务器任务
    server_task.abort();

    Ok(())
}

/// Modbus RTU 客户端使用示例（需要实际串口设备）
///
/// 注意：此示例需要实际的串口设备才能运行
#[allow(dead_code)]
pub async fn run_rtu_client_example() -> Result<()> {
    println!("Modbus RTU 客户端示例...");
    println!("注意：此示例需要实际的串口设备才能运行");

    // 创建 RTU 客户端（需要实际串口设备）
    // Windows 上可能是 "COM1", Linux 上可能是 "/dev/ttyUSB0"
    // 注意：根据你的系统情况修改端口名称和从机地址
    println!("尝试连接到 COM1 端口的从机地址 1（如需运行请取消注释并修改端口）...");
    
    // 为了演示目的，我们注释掉实际的客户端连接代码，因为需要真实的串口设备
    // 如果你有可用的串口设备，可以取消注释并修改端口名称
    let client = ModbusClient::new_rtu("COM1", 1);  // 根据实际情况修改端口和从机地址

    // 读取保持寄存器
    match client.read_holding(0, 10).await {
        Ok(values) => {
            println!("RTU 读取到的寄存器值: {:?}", values);
        }
        Err(e) => {
            eprintln!("RTU 读取失败: {}", e);
        }
    }
    
    // 写入单个保持寄存器
    println!("RTU 写入保持寄存器地址 0 的值为 200...");
    match client.write_holding(0, 200).await {
        Ok(()) => {
            println!("RTU 写入成功");
        }
        Err(e) => {
            eprintln!("RTU 写入失败: {}", e);
        }
    }
    
    // 读取输入寄存器（通过服务端内部数据）
    // 注意：输入寄存器只能读取，不能写入
    println!("RTU 读取输入寄存器...");
    match client.read_holding(0, 2).await {  // 服务端默认输入寄存器地址0=1234，地址1=5678
        Ok(values) => {
            println!("RTU 读取到的输入寄存器值: {:?}", values);
        }
        Err(e) => {
            eprintln!("RTU 读取输入寄存器失败: {}", e);
        }
    }

    Ok(())
}

/// 主函数 - 运行 Modbus 示例
pub async fn run_examples() -> Result<()> {
    println!("=== Modbus 使用示例 ===\n");

    // 运行 TCP 服务器示例
    if let Err(e) = run_tcp_server().await {
        eprintln!("TCP 服务器示例失败: {}", e);
    }
    
    // 显示 RTU 示例说明
    println!("\n=== Modbus RTU 示例说明 ===");
    run_rtu_server_example().await?;
    run_rtu_client_example().await?;
    println!("RTU 示例需要实际的串口设备才能运行");
    println!("请根据你的系统情况修改代码中的端口名称和从机地址");

    println!("\n=== 示例结束 ===");
    Ok(())
}