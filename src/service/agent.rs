use crate::entity::*;
use sea_orm::{ActiveValue, DbConn, EntityTrait};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AgentStatus {
    Failed,
    Idle,
    Running,
    Succeeded,
}
// qthread -> resource -> agent

pub struct Agent {
    pub id: Uuid,
    pub physical_id: Option<Uuid>,
    pub source: String,
    pub result: Option<String>,
    pub status: AgentStatus,
    pub option_id: Option<Uuid>,
}

impl Default for Agent {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            physical_id: None,
            // source should be a meaningful value, if set to default, it will cause error
            source: "".to_string(),
            result: None,
            status: AgentStatus::Idle,
            option_id: None,
        }
    }
}

impl Agent {
    pub fn new(physical_id: Uuid, source: String, option_id: Option<Uuid>) -> Self {
        Self {
            id: Uuid::new_v4(),
            physical_id: Some(physical_id),
            source,
            result: None,
            status: AgentStatus::Idle,
            option_id,
        }
    }

    pub fn get_agent_id(&self) -> Uuid {
        self.id
    }

    pub fn get_physical_id(&self) -> Option<Uuid> {
        self.physical_id
    }

    pub async fn insert_to_db(&self, db: &DbConn) -> Result<(), sea_orm::prelude::DbErr> {
        agent::Entity::insert(agent::ActiveModel {
            id: ActiveValue::set(self.id),
            physical_id: ActiveValue::set(self.physical_id.unwrap()),
            source: ActiveValue::set(self.source.clone()),
            result: ActiveValue::set(self.result.clone()),
            status: ActiveValue::set(match self.status {
                AgentStatus::Failed => sea_orm_active_enums::AgentStatus::Failed,
                AgentStatus::Idle => sea_orm_active_enums::AgentStatus::Idle,
                AgentStatus::Running => sea_orm_active_enums::AgentStatus::Running,
                AgentStatus::Succeeded => sea_orm_active_enums::AgentStatus::Succeeded,
            }),
            option_id: ActiveValue::set(self.option_id.unwrap()),
        })
        .exec(db)
        .await?;
        Ok(())
    }
}
