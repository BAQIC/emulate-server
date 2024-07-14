use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
pub enum TaskActive {
    Table,
    Id,
    Source,
    Result,
    Qubits,
    Shots,
    ExecShots,
    VExecShots,
    Depth,
    CreatedTime,
    UpdatedTime,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
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
                    .col(ColumnDef::new(TaskActive::Result).string().null())
                    .col(ColumnDef::new(TaskActive::Qubits).unsigned().not_null())
                    .col(ColumnDef::new(TaskActive::Shots).unsigned().not_null())
                    .col(ColumnDef::new(TaskActive::ExecShots).unsigned().not_null())
                    .col(ColumnDef::new(TaskActive::VExecShots).unsigned().not_null())
                    .col(ColumnDef::new(TaskActive::Depth).unsigned().not_null())
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
