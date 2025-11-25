use crate::database::sea_orm_db::DbManager;
use crate::models::device;
use sea_orm::{EntityTrait, Set, Schema, ConnectionTrait};

pub async fn run_sea_orm_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("开始运行SeaORM示例...");

    // 创建数据库连接
    let db_manager = DbManager::new("sqlite://guolu.db?mode=rwc").await?;
    let db = db_manager.get_connection();
    
    // 创建表
    println!("创建设备表...");
    let builder = db.get_database_backend();
    let schema = Schema::new(builder);
    let create_table_statement = schema.create_table_from_entity(device::Entity);
    
    // 执行创建表语句，忽略表已存在的错误
    match db.execute(builder.build(&create_table_statement)).await {
        Ok(_) => println!("设备表创建成功"),
        Err(e) => {
            println!("创建表时出现错误（可能是表已存在）: {}", e);
            println!("继续执行后续操作...");
        }
    }
    
    // 插入一些示例数据
    let device1 = device::ActiveModel {
        name: Set("锅炉1".to_string()),
        location: Set("车间A".to_string()),
        status: Set(1), // 1表示运行中
        ..Default::default()
    };
    
    let device2 = device::ActiveModel {
        name: Set("锅炉2".to_string()),
        location: Set("车间B".to_string()),
        status: Set(0), // 0表示停止
        ..Default::default()
    };
    
    println!("插入设备数据...");
    let insert_result1 = device::Entity::insert(device1).exec(db).await?;
    let insert_result2 = device::Entity::insert(device2).exec(db).await?;
    
    println!("插入的设备ID: {}, {}", insert_result1.last_insert_id, insert_result2.last_insert_id);
    
    // 查询所有设备
    println!("查询所有设备...");
    let devices = device::Entity::find().all(db).await?;
    for device in &devices {
        println!("设备: {} 位置: {} 状态: {}", device.name, device.location, 
                 if device.status == 1 { "运行中" } else { "停止" });
    }
    
    // 更新设备状态
    println!("更新设备状态...");
    let mut device_to_update: device::ActiveModel = devices[0].clone().into();
    device_to_update.status = Set(0); // 设置为停止状态
    let _update_result = device::Entity::update(device_to_update).exec(db).await?;
    println!("设备状态更新完成");
    
    // 查询特定设备
    println!("查询特定设备...");
    if let Some(device) = device::Entity::find_by_id(insert_result1.last_insert_id).one(db).await? {
        println!("找到设备: {} 状态: {}", device.name, 
                 if device.status == 1 { "运行中" } else { "停止" });
    }
    
    println!("SeaORM示例运行完成!");
    Ok(())
}