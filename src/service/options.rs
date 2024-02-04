use crate::entity::*;
use sea_orm::{ActiveModelTrait, ActiveValue, DbConn};

pub struct Options;

impl Options {
    pub async fn add_options(
        db: &DbConn,
        options: options::Model,
    ) -> Result<options::Model, sea_orm::prelude::DbErr> {
        options::ActiveModel {
            id: ActiveValue::set(options.id.to_owned()),
            format: ActiveValue::set(options.format.to_owned()),
            binary: ActiveValue::set(options.binary.to_owned()),
            hexadecimal: ActiveValue::set(options.hexadecimal.to_owned()),
            integer: ActiveValue::set(options.integer.to_owned()),
            statevector: ActiveValue::set(options.statevector.to_owned()),
            probabilities: ActiveValue::set(options.probabilities.to_owned()),
            times: ActiveValue::set(options.times.to_owned()),
            shots: ActiveValue::set(options.shots.to_owned()),
        }
        .insert(db)
        .await
    }

    pub async fn add_qasm_options(
        db: &DbConn,
        options: &qasmsim::options::Options,
    ) -> Result<options::Model, sea_orm::prelude::DbErr> {
        let options = options::Model {
            id: uuid::Uuid::new_v4(),
            format: match options.format {
                qasmsim::options::Format::Json => sea_orm_active_enums::Format::Json,
                qasmsim::options::Format::Tabular => sea_orm_active_enums::Format::Tabular,
            },
            binary: options.binary,
            hexadecimal: options.hexadecimal,
            integer: options.integer,
            statevector: options.statevector,
            probabilities: options.probabilities,
            times: options.times,
            shots: options.shots.map(|x| x as i32),
        };
        Options::add_options(db, options).await
    }
}
