use tracing::{trace, debug};

#[derive(Clone, Debug)]
pub struct Rest {
    c: reqwest::blocking::Client,
}

impl Rest {
    pub fn new() -> Rest {
        Rest {
            c: reqwest::blocking::Client::new(),
        }
    }

    pub fn get(&self, url: &str) -> Result<reqwest::blocking::Response, reqwest::Error> {
        let req = self.c.get(url);

        self.execute_request(req)
    }

    pub fn get_with_params(
        &self,
        url: &str,
        params: &[(&str, &str)],
    ) -> Result<reqwest::blocking::Response, reqwest::Error> {
        let req = self.c.get(url).query(params);

        self.execute_request(req)
    }

    fn execute_request(
        &self,
        req: reqwest::blocking::RequestBuilder,
    ) -> Result<reqwest::blocking::Response, reqwest::Error> {
        let req_builded = req.build()?;
        debug!(
            "sending request: method: {:?}; url: {:?}; headers: {:?}; body: {:?}.",
            req_builded.method(),
            req_builded.url().as_str(),
            req_builded.headers(),
            req_builded.body(),
        );

        self.c.execute(req_builded)
    }
}
