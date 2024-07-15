use serde::Deserialize;
#[derive(Deserialize, Clone)]
pub struct QSchedulerConfig {
    pub sched_min_gran: u32,
    pub sched_min_depth: u32,
}

impl Default for QSchedulerConfig {
    fn default() -> Self {
        Self {
            sched_min_gran: 200,
            sched_min_depth: 10,
        }
    }
}

pub fn get_qsched_config(path: &str) -> QSchedulerConfig {
    let qsched_config = std::fs::read_to_string(path).unwrap();
    let qsched_config: QSchedulerConfig = serde_json::from_str(&qsched_config).unwrap();
    qsched_config
}
