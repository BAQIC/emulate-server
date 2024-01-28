use sea_orm::prelude::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Qthread {
    /// all resources this qthread can use
    pub resources: Vec<Uuid>,

    /// all resource quotas this qthread can use
    pub quotas: Vec<u32>,

    /// all resource quotas this qthread is using
    pub current_quotas: Vec<u32>,
}

impl Default for Qthread {
    fn default() -> Self {
        Self {
            resources: vec![],
            quotas: vec![],
            current_quotas: vec![],
        }
    }
}

impl Qthread {
    pub fn new(resources: &Vec<Uuid>) -> Self {
        Self {
            resources: resources.clone(),
            quotas: resources.iter().map(|resource| 0).collect(),
            current_quotas: vec![0, resources.len() as u32],
        }
    }
}
