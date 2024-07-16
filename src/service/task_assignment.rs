use crate::entity::*;
use sea_orm::{ActiveModelTrait, ActiveValue, ColumnTrait, DbConn, EntityTrait, QueryFilter};

pub struct TaskAssignment;

impl TaskAssignment {
    /// Add a new task assignment to the database. This table is used to record
    /// the assignment of tasks to agents. The task assignment includes the task
    /// id, the agent id, the number of shots, and the status of the assignment.
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

    /// Update the status of the task assignment. The status can be `Runnig`,
    /// `Succeeded` or `Failed`.
    pub async fn update_assignment_status(
        db: &DbConn,
        assign_id: uuid::Uuid,
        status: sea_orm_active_enums::AssignmentStatus,
    ) -> Result<task_assignment::Model, sea_orm::prelude::DbErr> {
        let mut assignment: task_assignment::ActiveModel =
            task_assignment::Entity::find_by_id(assign_id)
                .one(db)
                .await?
                .unwrap()
                .into();
        assignment.status = ActiveValue::set(status);
        assignment.update(db).await
    }

    /// Get the task assignment with the given task id.
    pub async fn get_assignment_by_task(
        db: &DbConn,
        task_id: uuid::Uuid,
    ) -> Result<Vec<task_assignment::Model>, sea_orm::prelude::DbErr> {
        task_assignment::Entity::find()
            .filter(task_assignment::Column::TaskId.eq(task_id))
            .all(db)
            .await
    }

    /// Get the task assignment with the given agent id.
    pub async fn get_assignment_by_agent(
        db: &DbConn,
        agent_id: uuid::Uuid,
    ) -> Result<Vec<task_assignment::Model>, sea_orm::prelude::DbErr> {
        task_assignment::Entity::find()
            .filter(task_assignment::Column::AgentId.eq(agent_id))
            .all(db)
            .await
    }
}
