use crate::entity::*;
use sea_orm::{ActiveModelTrait, ActiveValue, DbConn, EntityTrait};

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
            created_time: ActiveValue::set(data.created_time.to_owned()),
            updated_time: ActiveValue::set(data.updated_time.to_owned()),
        }
        .insert(db)
        .await
    }

    pub async fn get_task(
        db: &DbConn,
        task_id: uuid::Uuid,
    ) -> Result<Option<task_active::Model>, sea_orm::prelude::DbErr> {
        task_active::Entity::find_by_id(task_id).one(db).await
    }

    pub async fn update_task_result(
        db: &DbConn,
        task_id: uuid::Uuid,
        result: Option<String>,
    ) -> Result<task_active::Model, sea_orm::prelude::DbErr> {
        let mut task: task_active::ActiveModel = task_active::Entity::find_by_id(task_id)
            .one(db)
            .await?
            .unwrap()
            .into();
        task.result = ActiveValue::set(result);
        task.updated_time = ActiveValue::set(chrono::Utc::now().naive_utc());
        task.update(db).await
    }
}
