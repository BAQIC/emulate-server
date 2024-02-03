use crate::entity::*;
use chrono::NaiveDateTime;
use sea_orm::{ActiveModelTrait, ActiveValue, DbConn, EntityTrait};
use uuid::Uuid;

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
    pub agent_id: Option<Uuid>,
    pub physical_agent_id: Option<Uuid>,
}

impl Default for Task {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            // source should be a meaningful value, if set to default, it will cause error
            source: "".to_string(),
            result: None,
            // option id should be a meaningful value, if set to default, it will cause error
            option_id: Uuid::new_v4(),
            status: TaskStatus::NotStarted,
            created_time: chrono::Utc::now().naive_utc(),
            updated_time: chrono::Utc::now().naive_utc(),
            agent_id: None,
            physical_agent_id: None,
        }
    }
}

impl Task {
    pub fn new(source: String, option_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            source,
            result: None,
            option_id,
            status: TaskStatus::NotStarted,
            created_time: chrono::Utc::now().naive_utc(),
            updated_time: chrono::Utc::now().naive_utc(),
            agent_id: None,
            physical_agent_id: None,
        }
    }

    pub fn get_id(&self) -> Uuid {
        self.id
    }

    pub fn set_agent_id(&mut self, agent_id: Uuid) {
        self.agent_id = Some(agent_id);
    }

    pub fn set_physical_id(&mut self, physical_agent_id: Uuid) {
        self.physical_agent_id = Some(physical_agent_id);
    }

    pub fn get_source(&self) -> &str {
        &self.source
    }

    pub fn get_option_id(&self) -> Uuid {
        self.option_id
    }

    pub async fn insert_to_db(&self, db: &DbConn) -> Result<(), sea_orm::prelude::DbErr> {
        task::Entity::insert(task::ActiveModel {
            id: ActiveValue::set(self.id),
            source: ActiveValue::set(self.source.clone()),
            result: ActiveValue::set(self.result.clone()),
            option_id: ActiveValue::set(self.option_id),
            status: ActiveValue::set(match self.status {
                TaskStatus::Failed => sea_orm_active_enums::TaskStatus::Failed,
                TaskStatus::NotStarted => sea_orm_active_enums::TaskStatus::NotStarted,
                TaskStatus::Running => sea_orm_active_enums::TaskStatus::Running,
                TaskStatus::Succeeded => sea_orm_active_enums::TaskStatus::Succeeded,
            }),
            created_time: ActiveValue::set(self.created_time),
            updated_time: ActiveValue::set(self.updated_time),
            agent_id: ActiveValue::set(self.agent_id),
            physical_agent_id: ActiveValue::set(self.physical_agent_id),
        })
        .exec(db)
        .await?;
        Ok(())
    }

    pub async fn updated_agent_id(&self, db: &DbConn) -> Result<(), sea_orm::prelude::DbErr> {
        let mut task: task::ActiveModel = task::Entity::find_by_id(self.id)
            .one(db)
            .await?
            .unwrap()
            .into();
        task.agent_id = ActiveValue::set(self.agent_id);
        task.updated_time = ActiveValue::set(chrono::Utc::now().naive_utc());
        task.update(db).await?;
        Ok(())
    }
}
