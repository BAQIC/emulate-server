use super::options::Options;
use super::resource::Resource;
use super::task::Task;
use sea_orm::DbConn;
use std::{collections::VecDeque, error::Error};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Qthread {
    /// all resources
    pub resource: Resource,

    /// queue of waitting tasks
    pub tasks: VecDeque<Uuid>,
}

impl Default for Qthread {
    fn default() -> Self {
        Self {
            resource: Resource::default(),
            tasks: VecDeque::new(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum EmulateError {
    AgentNotIdle(Uuid),
    AgentNotExists(Uuid),
    ResourceDbError(sea_orm::prelude::DbErr),
    QasmSimError(String),
}

impl Error for EmulateError {}

impl std::fmt::Display for EmulateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EmulateError::AgentNotIdle(id) => write!(f, "Agent {} is not idle", id),
            EmulateError::AgentNotExists(id) => write!(f, "Agent {} does not exist", id),
            EmulateError::ResourceDbError(err) => write!(f, "Resource db error: {}", err),
            EmulateError::QasmSimError(err) => write!(f, "QasmSim error: {}", err),
        }
    }
}

impl Qthread {
    pub fn new(physical_agents_num: u32) -> Self {
        Self {
            resource: Resource::new(physical_agents_num),
            tasks: VecDeque::new(),
        }
    }

    pub async fn add_task(
        &mut self,
        source: &str,
        options: &qasmsim::options::Options,
        db: &DbConn,
    ) -> Result<Uuid, EmulateError> {
        let option = Options::new(&options);
        let mut task = Task::new(source.to_string(), option.id);

        match (option.insert_to_db(db).await, task.insert_to_db(db).await) {
            (Ok(_), Ok(_)) => (),
            (Err(err), _) => return Err(EmulateError::ResourceDbError(err)),
            (_, Err(err)) => return Err(EmulateError::ResourceDbError(err)),
        }

        if self.resource.idle_agents_num > 0 {
            self.resource.submit_task(&mut task, db).await?;
        } else {
        }

        Ok(task.get_id())
    }
}
