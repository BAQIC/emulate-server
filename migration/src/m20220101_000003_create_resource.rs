use sea_orm_migration::{
    prelude::*,
    sea_orm::{EnumIter, Iterable},
    sea_query::extension::postgres::Type,
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
pub enum Resource {
    Table,
    Id,
    Status,
    MaximumAgentsNum,
    CurrentAgentsNum,
    AgentIds,
    CurrentAgentIds,
}

#[derive(DeriveIden, EnumIter)]
pub enum ResourceStatus {
    Table,
    FullyUsed,
    PartiallyUsed,
    Paused,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_type(
                Type::create()
                    .as_enum(ResourceStatus::Table)
                    .values(ResourceStatus::iter().skip(1))
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(Resource::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Resource::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(Resource::Status)
                            .enumeration(ResourceStatus::Table, ResourceStatus::iter().skip(1))
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Resource::MaximumAgentsNum)
                            .unsigned()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Resource::CurrentAgentsNum)
                            .unsigned()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Resource::AgentIds)
                            .array(ColumnType::Uuid)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Resource::CurrentAgentIds)
                            .array(ColumnType::Uuid)
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Resource::Table).if_exists().to_owned())
            .await
    }
}
