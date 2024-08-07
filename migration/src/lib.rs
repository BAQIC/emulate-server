pub use sea_orm_migration::prelude::*;

mod create_physical_agent;
mod create_task;
mod create_task_active;
mod create_task_assignment;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(create_physical_agent::Migration),
            Box::new(create_task::Migration),
            Box::new(create_task_active::Migration),
            Box::new(create_task_assignment::Migration),
        ]
    }
}
