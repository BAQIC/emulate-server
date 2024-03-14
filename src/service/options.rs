use crate::entity::*;
use sea_orm::{ActiveModelTrait, ActiveValue, DbConn, EntityTrait};

pub struct Options;

impl Options {
    pub async fn add_options(
        db: &DbConn,
        options: options::Model,
    ) -> Result<options::Model, sea_orm::prelude::DbErr> {
        options::ActiveModel {
            id: ActiveValue::set(options.id.to_owned()),
            agent_type: ActiveValue::set(options.agent_type.to_owned()),
            shots: ActiveValue::set(options.shots.to_owned()),
        }
        .insert(db)
        .await
    }

    pub async fn add_qasm_options(
        db: &DbConn,
        options: &crate::router::Options,
    ) -> Result<options::Model, sea_orm::prelude::DbErr> {
        let options = options::Model {
            id: uuid::Uuid::new_v4(),
            agent_type: match options.agent_type {
                crate::router::AgentType::QppSV => sea_orm_active_enums::AgentType::QppSv,
                crate::router::AgentType::QppDM => sea_orm_active_enums::AgentType::QppDm,
                crate::router::AgentType::QASMSim => sea_orm_active_enums::AgentType::QasmSim,
                crate::router::AgentType::CUDAQ => sea_orm_active_enums::AgentType::Cudaq,
            },
            shots: options.shots.map(|x| x as i32),
        };
        Options::add_options(db, options).await
    }

    pub async fn get_option(
        db: &DbConn,
        id: uuid::Uuid,
    ) -> Result<Option<options::Model>, sea_orm::prelude::DbErr> {
        options::Entity::find_by_id(id).one(db).await
    }
}
