use super::resource::Resource;
use super::task::Task;
use crate::entity::*;
use sea_orm::DbConn;

pub struct Qthread;

impl Qthread {
    pub async fn submit_task(
        db: &DbConn,
        data: task::Model,
    ) -> Result<task::Model, sea_orm::prelude::DbErr> {
        match Task::add_task(db, data).await {
            Ok(task) => match Resource::submit_task(db, task).await {
                Ok(task) => Ok(task),
                Err(err) => Err(err),
            },
            Err(err) => Err(err),
        }
    }

    pub async fn finish_task(
        db: &DbConn,
        task_id: uuid::Uuid,
        result: Option<String>,
        task_status: sea_orm_active_enums::TaskStatus,
        agent_status: sea_orm_active_enums::AgentStatus,
    ) -> Result<task::Model, sea_orm::prelude::DbErr> {
        match Task::update_task_status_result(db, task_id, task_status, result.clone()).await {
            Ok(task) => match Resource::finish_task(db, task.id, result, agent_status).await {
                Ok(_) => Ok(task),
                Err(err) => Err(err),
            },
            Err(err) => Err(err),
        }
    }

    pub async fn submit_waiting_task(
        db: &DbConn,
    ) -> Result<Option<task::Model>, sea_orm::prelude::DbErr> {
        match Task::get_first_waiting_task(db).await {
            Ok(Some(task)) => match Resource::submit_task(db, task).await {
                Ok(task) => Ok(Some(task)),
                Err(err) => Err(err),
            },
            Ok(None) => Ok(None),
            Err(err) => Err(err),
        }
    }
}
