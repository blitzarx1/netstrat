#[derive(Debug, Clone, Copy)]
pub enum Interval {
    Minute,
    Hour,
    Day,
}

impl Interval {
    pub fn as_str(&self) -> &str {
        match self {
            Interval::Minute => "1m",
            Interval::Hour => "1h",
            Interval::Day => "1d",
        }
    }
}

impl PartialEq for Interval {
    fn eq(&self, other: &Self) -> bool {
        format!("{:?}", self) == format!("{:?}", other)
    }
}
