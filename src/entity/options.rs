//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.12

use super::sea_orm_active_enums::AgentType;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "options")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub agent_type: AgentType,
    pub shots: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::agent::Entity")]
    Agent,
    #[sea_orm(has_many = "super::task::Entity")]
    Task,
}

impl Related<super::agent::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Agent.def()
    }
}

impl Related<super::task::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Task.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
