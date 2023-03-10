use quick_error::quick_error;

quick_error! {
    #[derive(Debug)]
    pub enum CandlesError {
        Error {}
    }
}
