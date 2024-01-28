use crate::entity::{resource, resource::Entity as Resource};

pub struct ResourceQuery;

impl ResourceQuery {
    pub async fn find_resouce_by_id(db: &DbConn, id: &Uuid) -> Result<Option<resource::Model>, DbErr> {
        Resource::find_by_id(id).one(db).await
    }
}