#[derive(Clone)]
pub struct QSchedulerConfig {
    pub shed_min_gran: u32,
    pub shed_min_depth: u32,
}

impl Default for QSchedulerConfig {
    fn default() -> Self {
        Self {
            shed_min_gran: 200,
            shed_min_depth: 10,
        }
    }
}
