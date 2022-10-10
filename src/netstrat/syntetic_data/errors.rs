use quick_error::quick_error;

quick_error! {
    #[derive(Debug)]
    pub enum Data {
        InputSize(size: usize) {
            display("invalid input size: {size}")
        }
        StepsNumber(steps: usize) {
            display("invalid number of steps: {steps}")
        }
    }
}
