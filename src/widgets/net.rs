#[derive(PartialEq, Clone)]
struct ConeSettings {
    node_name: String,
    cone_dir: ConeDir,
    cone_type: ConeType,
    max_steps: i32,
}

impl Default for ConeSettings {
    fn default() -> Self {
        Self {
            cone_dir: ConeDir::Plus,
            max_steps: -1,
            cone_type: ConeType::Custom,
            node_name: Default::default(),
        }
    }
}

#[derive(PartialEq, Clone)]
enum ConeDir {
    /// Go along arrow from head to tail
    Minus,
    /// Go along arrow from tail to head
    Plus,
}

#[derive(PartialEq, Clone)]
enum ConeType {
    Custom,
    Initial,
    Final,
}

#[derive(Default)]
struct ButtonClicks {
    reset: bool,
    create: bool,
    diamond_filter: bool,
    color_cones: bool,
    color_cycles: bool,
    export_dot: bool,
    delete_cone: bool,
    delete_cycles: bool,
}
