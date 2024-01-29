use super::content;
use crate::entity::*;
use sea_orm::{ActiveValue, DbConn, EntityTrait};

impl content::Resource {}

impl content::Qthread {
    pub async fn add_resources(
        &mut self,
        resources: &Vec<content::Resource>,
        db: &DbConn,
    ) -> Result<(), sea_orm::prelude::DbErr> {
        resource::Entity::insert_many(resources.iter().map(|r| {
            self.resources.push(content::QthreadResource {
                resource_id: r.id,
                quota: r.maximum_agents_num as u32,
                current_quota: r.current_agents_num as u32,
            });

            resource::ActiveModel {
                id: ActiveValue::set(r.id),
                status: ActiveValue::set(match r.status {
                    content::ResourceStatus::FullyUsed => {
                        sea_orm_active_enums::ResourceStatus::FullyUsed
                    }
                    content::ResourceStatus::PartiallyUsed => {
                        sea_orm_active_enums::ResourceStatus::PartiallyUsed
                    }
                    content::ResourceStatus::Paused => sea_orm_active_enums::ResourceStatus::Paused,
                }),
                maximum_agents_num: ActiveValue::set(r.maximum_agents_num),
                current_agents_num: ActiveValue::set(r.current_agents_num),
                agent_ids: ActiveValue::set(r.agent_ids.clone()),
                current_agent_ids: ActiveValue::set(r.current_agent_ids.clone()),
            }
        }))
        .exec(db)
        .await?;
        Ok(())
    }

    pub async fn add_resource(
        &mut self,
        resource: &content::Resource,
        db: &DbConn,
    ) -> Result<(), sea_orm::prelude::DbErr> {
        self.resources.push(content::QthreadResource {
            resource_id: resource.id,
            quota: resource.maximum_agents_num as u32,
            current_quota: resource.current_agents_num as u32,
        });

        resource::Entity::insert(resource::ActiveModel {
            id: ActiveValue::set(resource.id),
            status: ActiveValue::set(match resource.status {
                content::ResourceStatus::FullyUsed => {
                    sea_orm_active_enums::ResourceStatus::FullyUsed
                }
                content::ResourceStatus::PartiallyUsed => {
                    sea_orm_active_enums::ResourceStatus::PartiallyUsed
                }
                content::ResourceStatus::Paused => sea_orm_active_enums::ResourceStatus::Paused,
            }),
            maximum_agents_num: ActiveValue::set(resource.maximum_agents_num),
            current_agents_num: ActiveValue::set(resource.current_agents_num),
            agent_ids: ActiveValue::set(resource.agent_ids.clone()),
            current_agent_ids: ActiveValue::set(resource.current_agent_ids.clone()),
        })
        .exec(db)
        .await?;
        Ok(())
    }

    pub async fn add_task(
        &mut self,
        task: &content::Task,
        db: &DbConn,
    ) -> Result<(), sea_orm::prelude::DbErr> {
        task::Entity::insert(task::ActiveModel {
            id: ActiveValue::set(task.id),
            source: ActiveValue::set(task.source.clone()),
            result: ActiveValue::set(task.result.clone()),
            option_id: ActiveValue::set(task.option_id),
            status: ActiveValue::set(match task.status {
                content::TaskStatus::Failed => sea_orm_active_enums::TaskStatus::Failed,
                content::TaskStatus::Running => sea_orm_active_enums::TaskStatus::Running,
                content::TaskStatus::Succeeded => sea_orm_active_enums::TaskStatus::Succeeded,
                content::TaskStatus::NotStarted => sea_orm_active_enums::TaskStatus::NotStarted,
            }),
            created_time: ActiveValue::set(task.created_time),
            updated_time: ActiveValue::set(task.updated_time),
            resource_id: ActiveValue::set(task.resource_id),
            agent_id: ActiveValue::set(task.agent_id),
        })
        .exec(db)
        .await?;

        Ok(())
    }
}
