mod comm;
mod tables;

use comm::core::Core;
use tables::crontask::Crontask;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let core = Core::new().await?;

    // 初始化表结构
    Crontask::init(&core.db).await?;

    let crontask = Crontask {
        id: 12,
        dept_id: 1,
        name: "John Doe".to_string(),
        hire_date: "2025-03-07 17:53:12".to_string(),
        active: true,
    };
    
    // 修改后（显式打印错误）
    if let Err(e) = crontask.insert(&core.db).await {
        eprintln!("[ERROR] 定时任务插入失败: {}", e);
    }

    // 字符串长度
    //println!("{}, {}", Crontask::create_table_sql(), crontask.hire_date.len());

    //let rs = crontask.query(&core.db).await?;
    let rs = core.db.open("select * from crontask where id >= ?").set_param(1).query(&core.db).await?;

    for row_data in rs.iter() {
        let id: i32 = row_data.get("id")?;
        let name: String = row_data.get("name")?;
        
        println!("crontask {}: {}    {}", id, name, rs.rows());
    }
    Ok(())
}