use super::db::Database;
use std::error::Error;
use std::fs;

// 创建一个core结构体，里面初始化一个全局的数据
pub struct Core {
    pub db: Database,
}

impl Core {
    // 返回Result类型以处理错误
    pub async fn new() -> Result<Self, Box<dyn Error>> {
        let db_path = "evdata.db";
        
        // 简化文件存在检查
        if !std::path::Path::new(db_path).exists() {
            fs::File::create(db_path)?; // 使用?自动转换错误类型
        }

        // 直接返回Result
        let mydb = Database::new(db_path).await?;
        
        Ok(Self { db: mydb })
    }
}