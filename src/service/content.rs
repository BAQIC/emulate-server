use chrono::NaiveDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QthreadResource {
    pub resource_id: Uuid,
    pub quota: u32,
    pub current_quota: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Qthread {
    /// all resources
    pub resources: Vec<QthreadResource>,

    /// all running tasks
    pub running_tasks: Vec<Uuid>,

    /// all finished tasks
    pub finished_tasks: Vec<Uuid>,

    /// all tasks queued
    pub queued_tasks: Vec<Uuid>,
}

impl Default for Qthread {
    fn default() -> Self {
        Self {
            resources: vec![],
            running_tasks: vec![],
            finished_tasks: vec![],
            queued_tasks: vec![],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceStatus {
    FullyUsed,
    PartiallyUsed,
    Paused,
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

impl Default for Resource {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            status: ResourceStatus::PartiallyUsed,
            maximum_agents_num: 0,
            current_agents_num: 0,
            agent_ids: vec![],
            current_agent_ids: vec![],
        }
    }
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
