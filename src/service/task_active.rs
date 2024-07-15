use crate::entity::*;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DbConn, EntityTrait, QueryFilter, QueryOrder,
};

pub struct TaskActive;

impl TaskActive {
    pub async fn add_task(
        db: &DbConn,
        data: task_active::Model,
    ) -> Result<task_active::Model, sea_orm::prelude::DbErr> {
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

    pub async fn get_asc_tasks(
        db: &DbConn,
    ) -> Result<Vec<task_active::Model>, sea_orm::prelude::DbErr> {
        task_active::Entity::find()
            .filter(task_active::Column::Status.eq(sea_orm_active_enums::TaskActiveStatus::Waiting))
            .order_by_asc(task_active::Column::VExecShots)
            .all(db)
            .await
    }

    pub async fn get_task(
        db: &DbConn,
        task_id: uuid::Uuid,
    ) -> Result<Option<task_active::Model>, sea_orm::prelude::DbErr> {
        task_active::Entity::find_by_id(task_id).one(db).await
    }

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
