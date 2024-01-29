use chrono::NaiveDateTime;
use sea_orm::prelude::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceStatus {
    Failed,
    Running,
    Succeeded,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Resource {
    pub id: Uuid,
    pub status: ResourceStatus,
    pub maximum_agents_num: i32,
    pub current_agents_num: i32,
    pub agent_ids: Vec<Uuid>,
    pub current_agent_ids: Vec<Uuid>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TaskStatus {
    Failed,
    NotStarted,
    Running,
    Succeeded,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Task {
    pub id: Uuid,
    pub source: String,
    pub result: Option<String>,
    pub option_id: Uuid,
    pub status: TaskStatus,
    pub created_time: NaiveDateTime,
    pub updated_time: NaiveDateTime,
    pub resource_id: Option<Uuid>,
    pub agent_id: Option<Uuid>,
}
