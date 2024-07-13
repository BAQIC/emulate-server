use super::resource::Resource;
use super::task::Task;
use crate::entity::*;
use sea_orm::DbConn;

pub struct TaskAssignment;

impl TaskAssignment {
    pub async fn add_assignment(
        db: &DbConn,
        task_id: uuid::Uuid,
        agent_id: uuid::Uuid,
        shots: i32,
    ) -> Result<task_assignment::Model, sea_orm::prelude::DbErr> {
        TaskAssignment::add_assignment_with_status(
            db,
            task_id,
            agent_id,
            shots,
            sea_orm_active_enums::AssignmentStatus::Running,
        )
        .await
    }

    pub async fn get_assignment_by_task(
        db: &DbConn,
        task_id: uuid::Uuid,
    ) -> Result<Vec<task_assignment::Model>, sea_orm::prelude::DbErr> {
        TaskAssignment::Entity::find_by_id(task_id).all(db).await
    }

    pub async fn get_assignment_by_agent(
        db: &DbConn,
        agent_id: uuid::Uuid,
    ) -> Result<Vec<task_assignment::Model>, sea_orm::prelude::DbErr> {
        TaskAssignment::Entity::find_by_id(agent_id).all(db).await
    }
}
