use tracing::debug;

#[derive(Clone, Debug)]
pub struct Rest {
    c: reqwest::Client,
}

impl Rest {
    pub fn new() -> Rest {
        Rest {
            c: reqwest::Client::new(),
        }
    }

    pub async fn get(&self, url: &str) -> Result<reqwest::Response, reqwest::Error> {
        let req = self.c.get(url);

        self.execute_request(req).await
    }

    pub async fn get_with_params(
        &self,
        url: &str,
        params: &[(&str, &str)],
    ) -> Result<reqwest::Response, reqwest::Error> {
        let req = self.c.get(url).query(params);

        self.execute_request(req).await
    }

    async fn execute_request(
        &self,
        req: reqwest::RequestBuilder,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let req_builded = req.build()?;
        debug!(
            "Sending request: method: {:?}; url: {:?}; headers: {:?}; body: {:?}.",
            req_builded.method(),
            req_builded.url().as_str(),
            req_builded.headers(),
            req_builded.body(),
        );

        self.c.execute(req_builded).await
    }
}
