use crate::entity::*;
use sea_orm::{ActiveModelTrait, ActiveValue, DbConn, EntityTrait};

pub struct TaskAssignment;

impl TaskAssignment {
    pub async fn add_assignment(
        db: &DbConn,
        data: task_assignment::Model,
    ) -> Result<task_assignment::Model, sea_orm::prelude::DbErr> {
        task_assignment::ActiveModel {
            id: ActiveValue::set(data.id.to_owned()),
            task_id: ActiveValue::set(data.task_id.to_owned()),
            agent_id: ActiveValue::set(data.agent_id.to_owned()),
            shots: ActiveValue::set(data.shots.to_owned()),
            status: ActiveValue::set(data.status.to_owned()),
        }
        .insert(db)
        .await
    }

    pub async fn get_assignment_by_task(
        db: &DbConn,
        task_id: uuid::Uuid,
    ) -> Result<Vec<task_assignment::Model>, sea_orm::prelude::DbErr> {
        task_assignment::Entity::find_by_id(task_id).all(db).await
    }

    pub async fn get_assignment_by_agent(
        db: &DbConn,
        agent_id: uuid::Uuid,
    ) -> Result<Vec<task_assignment::Model>, sea_orm::prelude::DbErr> {
        task_assignment::Entity::find_by_id(agent_id).all(db).await
    }
}
