use crate::entity::*;
use sea_orm::{ActiveModelTrait, ColumnTrait, DbConn, DeleteResult, EntityTrait, QueryFilter, Set};

pub struct Agent;

impl Agent {
    pub async fn add_agent(
        db: &DbConn,
        data: agent::Model,
    ) -> Result<agent::Model, sea_orm::prelude::DbErr> {
        agent::ActiveModel {
            id: Set(data.id.to_owned()),
            physical_id: Set(data.physical_id.to_owned()),
            source: Set(data.source.to_owned()),
            result: Set(data.result.to_owned()),
            status: Set(data.status.to_owned()),
            option_id: Set(data.option_id.to_owned()),
        }
        .insert(db)
        .await
    }

    pub async fn updated_agent_status_result(
        db: &DbConn,
        agent_id: uuid::Uuid,
        status: sea_orm_active_enums::AgentStatus,
        result: Option<String>,
    ) -> Result<agent::Model, sea_orm::prelude::DbErr> {
        let mut agent: agent::ActiveModel = agent::Entity::find_by_id(agent_id)
            .one(db)
            .await?
            .unwrap()
            .into();

        agent.status = Set(status);
        agent.result = Set(result);
        agent.update(db).await
    }
}

pub struct PhysicalAgent;

impl PhysicalAgent {
    pub async fn add_physical_agent(
        db: &DbConn,
        data: physical_agent::Model,
    ) -> Result<physical_agent::Model, sea_orm::prelude::DbErr> {
        physical_agent::ActiveModel {
            id: Set(data.id.to_owned()),
            physical_agent_status: Set(data.physical_agent_status.to_owned()),
        }
        .insert(db)
        .await
    }

    pub async fn get_idle_physical_agent(
        db: &DbConn,
    ) -> Result<Option<physical_agent::Model>, sea_orm::prelude::DbErr> {
        match physical_agent::Entity::find()
            .filter(
                physical_agent::Column::PhysicalAgentStatus
                    .eq(sea_orm_active_enums::PhysicalAgentStatus::Idle),
            )
            .one(db)
            .await
        {
            Ok(Some(agent)) => {
                let mut agent_activate_model: physical_agent::ActiveModel = agent.clone().into();
                agent_activate_model.physical_agent_status =
                    Set(sea_orm_active_enums::PhysicalAgentStatus::Running);
                agent_activate_model.update(db).await?;
                Ok(Some(agent))
            }
            Ok(None) => Ok(None),
            Err(err) => Err(err),
        }
    }

    pub async fn update_physical_agent_status(
        db: &DbConn,
        agent_id: uuid::Uuid,
        status: sea_orm_active_enums::PhysicalAgentStatus,
    ) -> Result<physical_agent::Model, sea_orm::prelude::DbErr> {
        let mut agent: physical_agent::ActiveModel = physical_agent::Entity::find_by_id(agent_id)
            .one(db)
            .await?
            .unwrap()
            .into();

        agent.physical_agent_status = Set(status);
        agent.update(db).await
    }

    pub async fn remove_physical_agent(
        db: &DbConn,
        agent_id: uuid::Uuid,
    ) -> Result<DeleteResult, sea_orm::prelude::DbErr> {
        physical_agent::Entity::delete_by_id(agent_id)
            .exec(db)
            .await
    }
}
