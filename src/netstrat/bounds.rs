use std::cmp::{max, min, Ordering};

#[derive(Default, Debug, Clone, PartialEq, Eq, Ord)]
pub struct Bounds(i64, i64);

impl Bounds {
    /// merges 2 bounds into 1 bound if merge is possible
    pub fn union(&self, other: &Bounds) -> Option<Bounds> {
        if !(self.contains(other)
            || other.contains(self)
            || self.intersects(other)
            || self.neighbours(other))
        {
            return None;
        }

        Some(Bounds(min(self.0, other.0), max(self.1, other.1)))
    }

    /// intersects 2 bounds into 1 bound if intersect is possible
    pub fn intersect(&self, other: &Bounds) -> Option<Bounds> {
        if !(self.intersects(other) || self.contains(other) || other.contains(self)) {
            return None;
        }

        Some(Bounds(max(self.0, other.0), min(self.1, other.1)))
    }

    pub fn len(&self) -> i64 {
        self.1 - self.0
    }

    pub fn subtract(&self, other: &Bounds) -> Option<BoundsSet> {
        if other.contains(self) {
            return None;
        }

        if self < other {
            let b = Bounds(self.0, other.0 - 1);
            return Some(BoundsSet::new(vec![b]));
        }

        if self > other {
            let b = Bounds(other.1 + 1, self.1);
            return Some(BoundsSet::new(vec![b]));
        }

        if self.contains(other) {
            let mut res = BoundsSet::new(vec![]);
            if let Some(left_b) = self.subtract(&Bounds(other.0, self.1)) {
                res = res.concat(&left_b);
            }

            if let Some(right_b) = self.subtract(&Bounds(self.0, other.1)) {
                res = res.concat(&right_b);
            }

            if res.len() == 0 {
                return None;
            }

            return Some(res);
        }

        None
    }

    fn contains(&self, other: &Bounds) -> bool {
        self.0 <= other.0 && other.1 <= self.1
    }

    fn intersects(&self, other: &Bounds) -> bool {
        self.0 <= other.0 && other.0 <= self.1 || self.0 <= other.1 && other.1 <= self.1
    }

    fn neighbours(&self, other: &Bounds) -> bool {
        self.1 + 1 == other.0 || other.1 + 1 == self.0
    }
}

impl PartialOrd for Bounds {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.1 <= other.0 || (self.1 <= other.1 && self.0 <= other.0) {
            return Some(Ordering::Less);
        } else if other.1 <= self.0 || (other.1 <= self.1 && other.0 <= self.0) {
            return Some(Ordering::Greater);
        }

        Some(Ordering::Equal)
    }
}

#[cfg(test)]
mod bounds_tests {
    use super::*;

    #[test]
    fn test_eq() {
        assert_eq!(Bounds(1, 2), Bounds(1, 2));
        assert_ne!(Bounds(1, 4), Bounds(2, 3));
        assert_ne!(Bounds(1, 4), Bounds(2, 5));
        assert_ne!(Bounds(1, 4), Bounds(5, 6));
    }

    #[test]
    fn test_ord() {
        assert!(Bounds(1, 2) < Bounds(3, 4));
        assert!(Bounds(1, 2) < Bounds(2, 3));
        assert!(Bounds(1, 3) < Bounds(2, 3));
        assert!(Bounds(1, 3) > Bounds(0, 3));
    }

    #[test]
    fn test_union() {
        // containment
        assert_eq!(Bounds(3, 5).union(&Bounds(3, 4)), Some(Bounds(3, 5)));
        assert_eq!(Bounds(3, 5).union(&Bounds(2, 6)), Some(Bounds(2, 6)));

        // overlap
        assert_eq!(Bounds(3, 5).union(&Bounds(4, 6)), Some(Bounds(3, 6)));
        assert_eq!(Bounds(3, 5).union(&Bounds(2, 4)), Some(Bounds(2, 5)));

        // following
        assert_eq!(Bounds(3, 5).union(&Bounds(6, 7)), Some(Bounds(3, 7)));
        assert_eq!(Bounds(3, 5).union(&Bounds(1, 2)), Some(Bounds(1, 5)));

        // len = 1
        assert_eq!(Bounds(2, 2).union(&Bounds(3, 7)), Some(Bounds(2, 7)));
        assert_eq!(Bounds(2, 2).union(&Bounds(4, 7)), None);

        // no merge
        assert_eq!(Bounds(3, 5).union(&Bounds(8, 10)), None);
        assert_eq!(Bounds(3, 5).union(&Bounds(0, 1)), None);
    }

    #[test]
    fn test_intersect() {
        // containment
        assert_eq!(Bounds(3, 5).intersect(&Bounds(3, 4)), Some(Bounds(3, 4)));
        assert_eq!(Bounds(3, 5).intersect(&Bounds(2, 6)), Some(Bounds(3, 5)));

        // overlap
        assert_eq!(Bounds(3, 5).intersect(&Bounds(4, 6)), Some(Bounds(4, 5)));
        assert_eq!(Bounds(3, 5).intersect(&Bounds(2, 4)), Some(Bounds(3, 4)));

        // following
        assert_eq!(Bounds(3, 5).intersect(&Bounds(6, 7)), None);
        assert_eq!(Bounds(3, 5).intersect(&Bounds(1, 2)), None);

        // len = 1
        assert_eq!(Bounds(2, 2).intersect(&Bounds(3, 7)), None);
        assert_eq!(Bounds(2, 2).intersect(&Bounds(2, 7)), Some(Bounds(2, 2)));
        assert_eq!(Bounds(2, 2).intersect(&Bounds(2, 7)), Some(Bounds(2, 2)));

        // gap between
        assert_eq!(Bounds(3, 5).intersect(&Bounds(8, 10)), None);
        assert_eq!(Bounds(3, 5).intersect(&Bounds(0, 1)), None);
    }

    #[test]
    fn test_subtract() {
        // no relation
        assert_eq!(
            Bounds(2, 5).subtract(&Bounds(6, 6)),
            Some(BoundsSet::new(vec![Bounds(2, 5)]))
        );

        // overlap
        assert_eq!(
            Bounds(2, 5).subtract(&Bounds(4, 6)),
            Some(BoundsSet::new(vec![Bounds(2, 3)]))
        );
        assert_eq!(
            Bounds(2, 5).subtract(&Bounds(1, 3)),
            Some(BoundsSet::new(vec![Bounds(4, 5)]))
        );
        assert_eq!(
            Bounds(2, 5).subtract(&Bounds(3, 5)),
            Some(BoundsSet::new(vec![Bounds(2, 2)]))
        );

        // containment in other
        assert_eq!(Bounds(2, 5).subtract(&Bounds(1, 6)), None);
        assert_eq!(Bounds(2, 5).subtract(&Bounds(2, 5)), None);

        // containment in self
        assert_eq!(
            Bounds(2, 6).subtract(&Bounds(3, 4)),
            Some(BoundsSet::new(vec![Bounds(2, 2), Bounds(5, 6)]))
        );
        assert_eq!(
            Bounds(2, 7).subtract(&Bounds(4, 5)),
            Some(BoundsSet::new(vec![Bounds(2, 3), Bounds(6, 7)]))
        );
        let res = Bounds(0, 20).subtract(&Bounds(1, 10));
        assert_eq!(
            res,
            Some(BoundsSet::new(vec![Bounds(0, 0), Bounds(11, 20)]))
        );
    }
}

#[derive(Default, Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct BoundsSet {
    vals: Vec<Bounds>,
}

impl BoundsSet {
    pub fn new(vals: Vec<Bounds>) -> Self {
        Self { vals }
    }

    pub fn len(&self) -> usize {
        self.vals.len()
    }

    pub fn concat(&self, other: &Self) -> Self {
        let mut vals = self.vals.clone();
        vals.extend_from_slice(&other.vals);
        Self { vals }
    }

    /// concats, sorts and unions 2 bounds sequences
    pub fn union(&self, other: &BoundsSet) -> Self {
        let mut new_vals = self.concat(other).vals;

        new_vals.sort();

        Self {
            vals: new_vals.iter().fold(Vec::new(), |mut acc, v| {
                if acc.is_empty() {
                    acc.push(v.clone());

                    return acc;
                }

                let last = acc.last_mut().unwrap();
                if let Some(union) = last.union(v) {
                    *last = union;
                } else {
                    acc.push(v.clone());
                }

                acc
            }),
        }
    }

    /// concats and intersects 2 bounds sequences
    pub fn diff(&self, other: &BoundsSet) -> Option<BoundsSet> {
        if other.len() == 0 {
            return Some(self.clone());
        }

        let mut differences = BoundsSet::new(vec![]);
        self.vals.iter().for_each(|s| {
            other.vals.iter().for_each(|o| {
                if let Some(diff) = s.subtract(o) {
                    differences = differences.concat(&diff);
                };
            });
        });

        if differences.len() == 0 {
            return None;
        }

        let mut res = BoundsSet::new(vec![]);
        for i in 0..differences.len() {
            for j in i + 1..differences.len() {
                if let Some(intersection) = differences.vals[i].intersect(&differences.vals[j]) {
                    res = res.concat(&BoundsSet::new(vec![intersection]));
                }
            }
        }

        if res.len() == 0 {
            return Some(res.concat(&differences))
        }

        Some(res)
    }

    pub fn sort(&self) -> Self {
        let mut new_vals = self.vals.clone();
        new_vals.sort();

        Self { vals: new_vals }
    }
}

#[cfg(test)]
mod bounds_sequence_tests {
    use super::*;

    #[test]
    fn test_sort() {
        assert_eq!(
            BoundsSet::new(vec![Bounds(6, 10), Bounds(3, 5), Bounds(1, 2)]).sort(),
            BoundsSet::new(vec![Bounds(1, 2), Bounds(3, 5), Bounds(6, 10)])
        );

        assert_eq!(
            BoundsSet::new(vec![Bounds(6, 10), Bounds(2, 8), Bounds(1, 2)]).sort(),
            BoundsSet::new(vec![Bounds(1, 2), Bounds(2, 8), Bounds(6, 10)])
        );
    }

    #[test]
    fn test_union() {
        assert_eq!(
            BoundsSet::new(vec![Bounds(6, 10)])
                .union(&BoundsSet::new(vec![Bounds(3, 5), Bounds(1, 2)])),
            BoundsSet::new(vec![Bounds(1, 10)])
        );

        assert_eq!(
            BoundsSet::new(vec![Bounds(6, 10)])
                .union(&BoundsSet::new(vec![Bounds(2, 5), Bounds(1, 2)])),
            BoundsSet::new(vec![Bounds(1, 10)])
        );

        assert_eq!(
            BoundsSet::new(vec![Bounds(0, 1)])
                .union(&BoundsSet::new(vec![Bounds(3, 6), Bounds(4, 5)])),
            BoundsSet::new(vec![Bounds(0, 1), Bounds(3, 6)])
        );

        assert_eq!(
            BoundsSet::new(vec![Bounds(4, 5)])
                .union(&BoundsSet::new(vec![Bounds(0, 1), Bounds(3, 6)])),
            BoundsSet::new(vec![Bounds(0, 1), Bounds(3, 6)])
        );
    }

    // TODO: add test cases
    #[test]
    fn test_diff() {
        // zero
        assert_eq!(
            BoundsSet::new(vec![Bounds(1, 10)]).diff(&BoundsSet::new(vec![])),
            Some(BoundsSet::new(vec![Bounds(1, 10)])),
        );

        // same
        assert_eq!(
            BoundsSet::new(vec![Bounds(1, 10)]).diff(&BoundsSet::new(vec![Bounds(1, 10)])),
            None,
        );

        // standart
        assert_eq!(
            BoundsSet::new(vec![Bounds(0, 20)]).diff(&BoundsSet::new(vec![Bounds(1, 10)])),
            Some(BoundsSet::new(vec![Bounds(0, 0), Bounds(11, 20)])),
        );

        // standart2
        assert_eq!(
            BoundsSet::new(vec![Bounds(0, 20)])
                .diff(&BoundsSet::new(vec![Bounds(1, 10), Bounds(13, 15)])),
            Some(BoundsSet::new(vec![
                Bounds(0, 0),
                Bounds(11, 12),
                Bounds(16, 20)
            ])),
        );
    }
}
