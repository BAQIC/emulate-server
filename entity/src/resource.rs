use sea_orm::{entity::prelude::*, FromJsonQueryResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromJsonQueryResult)]
pub enum ResourceStatus {
    /// All agents are running
    FullyUsed,

    /// Some agents are running
    PartiallyUsed,

    /// This resource is paused, which means can not be allocated to any agent
    Paused,
}

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "resource")]
pub struct Model {
    /// resource id
    #[sea_orm(primary_key)]
    pub id: Uuid,

    /// resource status
    pub status: ResourceStatus,

    /// maximum runing agents number
    pub maximum_agents_num: u32,

    /// current running agents number
    pub current_agents_num: u32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::agent::Entity")]
    Agent,
}

// `Related` trait has to be implemented by hand
impl Related<super::agent::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Agent.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
