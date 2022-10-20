use std::collections::HashSet;

#[derive(Clone, Default)]
pub struct Elements {
    pub elements: HashSet<(usize, usize)>,
    pub rows: HashSet<usize>,
    pub cols: HashSet<usize>,
}
