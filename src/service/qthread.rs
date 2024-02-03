use super::resource::Resource;
use super::task::Task;
use crate::entity::*;
use sea_orm::{ActiveValue, DbConn, EntityTrait};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Qthread {
    /// all resources
    pub resource: Resource,

    /// queue of waitting tasks
    pub tasks: Vec<Uuid>,
}

impl Default for Qthread {
    fn default() -> Self {
        Self {
            resource: Resource::default(),
            tasks: Vec::new(),
        }
    }
}

impl Qthread {}
