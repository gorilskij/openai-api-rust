use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use reqwest::{Client, Response, Url};

#[derive(Clone, Debug)]
pub(crate) struct AsyncClient {
    client: Client,
    base_url: Url,
}

impl AsyncClient {
    pub(crate) fn new(token: &str, base_url: &str) -> crate::Result<Self> {
        let mut header_map = HeaderMap::new();
        let mut token = HeaderValue::from_str(&format!("Bearer {}", token))?;
        token.set_sensitive(true);
        header_map.insert(AUTHORIZATION, token);
        header_map.insert(CONTENT_TYPE, HeaderValue::from_str("application/json")?);
        let client = reqwest::ClientBuilder::new()
            .default_headers(header_map)
            .build()?;
        let base_url = Url::parse(base_url)?;
        Ok(Self { client, base_url })
    }

    #[cfg(test)]
    pub(crate) fn set_base_url(&mut self, base_url: &str) -> crate::Result<&mut Self> {
        self.base_url = Url::parse(base_url)?;
        Ok(self)
    }

    pub(crate) async fn get<T>(&self, endpoint: &str) -> crate::Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let url = self.base_url.join(endpoint)?;
        let request = self.client.get(url).build()?;
        let response = self.client.execute(request).await?;
        Self::handle_response::<T>(response).await
    }

    pub(crate) async fn post<B, R>(&self, endpoint: &str, body: B) -> crate::Result<R>
    where
        B: serde::ser::Serialize,
        R: serde::de::DeserializeOwned,
    {
        let url = self.base_url.join(endpoint)?;
        let body = serde_json::to_string(&body)?;
        let request = self.client.post(url).body(body).build()?;
        let response = self.client.execute(request).await?;
        Self::handle_response::<R>(response).await
    }

    async fn handle_response<T>(response: Response) -> crate::Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        if response.status().is_success() {
            Ok(response.json::<T>().await?)
        } else {
            let err = response.json::<crate::api::ErrorWrapper>().await?.error;
            Err(crate::Error::Api(err))
        }
    }
}
