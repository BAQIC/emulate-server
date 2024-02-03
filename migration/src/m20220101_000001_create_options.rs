use sea_orm_migration::{
    prelude::*,
    sea_orm::{EnumIter, Iterable},
    sea_query::extension::postgres::Type,
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden, EnumIter)]
pub enum Format {
    Table,
    Json,
    Tabular,
}

#[derive(DeriveIden)]
pub enum Options {
    Table,
    Id,
    Format,
    Binary,
    Hexadecimal,
    Integer,
    Statevector,
    Probabilities,
    Times,
    Shots,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_type(
                Type::create()
                    .as_enum(Format::Table)
                    .values(Format::iter().skip(1))
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(Options::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Options::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(Options::Format)
                            .enumeration(Format::Table, Format::iter().skip(1))
                            .not_null(),
                    )
                    .col(ColumnDef::new(Options::Binary).boolean().not_null())
                    .col(ColumnDef::new(Options::Hexadecimal).boolean().not_null())
                    .col(ColumnDef::new(Options::Integer).boolean().not_null())
                    .col(ColumnDef::new(Options::Statevector).boolean().not_null())
                    .col(ColumnDef::new(Options::Probabilities).boolean().not_null())
                    .col(ColumnDef::new(Options::Times).boolean().not_null())
                    .col(ColumnDef::new(Options::Shots).unsigned().null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Options::Table).if_exists().to_owned())
            .await
    }
}
