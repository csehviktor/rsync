use reqwest::{
    Method,
    blocking::{Client, Response},
};

use crate::error::*;
use crate::persistance::config::Config;
use crate::r2::sigv4;
use crate::r2::xml;

pub struct R2Client<'a> {
    config: &'a Config,
    endpoint: String,
    host: String,
    http: Client,
}

impl<'a> R2Client<'a> {
    pub fn new(config: &'a Config) -> Self {
        let host = format!("{}.r2.cloudflarestorage.com", config.credentials.account_id);
        let endpoint = format!("https://{host}");

        Self {
            config,
            endpoint,
            host,
            http: Client::new(),
        }
    }

    pub fn validate(&self) -> RsyncResult<()> {
        self.request(Method::HEAD, &self.bucket_path(), &[], Vec::new())?;
        Ok(())
    }

    pub fn get(&self, key: &str) -> RsyncResult<Vec<u8>> {
        let response = self.request(Method::GET, &self.object_path(key), &[], Vec::new())?;
        Ok(response.bytes()?.to_vec())
    }

    pub fn put(&self, key: &str, body: Vec<u8>) -> RsyncResult<()> {
        self.request(Method::PUT, &self.object_path(key), &[], body)?;
        Ok(())
    }

    pub fn delete(&self, key: &str) -> RsyncResult<()> {
        self.request(Method::DELETE, &self.object_path(key), &[], Vec::new())?;
        Ok(())
    }

    pub fn list(&self) -> RsyncResult<Vec<String>> {
        let mut keys = Vec::new();
        let mut token: Option<String> = None;

        loop {
            let mut params: Vec<(&str, &str)> = vec![("list-type", "2")];

            if let Some(current) = &token {
                params.push(("continuation-token", current));
            }

            let body = self
                .request(Method::GET, &self.bucket_path(), &params, Vec::new())?
                .text()?;

            let (mut page, next) = xml::parse_list(&body)?;
            keys.append(&mut page);

            match next {
                Some(next) => token = Some(next),
                None => return Ok(keys),
            }
        }
    }

    fn request(
        &self,
        method: Method,
        path: &str,
        params: &[(&str, &str)],
        body: Vec<u8>,
    ) -> RsyncResult<Response> {
        let uri = sigv4::encode_uri_path(path);
        let query = sigv4::canonical_query(params);
        let payload_hash = sigv4::hash(&body);

        let signed = sigv4::sign(
            method.as_str(),
            &self.host,
            &uri,
            &query,
            &payload_hash,
            &self.config.credentials,
        );

        let mut url = format!("{}{}", self.endpoint, uri);

        if !query.is_empty() {
            url.push('?');
            url.push_str(&query);
        }

        let response = self
            .http
            .request(method, url)
            .header("x-amz-content-sha256", payload_hash.as_str())
            .header("x-amz-date", signed.amz_date.as_str())
            .header("authorization", signed.authorization.as_str())
            .body(body)
            .send()?;

        let status = response.status();

        if !status.is_success() {
            return Err(Error::R2(status.to_string()));
        }

        Ok(response)
    }

    #[inline]
    fn bucket_path(&self) -> String {
        format!("/{}", self.config.settings.bucket)
    }

    #[inline]
    fn object_path(&self, key: &str) -> String {
        format!("/{}/{}", self.config.settings.bucket, key)
    }
}
