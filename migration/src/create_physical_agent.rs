use sea_orm_migration::{
    prelude::*,
    sea_orm::{EnumIter, Iterable},
    sea_query::extension::postgres::Type,
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
pub enum PhysicalAgent {
    Table,
    Id,
    Status,
    IP,
    Port,
    QubitCount,
    QubitUsing,
    CircuitDepth,
}

#[derive(DeriveIden, EnumIter)]
enum PhysicalAgentStatus {
    Table,
    Down,
    Running,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_type(
                Type::create()
                    .as_enum(PhysicalAgentStatus::Table)
                    .values(PhysicalAgentStatus::iter().skip(1))
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(PhysicalAgent::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PhysicalAgent::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(PhysicalAgent::Status)
                            .enumeration(
                                PhysicalAgentStatus::Table,
                                PhysicalAgentStatus::iter().skip(1),
                            )
                            .not_null(),
                    )
                    .col(ColumnDef::new(PhysicalAgent::IP).string().not_null())
                    .col(ColumnDef::new(PhysicalAgent::Port).integer().not_null())
                    .col(
                        ColumnDef::new(PhysicalAgent::QubitCount)
                            .unsigned()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PhysicalAgent::QubitUsing)
                            .unsigned()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PhysicalAgent::CircuitDepth)
                            .unsigned()
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
                    .table(PhysicalAgent::Table)
                    .if_exists()
                    .to_owned(),
            )
            .await
    }
}
