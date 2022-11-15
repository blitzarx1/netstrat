use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct MessageOperation {
    operation_type: OperationType,
}

impl MessageOperation {
    pub fn new(operation_type: OperationType) -> Self {
        Self { operation_type }
    }

    pub fn operation(&self) -> OperationType {
        self.operation_type.clone()
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub enum OperationType {
    BackStep,
    NextStep,
    Reset,
}
