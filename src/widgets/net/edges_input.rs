#[derive(PartialEq, Clone, Default)]
pub struct EdgesInput {
    pub input: String,
}

impl EdgesInput {
    pub fn splitted(&self) -> Vec<[String; 2]> {
        if self.input.is_empty() {
            return vec![];
        }
        self.input
            .split(',')
            .map(|el| {
                let mut splitted = el.split("->");
                let s = splitted.next().unwrap().trim().to_string();
                let e = splitted.next().unwrap().trim().to_string();
                [s, e]
            })
            .collect()
    }
}
