use super::{agent::Agent, agent::AgentStatus, qthread::EmulateError, task::Task};
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

impl Resource {
    pub fn new(physical_agents_num: u32) -> Self {
        let mut resource = Resource::default();
        for _ in 0..physical_agents_num {
            resource.add_physical_agent(&Uuid::new_v4());
        }
        resource
    }

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

    pub fn remove_physical_agent(&mut self, agent_id: &Uuid) -> Result<(), EmulateError> {
        match self.physical_agents.remove(agent_id) {
            Some(AgentStatus::Idle) => {
                self.physical_agents_num -= 1;
                self.idle_agents_num -= 1;
                Ok(())
            }
            Some(_) => Err(EmulateError::AgentNotIdle(agent_id.clone())),
            None => Err(EmulateError::AgentNotExists(agent_id.clone())),
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
    ) -> Result<(), EmulateError> {
        match self.physical_agents.get_mut(agent_id) {
            Some(s) => {
                *s = status;
                Ok(())
            }
            None => Err(EmulateError::AgentNotExists(agent_id.clone())),
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

    pub async fn submit_task(&mut self, task: &mut Task, db: &DbConn) -> Result<(), EmulateError> {
        match self.get_idle_agent() {
            Some(physical_agent_id) => {
                self.set_agent_status(&physical_agent_id, AgentStatus::Running)?;
                let agent = Agent::new(
                    physical_agent_id,
                    task.get_source().to_string(),
                    Some(task.get_option_id()),
                );

                task.set_agent_id(agent.get_agent_id());

                match (
                    agent.insert_to_db(db).await,
                    task.updated_agent_id(db).await,
                ) {
                    (Ok(_), Ok(_)) => Ok(()),
                    (Err(err), _) => {
                        self.set_agent_status(&physical_agent_id, AgentStatus::Idle)?;
                        Err(EmulateError::ResourceDbError(err))
                    }
                    (_, Err(err)) => {
                        self.set_agent_status(&physical_agent_id, AgentStatus::Idle)?;
                        Err(EmulateError::ResourceDbError(err))
                    }
                }
            }
            None => Err(EmulateError::AgentNotIdle(Uuid::new_v4())),
        }
    }

    pub async fn run_task(
        &self,
        task: &Task,
        options: &qasmsim::options::Options,
    ) -> Result<String, EmulateError> {
        match qasmsim::run(&task.source, options.shots) {
            Ok(result) => Ok(qasmsim::print_result(&result, &options)),
            Err(err) => Err(EmulateError::QasmSimError(format!("{}", err))),
        }
    }
}
