use super::create_task::TaskMode;
use extension::postgres::Type;
use sea_orm::{EnumIter, Iterable};
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
pub enum TaskActive {
    Table,
    Id,
    Source,
    Vars,
    Result,
    Qubits,
    Shots,
    ExecShots,
    VExecShots,
    Depth,
    Mode,
    Status,
    CreatedTime,
    UpdatedTime,
}

#[derive(DeriveIden, EnumIter)]
pub enum TaskActiveStatus {
    Table,
    Running,
    Waiting,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_type(
                Type::create()
                    .as_enum(TaskActiveStatus::Table)
                    .values(TaskActiveStatus::iter().skip(1))
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(TaskActive::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TaskActive::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(TaskActive::Source).string().not_null())
                    .col(ColumnDef::new(TaskActive::Vars).string().null())
                    .col(ColumnDef::new(TaskActive::Result).string().null())
                    .col(ColumnDef::new(TaskActive::Qubits).unsigned().not_null())
                    .col(ColumnDef::new(TaskActive::Shots).unsigned().not_null())
                    .col(ColumnDef::new(TaskActive::ExecShots).unsigned().not_null())
                    .col(ColumnDef::new(TaskActive::VExecShots).unsigned().not_null())
                    .col(ColumnDef::new(TaskActive::Depth).unsigned().not_null())
                    .col(
                        ColumnDef::new(TaskActive::Status)
                            .enumeration(TaskActiveStatus::Table, TaskActiveStatus::iter().skip(1))
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TaskActive::Mode)
                            .enumeration(TaskMode::Table, TaskMode::iter().skip(1))
                            .null(),
                    )
                    .col(
                        ColumnDef::new(TaskActive::CreatedTime)
                            .timestamp()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TaskActive::UpdatedTime)
                            .timestamp()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(TaskActive::Table)
                    .if_exists()
                    .to_owned(),
            )
            .await
    }
}
