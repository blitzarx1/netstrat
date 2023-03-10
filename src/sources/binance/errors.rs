use quick_error::quick_error;

quick_error! {
    #[derive(Debug)]
    pub enum ClientError {
        Reqwest(err: reqwest::Error) {
            from()
            display("{}", err)
        }
        Serialization(err: serde_json::Error) {
            from()
            display("{}", err)
        }
    }
}
