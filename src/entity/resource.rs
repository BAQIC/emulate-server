//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.12

use super::sea_orm_active_enums::ResourceStatus;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "resource")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub status: ResourceStatus,
    pub maximum_agents_num: i32,
    pub current_agents_num: i32,
    pub agent_ids: Vec<Uuid>,
    pub current_agent_ids: Vec<Uuid>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::task::Entity")]
    Task,
}

impl Related<super::task::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Task.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
