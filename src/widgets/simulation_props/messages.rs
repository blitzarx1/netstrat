use serde::{Deserialize, Serialize};

use crate::widgets::net_props::FrozenElements;

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

#[derive(Serialize, Deserialize)]
pub struct MessageOperationResult {
    signal_holders: FrozenElements,
}

impl MessageOperationResult {
    pub fn new(signal_holders: FrozenElements) -> Self {
        Self { signal_holders }
    }

    pub fn signal_holders(&self) -> &FrozenElements {
        &self.signal_holders
    }
}
