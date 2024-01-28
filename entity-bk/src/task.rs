use super::options::Options;
use sea_orm::{entity::prelude::*, FromJsonQueryResult};
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, FromJsonQueryResult)]
pub enum TaskStatus {
    /// Task is running
    Running,

    /// Task is finished
    Succeeded,

    /// Task is failed
    Failed,

    /// Task is not started
    NotStarted,
}

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "task")]
pub struct Model {
    /// task id
    #[sea_orm(primary_key)]
    pub id: Uuid,

    /// openQASM 2.0 source code
    pub source: String,

    /// openQASM 2.0 simulation result
    pub result: Option<String>,

    /// openQASM 2.0 simulation options
    pub options: Options,

    /// task status
    pub status: TaskStatus,

    /// created time
    pub created_time: Time,

    /// updated time
    pub updated_time: Time,

    /// related resource id, foreign key
    pub resource_id: Option<Uuid>,

    /// related agent id, foreign key
    pub agent_id: Option<Uuid>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::resource::Entity",
        from = "Column::ResourceId",
        to = "super::resource::Column::Id"
    )]
    Resource,

    #[sea_orm(
        belongs_to = "super::agent::Entity",
        from = "Column::AgentId",
        to = "super::agent::Column::Id"
    )]
    Agent,
}

impl Related<super::resource::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Resource.def()
    }
}

impl Related<super::agent::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Agent.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
