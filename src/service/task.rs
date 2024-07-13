use crate::entity::*;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DbConn, EntityTrait, QueryFilter, QueryOrder,
};

pub struct Task;

impl Task {
    pub async fn add_task(
        db: &DbConn,
        data: task::Model,
    ) -> Result<task::Model, sea_orm::prelude::DbErr> {
        task::ActiveModel {
            id: ActiveValue::set(data.id.to_owned()),
            source: ActiveValue::set(data.source.to_owned()),
            result: ActiveValue::set(data.result.to_owned()),
            qubits: ActiveValue::set(data.qubits.to_owned()),
            depth: ActiveValue::set(data.depth.to_owned()),
            shots: ActiveValue::set(data.shots.to_owned()),
            status: ActiveValue::set(data.status.to_owned()),
            created_time: ActiveValue::set(data.created_time.to_owned()),
            updated_time: ActiveValue::set(data.updated_time.to_owned()),
        }
        .insert(db)
        .await
    }

    pub async fn get_task(
        db: &DbConn,
        task_id: uuid::Uuid,
    ) -> Result<Option<task::Model>, sea_orm::prelude::DbErr> {
        task::Entity::find_by_id(task_id).one(db).await
    }

    pub async fn update_task_status(
        db: &DbConn,
        task_id: uuid::Uuid,
        status: sea_orm_active_enums::TaskStatus,
    ) -> Result<task::Model, sea_orm::prelude::DbErr> {
        let mut task: task::ActiveModel = task::Entity::find_by_id(task_id)
            .one(db)
            .await?
            .unwrap()
            .into();
        task.status = ActiveValue::set(status);
        task.updated_time = ActiveValue::set(chrono::Utc::now().naive_utc());
        task.update(db).await
    }

    pub async fn update_task_result(
        db: &DbConn,
        task_id: uuid::Uuid,
        status: sea_orm_active_enums::TaskStatus,
    ) -> Result<task::Model, sea_orm::prelude::DbErr> {
        let mut task: task::ActiveModel = task::Entity::find_by_id(task_id)
            .one(db)
            .await?
            .unwrap()
            .into();
        task.status = ActiveValue::set(status);
        task.result = ActiveValue::set(result);
        task.updated_time = ActiveValue::set(chrono::Utc::now().naive_utc());
        task.update(db).await
    }
}
