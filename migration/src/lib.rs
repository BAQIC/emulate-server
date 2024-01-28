pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_options;
mod m20220101_000002_create_agent;
mod m20220101_000003_create_resource;
mod m20220101_000004_create_task;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_options::Migration),
            Box::new(m20220101_000003_create_resource::Migration),
            Box::new(m20220101_000002_create_agent::Migration),
            Box::new(m20220101_000004_create_task::Migration),
        ]
    }
}
