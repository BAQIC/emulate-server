use crate::entity::*;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DbConn, DeleteResult, EntityTrait, QueryFilter,
    QueryOrder, Set,
};

pub struct PhysicalAgent;

impl PhysicalAgent {
    pub async fn add_physical_agent(
        db: &DbConn,
        data: physical_agent::Model,
    ) -> Result<physical_agent::Model, sea_orm::prelude::DbErr> {
        physical_agent::ActiveModel {
            id: Set(data.id.to_owned()),
            status: Set(data.status.to_owned()),
            ip: Set(data.ip.to_owned()),
            port: Set(data.port.to_owned()),
            qubit_count: Set(data.qubit_count.to_owned()),
            qubit_idle: Set(data.qubit_idle.to_owned()),
            circuit_depth: Set(data.circuit_depth.to_owned()),
        }
        .insert(db)
        .await
    }

    pub async fn get_most_available_physical_agent(
        db: &DbConn,
        task_qubits: u32,
        task_depth: u32,
    ) -> Result<Option<physical_agent::Model>, sea_orm::prelude::DbErr> {
        match physical_agent::Entity::find()
            .filter(
                Condition::all()
                    .add(
                        physical_agent::Column::Status
                            .eq(sea_orm_active_enums::PhysicalAgentStatus::Running),
                    )
                    .add(physical_agent::Column::QubitIdle.gte(task_qubits as i32))
                    .add(physical_agent::Column::CircuitDepth.gte(task_depth as i32)),
            )
            .order_by_desc(physical_agent::Column::QubitIdle)
            .one(db)
            .await
        {
            Ok(Some(agent)) => Ok(Some(agent)),
            Ok(None) => Ok(None),
            Err(err) => Err(err),
        }
    }

    pub async fn get_least_available_physical_agent(
        db: &DbConn,
        task_qubits: u32,
        task_depth: u32,
    ) -> Result<Option<physical_agent::Model>, sea_orm::prelude::DbErr> {
        match physical_agent::Entity::find()
            .filter(
                Condition::all()
                    .add(
                        physical_agent::Column::Status
                            .eq(sea_orm_active_enums::PhysicalAgentStatus::Running),
                    )
                    .add(physical_agent::Column::QubitIdle.gte(task_qubits as i32))
                    .add(physical_agent::Column::CircuitDepth.gte(task_depth as i32)),
            )
            .order_by_asc(physical_agent::Column::QubitIdle)
            .one(db)
            .await
        {
            Ok(Some(agent)) => Ok(Some(agent)),
            Ok(None) => Ok(None),
            Err(err) => Err(err),
        }
    }

    pub async fn get_physical_agent(
        db: &DbConn,
        agent_id: uuid::Uuid,
    ) -> Result<Option<physical_agent::Model>, sea_orm::prelude::DbErr> {
        physical_agent::Entity::find_by_id(agent_id).one(db).await
    }

    pub async fn update_physical_agent_qubits_idle(
        db: &DbConn,
        agent_id: uuid::Uuid,
        qubits_idle: i32,
    ) -> Result<physical_agent::Model, sea_orm::prelude::DbErr> {
        let mut agent: physical_agent::ActiveModel = physical_agent::Entity::find_by_id(agent_id)
            .one(db)
            .await?
            .unwrap()
            .into();

        agent.qubit_idle = Set(qubits_idle);
        agent.update(db).await
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

        agent.status = Set(status);
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
