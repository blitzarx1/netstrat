use super::data::Data;
use super::errors;

const MIN_RESOLUTION_STEPS: usize = 10;

/// Dataset computes training and test set with dynamic step size based on the derivative value
#[derive(Debug)]
pub struct Dataset {
    data: Vec<[f64; 2]>,
    training_part: f64,
    mesh: Vec<f64>,
}

impl Dataset {
    pub fn new(
        name: &str,
        x0: f64,
        steps: usize,
        mesh_size: f64,
        f: Box<dyn Fn(f64) -> f64>,
    ) -> Result<Self, errors::Data> {
        if steps < 2 {
            return Err(errors::Data::StepsNumber(steps));
        }

        let mesh_len = steps * MIN_RESOLUTION_STEPS;
        let mut mesh = Vec::with_capacity(mesh_len);
        (0..mesh_len).for_each(|i| {
            mesh.push(x0 + i as f64 * mesh_size);
        });
        let data = Dataset::compute_data(mesh.clone(), steps, f)?;

        Ok(Self {
            data,
            mesh,
            training_part: 0.8,
        })
    }

    pub fn training_set(&self) -> Vec<[f64; 2]> {
        let fin_idx = (self.training_part * self.data.len() as f64).ceil() as usize;
        self.data[0..fin_idx].to_vec()
    }

    pub fn test_set(&self) -> Vec<[f64; 2]> {
        let fin_training_idx = (self.training_part * self.data.len() as f64).ceil() as usize;
        self.data[fin_training_idx..].to_vec()
    }

    fn compute_data(
        mesh: Vec<f64>,
        steps: usize,
        f: Box<dyn Fn(f64) -> f64>,
    ) -> Result<Vec<[f64; 2]>, errors::Data> {
        let mut res = Vec::with_capacity(steps);
        let mut data = Data::new(mesh.clone(), f)?;

        let max_derivative = (0..mesh.len() - 1)
            .map(|i| data.derivative(i).unwrap())
            .max_by(|left, right| left.partial_cmp(right).unwrap())
            .unwrap();

        let mut x_idx = 0;
        (0..steps).for_each(|_i| {
            res.push([*mesh.get(x_idx).unwrap(), data.value(x_idx).unwrap()]);

            let d = data.derivative(x_idx).unwrap();
            let coeff = 1f64 - (d / max_derivative).abs();
            x_idx += (MIN_RESOLUTION_STEPS as f64 * coeff).ceil() as usize
        });

        Ok(res)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_compute_data() {
        let steps = 10;
        let x0 = 3.0;
        let mesh_size = 0.5;
        let mesh_len = steps * MIN_RESOLUTION_STEPS;
        let mut mesh = Vec::with_capacity(mesh_len);
        (0..mesh_len).for_each(|i| {
            mesh.push(x0 + i as f64 * mesh_size);
        });

        Dataset::compute_data(mesh, steps, Box::new(|x| x.sin()));
    }

    #[test]
    fn test_dataset_split() {
        let dataset = Dataset::new("sin", 0.0, 100, 0.5, Box::new(|x| x.sin())).unwrap();
        let training_set = dataset.training_set();
        let test_set = dataset.test_set();

        assert_eq!(training_set.len(), 80);
        assert_eq!(test_set.len(), 20);
    }
}
