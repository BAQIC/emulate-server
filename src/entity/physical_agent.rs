//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.12

use super::sea_orm_active_enums::PhysicalAgentStatus;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "physical_agent")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub status: PhysicalAgentStatus,
    pub ip: String,
    pub port: i32,
    pub qubit_count: i32,
    pub qubit_idle: i32,
    pub circuit_depth: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::task_assignment::Entity")]
    TaskAssignment,
}

impl Related<super::task_assignment::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TaskAssignment.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
