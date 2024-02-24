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
            option_id: ActiveValue::set(data.option_id.to_owned()),
            status: ActiveValue::set(data.status.to_owned()),
            created_time: ActiveValue::set(data.created_time.to_owned()),
            updated_time: ActiveValue::set(data.updated_time.to_owned()),
            agent_id: ActiveValue::set(data.agent_id.to_owned()),
        }
        .insert(db)
        .await
    }

    pub async fn update_task_agent_id_status(
        db: &DbConn,
        task_id: uuid::Uuid,
        agent_id: uuid::Uuid,
        status: sea_orm_active_enums::TaskStatus,
    ) -> Result<task::Model, sea_orm::prelude::DbErr> {
        let mut task: task::ActiveModel = task::Entity::find_by_id(task_id)
            .one(db)
            .await?
            .unwrap()
            .into();
        task.agent_id = ActiveValue::set(Some(agent_id));
        task.updated_time = ActiveValue::set(chrono::Utc::now().naive_utc());
        task.status = ActiveValue::set(status);
        task.update(db).await
    }

    pub async fn get_first_waiting_task(
        db: &DbConn,
    ) -> Result<Option<task::Model>, sea_orm::prelude::DbErr> {
        task::Entity::find()
            .filter(task::Column::Status.eq(sea_orm_active_enums::TaskStatus::NotStarted))
            .order_by_asc(task::Column::CreatedTime)
            .one(db)
            .await
    }

    // get all waiting tasks if num is None or num is larger than the number of
    // waiting tasks
    pub async fn get_waiting_task(
        db: &DbConn,
        num: Option<u64>,
    ) -> Result<Vec<task::Model>, sea_orm::prelude::DbErr> {
        match num {
            Some(num) => {
                match task::Entity::find()
                    .filter(task::Column::Status.eq(sea_orm_active_enums::TaskStatus::NotStarted))
                    .order_by_asc(task::Column::CreatedTime)
                    .all(db)
                    .await
                {
                    Ok(tasks) => {
                        let num = num as usize;
                        if tasks.len() > num {
                            Ok(tasks[..num].to_vec())
                        } else {
                            Ok(tasks)
                        }
                    }
                    Err(e) => Err(e),
                }
            }
            None => {
                task::Entity::find()
                    .filter(task::Column::Status.eq(sea_orm_active_enums::TaskStatus::NotStarted))
                    .order_by_asc(task::Column::CreatedTime)
                    .all(db)
                    .await
            }
        }
    }

    pub async fn get_task(
        db: &DbConn,
        task_id: uuid::Uuid,
    ) -> Result<Option<task::Model>, sea_orm::prelude::DbErr> {
        task::Entity::find_by_id(task_id).one(db).await
    }

    pub async fn update_task_status_result(
        db: &DbConn,
        task_id: uuid::Uuid,
        status: sea_orm_active_enums::TaskStatus,
        result: Option<String>,
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
