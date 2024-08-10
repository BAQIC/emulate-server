//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.12

use super::sea_orm_active_enums::TaskActiveStatus;
use super::sea_orm_active_enums::TaskMode;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "task_active")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub source: String,
    pub result: Option<String>,
    pub qubits: i32,
    pub shots: i32,
    pub exec_shots: i32,
    pub v_exec_shots: i32,
    pub depth: i32,
    pub status: TaskActiveStatus,
    pub mode: Option<TaskMode>,
    pub created_time: DateTime,
    pub updated_time: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
