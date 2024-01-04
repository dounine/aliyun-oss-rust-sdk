use std::collections::HashMap;
use tracing::debug;
use crate::auth::AuthAPI;
use crate::error::Error;
use crate::oss::{API, OSS};
use crate::request::{RequestBuilder, RequestType};

pub trait ObjectAPI {
    fn get_object<S: AsRef<str>>(
        &self,
        key: S,
        build: RequestBuilder,
    ) -> Result<Vec<u8>, Error>;
}

impl ObjectAPI for OSS {
    fn get_object<S: AsRef<str>>(&self, key: S, build: RequestBuilder) -> Result<Vec<u8>, Error> {
        let key = self.format_key(key);
        let mut header = HashMap::new();
        let time = chrono::Local::now().naive_local();
        header.insert("Date".to_string(), time.timestamp().to_string());
        let signature = self.oss_sign(
            &RequestType::Get,
            key.as_str(),
            "",
            &header,
            &build,
        );
        debug!("signature: {}", signature);
        Ok(vec![])
    }
}

