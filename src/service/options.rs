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
    pub shots: Option<i32>,
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
