use sqlx::{sqlite::{SqlitePool, SqliteRow}, Row, Column};
use chrono::NaiveDateTime;
use std::collections::HashMap;

// 1. 定义参数枚举
#[derive(Debug)]
pub enum Param {
    I32(i32),
    I64(i64),
    String(String),
    Bool(bool),
    DateTime(NaiveDateTime),
    #[allow(dead_code)]
    Null,
    // 扩展其他类型...
}

// 2. 新增 Option 转换实现
// 新版写法
//stmt.set_param(None::<i32>);
//stmt.set_param(None::<&str>);
impl<T> From<Option<T>> for Param
where
    T: Into<Param>,
{
    fn from(opt: Option<T>) -> Self {
        match opt {
            Some(v) => v.into(),
            None => Param::Null,
        }
    }
}

// 2. 为常用类型实现自动转换到 Param
impl From<i32> for Param {
    fn from(v: i32) -> Self {
        Param::I32(v)
    }
}

impl From<i64> for Param {
    fn from(v: i64) -> Self {
        Param::I64(v)
    }
}

impl From<String> for Param {
    fn from(v: String) -> Self {
        Param::String(v)
    }
}

impl From<bool> for Param {
    fn from(v: bool) -> Self {
        Param::Bool(v)
    }
}

impl From<NaiveDateTime> for Param {
    fn from(v: NaiveDateTime) -> Self {
        Param::DateTime(v)
    }
}

pub struct Database {
    pool: SqlitePool,
}

pub struct Statement {
    sql: String,
    params: Vec<Param>, // 存储枚举类型
}

pub enum StatementResult {
    Update(u64),
    Query(ResultSet),
}

pub struct ResultSet {
    rows: Vec<SqliteRow>,
    columns: HashMap<String, usize>
}

// 行数据包装器
pub struct RowData<'a> {
    row: &'a SqliteRow,
    columns: &'a HashMap<String, usize>,
}

impl Database {
    pub async fn new(db_path: &str) -> Result<Self, sqlx::Error> {
        let pool = SqlitePool::connect(&format!("sqlite://{}", db_path)).await?;
        Ok(Database { pool })
    }

    pub fn open(&self, sql: &str) -> Statement {
        Statement {
            sql: sql.to_string(),
            params: Vec::new(),
        }
    }

    pub async fn execute(&self, stmt: Statement) -> Result<StatementResult, sqlx::Error> {
        let mut query = sqlx::query(&stmt.sql);
        
        // 通用参数绑定
        for param in stmt.params {
            match param {
                Param::I32(v) => query = query.bind(v),
                Param::I64(v) => query = query.bind(v),
                Param::String(v) => query = query.bind(v),
                Param::Bool(v) => query = query.bind(v),
                Param::DateTime(v) => query = query.bind(v.to_string()),
                Param::Null => query = query.bind(None::<i32>),
            }
        }

        // 根据SQL类型返回不同结果
        if is_select_query(&stmt.sql) {
            let rows = query.fetch_all(&self.pool).await?;
            let columns = get_columns(&rows);
            Ok(StatementResult::Query(ResultSet {
                rows,
                columns
            }))
        } else {
            let result = query.execute(&self.pool).await?;
            Ok(StatementResult::Update(result.rows_affected()))
        }
    }
}

impl Statement {
    // 3. 接受任何可转换为 Param 的类型
    pub fn set_param<T: Into<Param>>(mut self, param: T) -> Self {
        self.params.push(param.into());
        self
    }

    pub async fn exec(self, db: &Database) -> Result<u64, sqlx::Error> {
        match db.execute(self).await? {
            StatementResult::Update(affected_rows) => Ok(affected_rows),
            StatementResult::Query(_) => Err(sqlx::Error::Configuration(
                "Expected update statement".into()
            )),
        }
    }
    
    pub async fn query(self, db: &Database) -> Result<ResultSet, sqlx::Error> {
        match db.execute(self).await? {
            StatementResult::Query(rs) => Ok(rs),
            _ => Err(sqlx::Error::Configuration("Not a query statement".into()))
        }
    }
}

impl ResultSet {
    // 返回包含行数据和列信息的迭代器
    pub fn iter(&self) -> impl Iterator<Item = RowData> {
        self.rows.iter().map(|row| RowData {
            row,
            columns: &self.columns
        })
    }

    // 返回行数
    pub fn rows(&self) -> usize {
        self.rows.len()
    }
}


impl<'a> RowData<'a> {
    // 安全获取值的方法
    pub fn get<T: sqlx::Decode<'a, sqlx::Sqlite> + sqlx::Type<sqlx::Sqlite>>(&self, name: &str) -> Result<T, sqlx::Error> {
        let index = self.columns.get(name)
            .ok_or_else(|| sqlx::Error::ColumnNotFound(name.to_string()))?;
        
        self.row.try_get(*index)
    }
}

// 辅助函数
fn is_select_query(sql: &str) -> bool {
    sql.trim_start().to_uppercase().starts_with("SELECT")
}

fn get_columns(rows: &[SqliteRow]) -> HashMap<String, usize> {
    let mut columns = HashMap::new();
    if let Some(row) = rows.first() {
        for (idx, column) in row.columns().iter().enumerate() {
            columns.insert(column.name().to_string(), idx);
        }
    }
    columns
}