use crate::comm::db::{Database, ResultSet};
use macros::SqlCRUD;

#[derive(Default)]
#[derive(Clone)]
#[derive(SqlCRUD)]
#[table_name = "crontask"]
// 员工信息表
pub struct Crontask {
    #[primary_key]
    pub id: i32,
    #[primary_key]
    //#[comment = "部门编号"]
    pub dept_id: i32,
    #[sql_type = "VARCHAR(55)"]
    pub name: String,
    //#[comment = "入职日期111"]
    #[sql_type = "VARCHAR(19)"]
    pub hire_date: String,
    pub active: bool,
}