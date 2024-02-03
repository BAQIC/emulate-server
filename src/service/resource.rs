use super::{agent::Agent, agent::AgentStatus, task::Task};
use sea_orm::DbConn;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resource {
    pub physical_agents_num: u32,
    pub idle_agents_num: u32,
    pub physical_agents: HashMap<Uuid, AgentStatus>,
    pub idle_physical_agents: Vec<Uuid>,
}

impl Default for Resource {
    fn default() -> Self {
        Self {
            physical_agents_num: 0,
            idle_agents_num: 0,
            physical_agents: HashMap::new(),
            idle_physical_agents: Vec::new(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ResourceError {
    AgentNotIdle(Uuid),
    AgentNotExists(Uuid),
    ResourceDbError(sea_orm::prelude::DbErr),
}

impl std::fmt::Display for ResourceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResourceError::AgentNotIdle(id) => write!(f, "Agent {} is not idle", id),
            ResourceError::AgentNotExists(id) => write!(f, "Agent {} does not exist", id),
            ResourceError::ResourceDbError(err) => write!(f, "Resource db error: {}", err),
        }
    }
}

impl Resource {
    pub fn add_physical_agent(&mut self, agent_id: &Uuid) {
        self.physical_agents
            .insert(agent_id.clone(), AgentStatus::Idle);
        self.physical_agents_num += 1;
        self.idle_agents_num += 1;
        self.idle_physical_agents.push(agent_id.clone());
    }

    pub fn add_physical_agents(&mut self, agent_ids: Vec<Uuid>) {
        agent_ids.iter().for_each(|id| self.add_physical_agent(id));
    }

    pub fn remove_physical_agent(&mut self, agent_id: &Uuid) -> Result<(), ResourceError> {
        match self.physical_agents.remove(agent_id) {
            Some(AgentStatus::Idle) => {
                self.physical_agents_num -= 1;
                self.idle_agents_num -= 1;
                Ok(())
            }
            Some(_) => Err(ResourceError::AgentNotIdle(agent_id.clone())),
            None => Err(ResourceError::AgentNotExists(agent_id.clone())),
        }
    }

    pub fn get_idle_agent(&mut self) -> Option<Uuid> {
        self.idle_physical_agents.pop()
    }

    pub fn get_agent_status(&self, agent_id: &Uuid) -> Option<AgentStatus> {
        self.physical_agents.get(agent_id).cloned()
    }

    pub fn set_agent_status(
        &mut self,
        agent_id: &Uuid,
        status: AgentStatus,
    ) -> Result<(), ResourceError> {
        match self.physical_agents.get_mut(agent_id) {
            Some(s) => {
                *s = status;
                Ok(())
            }
            None => Err(ResourceError::AgentNotExists(agent_id.clone())),
        }
    }

    pub fn get_agents_num(&self) -> u32 {
        self.physical_agents_num
    }

    pub fn get_idle_agents_num(&self) -> u32 {
        self.idle_agents_num
    }

    pub fn get_agents(&self) -> Vec<Uuid> {
        self.physical_agents.keys().cloned().collect()
    }

    pub async fn submit_task(&mut self, task: &mut Task, db: &DbConn) -> Result<(), ResourceError> {
        match self.get_idle_agent() {
            Some(physical_agent_id) => {
                self.set_agent_status(&physical_agent_id, AgentStatus::Running)?;
                let agent = Agent::new(
                    physical_agent_id,
                    task.get_source().to_string(),
                    Some(task.get_option_id()),
                );
                task.set_agent_id(agent.get_agent_id());

                match (agent.insert_to_db(db).await, task.insert_to_db(db).await) {
                    (Ok(_), Ok(_)) => Ok(()),
                    (Err(err), _) => {
                        self.set_agent_status(&physical_agent_id, AgentStatus::Idle)?;
                        Err(ResourceError::ResourceDbError(err))
                    }
                    (_, Err(err)) => {
                        self.set_agent_status(&physical_agent_id, AgentStatus::Idle)?;
                        Err(ResourceError::ResourceDbError(err))
                    }
                }
            }
            None => Err(ResourceError::AgentNotIdle(Uuid::new_v4())),
        }
    }
}
