use super::{create_physical_agent::PhysicalAgent, create_task::Task};
use sea_orm_migration::{
    prelude::*,
    sea_orm::{EnumIter, Iterable},
    sea_query::extension::postgres::Type,
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
pub enum TaskAssignment {
    Table,
    Id,
    TaskId,
    AgentId,
    AssignmentStatus,
}

#[derive(DeriveIden, EnumIter)]
pub enum AssignmentStatus {
    Table,
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
                    .as_enum(AssignmentStatus::Table)
                    .values(AssignmentStatus::iter().skip(1))
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(TaskAssignment::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TaskAssignment::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(TaskAssignment::AgentId).uuid().not_null())
                    .col(ColumnDef::new(TaskAssignment::TaskId).uuid().not_null())
                    .col(
                        ColumnDef::new(TaskAssignment::AssignmentStatus)
                            .enumeration(AssignmentStatus::Table, AssignmentStatus::iter().skip(1))
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_task_assignment_agent_id")
                            .from_col(TaskAssignment::AgentId)
                            .to(PhysicalAgent::Table, PhysicalAgent::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_task_assignment_task_id")
                            .from_col(TaskAssignment::TaskId)
                            .to(Task::Table, Task::Id),
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
