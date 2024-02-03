use super::m20220101_000001_create_options::Options;
use sea_orm_migration::{
    prelude::*,
    sea_orm::{EnumIter, Iterable},
    sea_query::extension::postgres::Type,
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
pub enum Agent {
    Table,
    Id,
    PhysicalId,
    Source,
    Result,
    Status,
    OptionId,
}

#[derive(DeriveIden, EnumIter)]
enum AgentStatus {
    Table,
    Idle,
    Running,
    Succeeded,
    Failed,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_type(
                Type::create()
                    .as_enum(AgentStatus::Table)
                    .values(AgentStatus::iter().skip(1))
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Agent::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Agent::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Agent::PhysicalId).uuid().not_null())
                    .col(ColumnDef::new(Agent::Source).string().not_null())
                    .col(ColumnDef::new(Agent::Result).string().null())
                    .col(
                        ColumnDef::new(Agent::Status)
                            .enumeration(AgentStatus::Table, AgentStatus::iter().skip(1))
                            .not_null(),
                    )
                    .col(ColumnDef::new(Agent::OptionId).uuid().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_agent_options_id")
                            .from(Agent::Table, Agent::OptionId)
                            .to(Options::Table, Options::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Agent::Table).if_exists().to_owned())
            .await
    }
}
