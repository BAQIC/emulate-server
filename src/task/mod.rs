use uuid::Uuid;

pub enum TaskStatus {
    Running,
    Succeeded,
    Failed,
    NotStarted,
}

pub struct Task {
    pub id: Uuid,
    pub qubits: usize,
    pub depth: usize,
    pub shots: usize,
    pub exec_shots: usize,
    pub vexec_shots: usize,
    pub task_status: TaskStatus,
}
