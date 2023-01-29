use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Default, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ElementID {
    pub id: Uuid,
    pub name: String,
}
