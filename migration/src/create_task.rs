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
    Shots,
    ExecShots,
    VExecShots,
    Depth,
    Status,
    CreatedTime,
    UpdatedTime,
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
                    .col(ColumnDef::new(Task::Shots).unsigned().null())
                    .col(ColumnDef::new(Task::ExecShots).unsigned().null())
                    .col(ColumnDef::new(Task::VExecShots).unsigned().null())
                    .col(ColumnDef::new(Task::Depth).unsigned().null())
                    .col(
                        ColumnDef::new(Task::Status)
                            .enumeration(TaskStatus::Table, TaskStatus::iter().skip(1))
                            .not_null(),
                    )
                    .col(ColumnDef::new(Task::CreatedTime).timestamp().not_null())
                    .col(ColumnDef::new(Task::UpdatedTime).timestamp().not_null())
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
