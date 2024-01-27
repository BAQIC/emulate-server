use sea_orm::entity::prelude::*;

pub struct Resource {
    pub id: i32,
    pub uuid: Uuid,
}
