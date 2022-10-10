use super::errors;

/// Data is a lazy wrapper for function. It computes function values only
/// when they are needed.
pub struct Data {
    input: Vec<f64>,
    f: Box<dyn Fn(f64) -> f64>,
    last_computed: Option<usize>,
    values: Vec<f64>,
}

impl Data {
    /// Input size must be >= 2.
    pub fn new(input: Vec<f64>, f: Box<dyn Fn(f64) -> f64>) -> Result<Self, errors::Data> {
        if input.len() < 2 {
            return Err(errors::Data::InputSize(input.len()));
        }

        let input_len = input.len();
        Ok(Self {
            last_computed: Default::default(),
            input,
            f,
            values: Vec::with_capacity(input_len),
        })
    }

    /// Counts derivative for x index in self.values.
    /// We use only n-2 values if self.values has size n.
    /// If x>n-2 returns None
    pub fn derivative(&mut self, x: usize) -> Option<f64> {
        if x > self.input.len() - 2 {
            return None;
        }

        let left = self.value(x)?;
        let right = self.value(x + 1)?;

        Some((right - left) / (self.input.get(x + 1)? - self.input.get(x)?).abs())
    }

    /// Computes values for inputs in range (self.last_computed, x] and stores results in self.values.
    /// Returns computed value for self.input at index x.
    pub fn value(&mut self, x: usize) -> Option<f64> {
        if x > self.input.len() - 1 {
            return None;
        }

        if self.last_computed.is_some() && x < self.last_computed.unwrap() {
            return Some(*self.values.get(x).unwrap());
        }

        let mut start = 0;
        if self.last_computed.is_some() {
            start = self.last_computed.unwrap() + 1;
        }

        self.values
            .extend((start..x + 1).map(|idx| self.f.as_ref()(*self.input.get(idx).unwrap())));

        self.last_computed = Some(x);

        Some(*self.values.get(x).unwrap())
    }

    pub fn extend_input(&mut self, input_extension: Vec<f64>) {
        self.input.extend(input_extension.iter());
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_value() {
        let mut data = Data::new(vec![1.0, 2.0, 3.0, 4.0, 5.0], Box::new(|x| x)).unwrap();

        assert_eq!(data.value(0).unwrap(), 1.0);
        assert_eq!(data.value(3).unwrap(), 4.0);

        assert_eq!(*data.values.get(0).unwrap(), 1.0);
        assert_eq!(*data.values.get(1).unwrap(), 2.0);
        assert_eq!(*data.values.get(2).unwrap(), 3.0);

        assert_eq!(data.values.get(4), None);

        assert_eq!(data.value(4).unwrap(), 5.0);
        assert_eq!(*data.values.get(4).unwrap(), 5.0);
    }

    #[test]
    fn derivative_line() {
        let mut data = Data::new(vec![1.0, 2.0, 3.0, 4.0, 5.0], Box::new(|x| x)).unwrap();

        assert_eq!(data.derivative(0).unwrap(), 1.0);
        assert_eq!(data.derivative(1).unwrap(), 1.0);
        assert_eq!(data.derivative(2).unwrap(), 1.0);
        assert_eq!(data.derivative(3).unwrap(), 1.0);
        assert_eq!(data.derivative(4), None)
    }

    #[test]
    fn derivative_parabolic() {
        let mut data = Data::new(vec![1.0, 2.0, 3.0, 4.0, 5.0], Box::new(|x| x * x)).unwrap();

        assert_eq!(data.derivative(0).unwrap(), 3.0);
        assert_eq!(data.derivative(1).unwrap(), 5.0);
        assert_eq!(data.derivative(2).unwrap(), 7.0);
        assert_eq!(data.derivative(3).unwrap(), 9.0);
        assert_eq!(data.derivative(4), None)
    }

    #[test]
    fn derivative_for_small_input_size() {
        let mut data = Data::new(vec![1.0, 2.0], Box::new(|x| x)).unwrap();

        assert_eq!(data.derivative(0).unwrap(), 1.0);
        assert_eq!(data.derivative(1), None);
    }
}
