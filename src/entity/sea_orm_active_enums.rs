//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.12

use sea_orm::entity::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "agent_status")]
pub enum AgentStatus {
    #[sea_orm(string_value = "failed")]
    Failed,
    #[sea_orm(string_value = "running")]
    Running,
    #[sea_orm(string_value = "succeeded")]
    Succeeded,
}
#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "format")]
pub enum Format {
    #[sea_orm(string_value = "json")]
    Json,
    #[sea_orm(string_value = "tabular")]
    Tabular,
}
#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(
    rs_type = "String",
    db_type = "Enum",
    enum_name = "physical_agent_status"
)]
pub enum PhysicalAgentStatus {
    #[sea_orm(string_value = "idle")]
    Idle,
    #[sea_orm(string_value = "running")]
    Running,
}
#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "task_status")]
pub enum TaskStatus {
    #[sea_orm(string_value = "failed")]
    Failed,
    #[sea_orm(string_value = "not_started")]
    NotStarted,
    #[sea_orm(string_value = "running")]
    Running,
    #[sea_orm(string_value = "succeeded")]
    Succeeded,
}
