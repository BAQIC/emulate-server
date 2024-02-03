//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.12

use super::sea_orm_active_enums::TaskStatus;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "task")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub source: String,
    pub result: Option<String>,
    pub option_id: Uuid,
    pub status: TaskStatus,
    pub created_time: DateTime,
    pub updated_time: DateTime,
    pub agent_id: Option<Uuid>,
    pub physical_agent_id: Option<Uuid>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::agent::Entity",
        from = "Column::AgentId",
        to = "super::agent::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Agent,
    #[sea_orm(
        belongs_to = "super::options::Entity",
        from = "Column::OptionId",
        to = "super::options::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Options,
}

impl Related<super::agent::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Agent.def()
    }
}

impl Related<super::options::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Options.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
