use crate::entity::*;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DbConn, EntityTrait, QueryFilter, QueryOrder,
};

pub struct TaskActive;

impl TaskActive {
    /// Add a new task to the database. If there is no available physical agent,
    /// it will return an error.
    pub async fn add_task(
        db: &DbConn,
        data: task_active::Model,
    ) -> Result<task_active::Model, sea_orm::prelude::DbErr> {
        match super::physical_agent::PhysicalAgent::get_physical_agent_available(
            db,
            data.qubits,
            data.depth,
        )
        .await
        {
            Ok(agents) => {
                if agents.is_empty() {
                    Err(sea_orm::prelude::DbErr::Custom(
                        "No available physical agent".to_owned(),
                    ))
                } else {
                    task_active::ActiveModel {
                        id: ActiveValue::set(data.id.to_owned()),
                        source: ActiveValue::set(data.source.to_owned()),
                        result: ActiveValue::set(data.result.to_owned()),
                        qubits: ActiveValue::set(data.qubits.to_owned()),
                        depth: ActiveValue::set(data.depth.to_owned()),
                        shots: ActiveValue::set(data.shots.to_owned()),
                        exec_shots: ActiveValue::set(data.exec_shots.to_owned()),
                        v_exec_shots: ActiveValue::set(data.v_exec_shots.to_owned()),
                        status: ActiveValue::set(data.status.to_owned()),
                        created_time: ActiveValue::set(data.created_time.to_owned()),
                        updated_time: ActiveValue::set(data.updated_time.to_owned()),
                    }
                    .insert(db)
                    .await
                }
            }
            Err(err) => Err(err),
        }
    }

    /// Get all the tasks that are waiting to be executed. The tasks are ordered
    /// by the number of virtual executed shots in ascending order.
    pub async fn get_asc_tasks(
        db: &DbConn,
    ) -> Result<Vec<task_active::Model>, sea_orm::prelude::DbErr> {
        task_active::Entity::find()
            .filter(task_active::Column::Status.eq(sea_orm_active_enums::TaskActiveStatus::Waiting))
            .order_by_asc(task_active::Column::VExecShots)
            .all(db)
            .await
    }

    /// Get the task with the given ID.
    pub async fn get_task(
        db: &DbConn,
        task_id: uuid::Uuid,
    ) -> Result<Option<task_active::Model>, sea_orm::prelude::DbErr> {
        task_active::Entity::find_by_id(task_id).one(db).await
    }

    /// Get the minimum number of virtual executed shots of the tasks that are
    /// waiting to be executed. This function is used to update the vexec_shots
    /// for the new task.
    pub async fn get_min_vexec_shots(db: &DbConn) -> Result<i32, sea_orm::prelude::DbErr> {
        match task_active::Entity::find()
            .filter(task_active::Column::Status.eq(sea_orm_active_enums::TaskActiveStatus::Waiting))
            .order_by_asc(task_active::Column::VExecShots)
            .one(db)
            .await
        {
            Ok(Some(task)) => Ok(task.v_exec_shots),
            Ok(None) => Ok(0),
            Err(err) => Err(err),
        }
    }

    /// Update the task result with the given information.
    pub async fn update_task_result(
        db: &DbConn,
        task_id: uuid::Uuid,
        exec_shots: i32,
        vexec_shots: i32,
        result: Option<String>,
        status: sea_orm_active_enums::TaskActiveStatus,
    ) -> Result<task_active::Model, sea_orm::prelude::DbErr> {
        let mut task: task_active::ActiveModel = task_active::Entity::find_by_id(task_id)
            .one(db)
            .await?
            .unwrap()
            .into();
        task.result = ActiveValue::set(result);
        task.exec_shots = ActiveValue::set(exec_shots);
        task.v_exec_shots = ActiveValue::set(vexec_shots);
        task.status = ActiveValue::set(status);
        task.updated_time = ActiveValue::set(chrono::Utc::now().naive_utc());
        task.update(db).await
    }

    /// Remove the task with the given ID. After removing it, the task will be
    /// added to the history task table.
    pub async fn remove_active_task(
        db: &DbConn,
        task_id: uuid::Uuid,
    ) -> Result<task_active::Model, sea_orm::prelude::DbErr> {
        let task = task_active::Entity::find_by_id(task_id)
            .one(db)
            .await?
            .unwrap();
        task_active::Entity::delete_by_id(task_id).exec(db).await?;
        Ok(task)
    }
}
