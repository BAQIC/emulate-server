use crate::entity::*;
use sea_orm::{ActiveValue, DbConn, EntityTrait};
use uuid::Uuid;

/// Output format.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Format {
    /// Tabular format.
    Tabular,

    /// JSON format.
    Json,
}

/// Output options.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Options {
    pub id: Uuid,
    pub format: Format,
    pub binary: bool,
    pub hexadecimal: bool,
    pub integer: bool,
    pub statevector: bool,
    pub probabilities: bool,
    pub times: bool,
    pub shots: Option<usize>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            format: Format::Json,
            binary: true,
            hexadecimal: true,
            integer: true,
            statevector: true,
            probabilities: true,
            times: true,
            shots: None,
        }
    }
}

impl Options {
    pub fn new(options: &qasmsim::options::Options) -> Self {
        Self {
            id: Uuid::new_v4(),
            format: match options.format {
                qasmsim::options::Format::Tabular => Format::Tabular,
                qasmsim::options::Format::Json => Format::Json,
            },
            binary: options.binary,
            hexadecimal: options.hexadecimal,
            integer: options.integer,
            statevector: options.statevector,
            probabilities: options.probabilities,
            times: options.times,
            shots: options.shots,
        }
    }

    pub async fn insert_to_db(&self, db: &DbConn) -> Result<(), sea_orm::prelude::DbErr> {
        options::Entity::insert(options::ActiveModel {
            id: ActiveValue::set(self.id),
            format: ActiveValue::set(match self.format {
                Format::Tabular => sea_orm_active_enums::Format::Tabular,
                Format::Json => sea_orm_active_enums::Format::Json,
            }),
            binary: ActiveValue::set(self.binary),
            hexadecimal: ActiveValue::set(self.hexadecimal),
            integer: ActiveValue::set(self.integer),
            statevector: ActiveValue::set(self.statevector),
            probabilities: ActiveValue::set(self.probabilities),
            times: ActiveValue::set(self.times),
            shots: ActiveValue::set(self.shots.map(|x| x as i32)),
        })
        .exec(db)
        .await
        .map(|_| ())
    }

    pub fn get_id(&self) -> Uuid {
        self.id
    }
}
