use super::{
    m20220101_000001_create_options::Options,
    m20220101_000002_create_agent::Agent,
    // m20220101_000003_create_resource::Resource,
};
use sea_orm_migration::{
    prelude::*,
    sea_orm::{EnumIter, Iterable},
    sea_query::extension::postgres::Type,
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
pub enum Task {
    Table,
    Id,
    Source,
    Result,
    OptionId,
    Status,
    CreatedTime,
    UpdatedTime,
    AgentId,
    PhysicalAgentId,
}

#[derive(DeriveIden, EnumIter)]
pub enum TaskStatus {
    Table,
    Running,
    Succeeded,
    Failed,
    NotStarted,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_type(
                Type::create()
                    .as_enum(TaskStatus::Table)
                    .values(TaskStatus::iter().skip(1))
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(Task::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Task::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Task::Source).string().not_null())
                    .col(ColumnDef::new(Task::Result).string().null())
                    .col(ColumnDef::new(Task::OptionId).uuid().not_null())
                    .col(
                        ColumnDef::new(Task::Status)
                            .enumeration(TaskStatus::Table, TaskStatus::iter().skip(1))
                            .not_null(),
                    )
                    .col(ColumnDef::new(Task::CreatedTime).timestamp().not_null())
                    .col(ColumnDef::new(Task::UpdatedTime).timestamp().not_null())
                    .col(ColumnDef::new(Task::AgentId).uuid().null())
                    .col(ColumnDef::new(Task::PhysicalAgentId).uuid().null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_task_option_id")
                            .from_col(Task::OptionId)
                            .to(Options::Table, Options::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_task_agent_id")
                            .from_col(Task::AgentId)
                            .to(Agent::Table, Agent::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Task::Table).if_exists().to_owned())
            .await
    }
}
