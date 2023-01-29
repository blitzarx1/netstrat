use super::{nodes_input::NodesInput, settings::ConeSettings};

#[derive(PartialEq, Clone)]
pub struct ConeSettingsInputs {
    pub cone_type: ConeType,
    pub settings: Vec<ConeInput>,
}

impl Default for ConeSettingsInputs {
    fn default() -> Self {
        Self {
            cone_type: ConeType::Custom,
            settings: vec![ConeInput::default()],
        }
    }
}

#[derive(PartialEq, Clone, Default)]
pub struct ConeInput {
    pub nodes_names: NodesInput,
    pub cone_settings: ConeSettings,
}

impl ConeInput {
    pub fn prepare_settings(&self) -> ConeSettings {
        ConeSettings {
            roots_names: self.nodes_names.splitted(),
            dir: self.cone_settings.dir,
            max_steps: self.cone_settings.max_steps,
        }
    }
}

#[derive(PartialEq, Clone)]
pub enum ConeType {
    Custom,
    Initial,
    Final,
}
