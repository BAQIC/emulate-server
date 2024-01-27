use sea_orm::prelude::Uuid;

pub struct Qthread {
    /// all resources this qthread can use
    pub resources: Vec<Uuid>,
}
