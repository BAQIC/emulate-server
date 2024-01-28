use super::options::Options;
use sea_orm::{entity::prelude::*, FromJsonQueryResult};
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromJsonQueryResult)]
pub enum AgentStatus {
    /// Agent is running.
    Running,

    /// Agent has finished.
    Succeeded,

    /// Agent has failed.
    Failed,
}

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "agent")]
pub struct Model {
    /// agent id
    #[sea_orm(primary_key)]
    pub id: Uuid,

    /// agent openQASM 2.0 source code
    pub source: String,

    /// openQASM 2.0 simulation result
    #[sea_orm(nullable)]
    pub result: Option<String>,

    /// openQASM 2.0 simulation options
    pub options: Options,

    /// agent status
    pub status: AgentStatus,

    /// related resource id, foreign key
    pub resource_id: u32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::resource::Entity",
        from = "Column::ResourceId",
        to = "super::resource::Column::Id"
    )]
    Resource,
}

impl Related<super::resource::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Resource.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
