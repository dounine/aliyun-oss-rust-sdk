use base64::Engine;
use base64::engine::general_purpose;
use hmac::{Hmac, Mac};
use reqwest::header::DATE;
use tracing::debug;
use crate::oss::{API, OSS, OSSInfo};
use crate::request::{RequestBuilder};

pub trait AuthAPI {
    fn sign<S: AsRef<str>>(
        &self,
        object: S,
        build: &RequestBuilder,
    ) -> String;

    fn oss_sign<S: AsRef<str>>(
        &self,
        object: S,
        build: &RequestBuilder,
    ) -> String;
}

impl<'a> AuthAPI for OSS {
    fn sign<S: AsRef<str>>(
        &self,
        key: S,
        build: &RequestBuilder,
    ) -> String {
        let date = build
            .headers
            .get(&DATE.to_string())
            .expect("Date header is required");
        let mut oss_headers = build
            .oss_headers
            .iter()
            .map(|(k, v)| (k.to_lowercase(), v))
            .collect::<Vec<_>>();

        oss_headers.sort_by(|a, b| a.0.cmp(&b.0));

        let oss_header_str = oss_headers
            .iter()
            .map(|(k, v)| format!("{}:{}", k, v))
            .collect::<Vec<_>>()
            .join("\n");

        let mut oss_resource_str = self.format_oss_resource_str(self.bucket().as_str(), key.as_ref());
        if build.parameters.len() > 0 {
            let mut params = build
                .parameters
                .iter()
                .collect::<Vec<_>>();
            params.sort_by(|a, b| a.0.cmp(&b.0));
            oss_resource_str = format!(
                "{}?{}",
                oss_resource_str,
                params
                    .into_iter()
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect::<Vec<_>>()
                    .join("&")
            );
        }
        let sign_str = format!(
            "{}\n{}\n{}\n{}\n{}{}",
            build.method,
            build.content_md5.clone().unwrap_or_default(),
            build.content_type.clone().unwrap_or_default(),
            date,
            oss_header_str,
            oss_resource_str,
        );
        debug!("sign_str: {}", sign_str);
        let mut hasher: Hmac<sha1::Sha1> = Hmac::new_from_slice(self.key_secret().as_bytes()).unwrap();
        hasher.update(sign_str.as_bytes());

        general_purpose::STANDARD.encode(&hasher.finalize().into_bytes())
    }

    fn oss_sign<S: AsRef<str>>(&self, object: S, build: &RequestBuilder) -> String {
        let sign_str_base64 = self.sign(object, build);
        format!("OSS {}:{}", self.key_id(), sign_str_base64)
    }
}
