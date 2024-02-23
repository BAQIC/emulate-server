use super::{
    agent::{Agent, PhysicalAgent},
    task::Task,
};
use crate::entity::*;
use sea_orm::{DbConn, DeleteResult};
use uuid::Uuid;

pub struct Resource;

impl Resource {
    pub async fn random_init_physical_agents(
        db: &DbConn,
        physical_agents_num: u32,
    ) -> Result<Vec<physical_agent::Model>, sea_orm::prelude::DbErr> {
        let mut agents = vec![];
        for _ in 0..physical_agents_num {
            agents.push(Resource::add_physical_agent(db, Uuid::new_v4()).await?);
        }
        Ok(agents)
    }

    pub async fn add_physical_agent(
        db: &DbConn,
        agent_id: Uuid,
    ) -> Result<physical_agent::Model, sea_orm::prelude::DbErr> {
        PhysicalAgent::add_physical_agent(
            db,
            physical_agent::Model {
                id: agent_id,
                physical_agent_status: sea_orm_active_enums::PhysicalAgentStatus::Idle,
            },
        )
        .await
    }

    pub async fn remove_physical_agent(
        db: &DbConn,
        agent_id: Uuid,
    ) -> Result<DeleteResult, sea_orm::prelude::DbErr> {
        PhysicalAgent::remove_physical_agent(db, agent_id).await
    }

    pub async fn get_idle_agent(
        db: &DbConn,
    ) -> Result<Option<physical_agent::Model>, sea_orm::prelude::DbErr> {
        PhysicalAgent::get_idle_physical_agent(db).await
    }

    pub async fn get_idle_agent_num(db: &DbConn) -> Result<u64, sea_orm::prelude::DbErr> {
        PhysicalAgent::get_idle_physical_agent_num(db).await
    }

    pub async fn update_physical_agent_status(
        db: &DbConn,
        agent_id: Uuid,
        status: sea_orm_active_enums::PhysicalAgentStatus,
    ) -> Result<physical_agent::Model, sea_orm::prelude::DbErr> {
        PhysicalAgent::update_physical_agent_status(db, agent_id, status).await
    }

    pub async fn submit_task(
        db: &DbConn,
        task: task::Model,
    ) -> Result<task::Model, sea_orm::prelude::DbErr> {
        let result = Resource::get_idle_agent(db).await;
        match result {
            Ok(Some(physical_agent)) => {
                match Agent::add_agent(
                    db,
                    agent::Model {
                        id: uuid::Uuid::new_v4(),
                        physical_id: physical_agent.id,
                        source: task.source.clone(),
                        result: None,
                        status: sea_orm_active_enums::AgentStatus::Running,
                        option_id: task.option_id,
                    },
                )
                .await
                {
                    Ok(agent) => {
                        match Task::update_task_agent_id_status(
                            db,
                            task.id,
                            agent.id,
                            sea_orm_active_enums::TaskStatus::Running,
                        )
                        .await
                        {
                            Ok(task) => Ok(task),
                            Err(e) => Err(e),
                        }
                    }
                    Err(e) => Err(e),
                }
            }
            Ok(None) => Ok(task),
            Err(e) => Err(e),
        }
    }

    pub async fn finish_task(
        db: &DbConn,
        agent_id: Uuid,
        result: Option<String>,
        agent_status: sea_orm_active_enums::AgentStatus,
    ) -> Result<agent::Model, sea_orm::prelude::DbErr> {
        match Agent::updated_agent_status_result(db, agent_id, agent_status, result).await {
            Ok(agent) => {
                match PhysicalAgent::update_physical_agent_status(
                    db,
                    agent.physical_id,
                    sea_orm_active_enums::PhysicalAgentStatus::Idle,
                )
                .await
                {
                    Ok(_) => Ok(agent),
                    Err(e) => Err(e),
                }
            }
            Err(e) => Err(e),
        }
    }
}
