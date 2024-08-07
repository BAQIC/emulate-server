use serde::Deserialize;
#[derive(Deserialize, Clone)]
pub struct QSchedulerConfig {
    pub sched_min_gran: u32,
    pub sched_min_depth: u32,
    pub listen_ip: String,
    pub listen_port: u32,
    pub db_url: String,
    pub agent_file: String,
}

impl Default for QSchedulerConfig {
    fn default() -> Self {
        Self {
            sched_min_gran: 200,
            sched_min_depth: 10,
            listen_ip: "0.0.0.0".to_string(),
            listen_port: 3000,
            db_url: "".to_string(),
            agent_file: "".to_string(),
        }
    }
}

pub fn get_qsched_config(path: &str) -> QSchedulerConfig {
    let qsched_config = std::fs::read_to_string(path).unwrap();
    let qsched_config: QSchedulerConfig = serde_json::from_str(&qsched_config).unwrap();
    qsched_config
}
