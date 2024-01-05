use std::collections::HashMap;
use reqwest::header::{AUTHORIZATION, DATE, HeaderMap};
use strum_macros::Display;
use anyhow::Result;
use chrono::{DateTime, Utc};

use tracing::debug;

use crate::auth::AuthAPI;
use crate::request::{RequestBuilder, RequestType};

/// OSS配置
pub struct OSS {
    key_id: String,
    key_secret: String,
    endpoint: String,
    bucket: String,
}

pub trait OSSInfo {
    fn endpoint(&self) -> String;
    fn bucket(&self) -> String;
    fn key_id(&self) -> String;
    fn key_secret(&self) -> String;
}

pub trait API {
    fn sign_url<S: AsRef<str>>(&self, key: S, build: &mut RequestBuilder) -> String;
    fn key_urlencode<S: AsRef<str>>(&self, key: S) -> String {
        key
            .as_ref()
            .split("/")
            .map(|x| urlencoding::encode(x))
            .collect::<Vec<_>>()
            .join("/")
    }

    fn format_key<S: AsRef<str>>(&self, key: S) -> String {
        let key = key.as_ref();
        if key.starts_with("/") {
            key.to_string()
        } else {
            format!("/{}", key)
        }
    }

    fn format_oss_resource_str<S: AsRef<str>>(&self, bucket: S, key: S) -> String;
}

pub trait OSSAPI: OSSInfo + API {
    /// 签名下载URL
    ///
    /// # 使用例子
    ///
    /// ```
    /// use aliyun_oss_rust_sdk::oss::{OSS, OSSAPI, RequestBuilder};
    /// let oss = OSS::from_env();//也可以使用OSS::new()方法传递参数
    /// let build = RequestBuilder::new()
    ///    //.with_cdn("https://mydomain.com")
    ///    .expire(60) //60秒链接过期
    ///   .oss_download_speed_limit(30);//限速30kb
    /// let download_url = oss.sign_download_url(
    ///     "/ipas/cn/-10/ipadump.com_imem内存修改器_1.0.0.ipa",
    ///     &build
    ///     );
    ///  println!("download_url: {}", download_url);
    /// ```
    fn sign_download_url(&self, key: &str, build: &mut RequestBuilder) -> String {
        let sign = self.sign_url(key, build);
        if let Some(cdn) = &build.cdn {
            let download_url = format!("{}{}", cdn, sign);
            debug!("download_url: {}", download_url);
            download_url
        } else {
            let download_url = format!("{}.{}{}", self.bucket(), self.endpoint(), sign);
            debug!("download_url: {}", download_url);
            download_url
        }
    }
}

impl OSSAPI for OSS {}

impl OSSInfo for OSS {
    fn endpoint(&self) -> String {
        self.endpoint.clone()
    }
    fn bucket(&self) -> String {
        self.bucket.clone()
    }

    fn key_id(&self) -> String {
        self.key_id.clone()
    }

    fn key_secret(&self) -> String {
        self.key_secret.clone()
    }
}

impl API for OSS {
    fn sign_url<S: AsRef<str>>(&self, key: S, build: &mut RequestBuilder) -> String {
        let key = self.format_key(key);
        let expiration = chrono::Local::now() + chrono::Duration::seconds(build.expire);
        build.headers.insert(DATE.to_string(), expiration.timestamp().to_string());
        let signature = self.sign(
            key.as_str(),
            &build,
        );
        debug!("signature: {}", signature);
        let mut query_parameters = HashMap::new();
        query_parameters.insert("Expires".to_string(), expiration.timestamp().to_string());
        query_parameters.insert("OSSAccessKeyId".to_string(), self.key_id.to_string());
        query_parameters.insert("Signature".to_string(), urlencoding::encode(&signature).into_owned());
        build.parameters.iter().for_each(|(k, v)| {
            query_parameters.insert(k.to_string(), urlencoding::encode(v).into_owned());
        });

        let mut params = query_parameters
            .into_iter()
            .filter(|(k, _)| k != "x-oss-ac-source-ip")
            .collect::<Vec<_>>();

        params.sort_by(|a, b| a.0.cmp(&b.0));

        format!(
            "{}?{}",
            self.key_urlencode(key),
            params.into_iter().map(|(k, v)| format!("{}={}", k, v)).collect::<Vec<String>>().join("&")
        )
    }

    fn format_oss_resource_str<S: AsRef<str>>(&self, bucket: S, key: S) -> String {
        let bucket = bucket.as_ref();
        if bucket == "" {
            format!("/{}", bucket)
        } else {
            format!("/{}{}", bucket, key.as_ref())
        }
    }
}

impl<'a> OSS {
    pub fn new<S: Into<String>>(key_id: S, key_secret: S, endpoint: S, bucket: S) -> Self {
        OSS {
            key_id: key_id.into(),
            key_secret: key_secret.into(),
            endpoint: endpoint.into(),
            bucket: bucket.into(),
        }
    }

    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();
        let key_id = dotenvy::var("OSS_KEY_ID").expect("OSS_KEY_ID not found");
        let key_secret = dotenvy::var("OSS_KEY_SECRET").expect("OSS_KEY_SECRET not found");
        let endpoint = dotenvy::var("OSS_ENDPOINT").expect("OSS_ENDPOINT not found");
        let bucket = dotenvy::var("OSS_BUCKET").expect("OSS_BUCKET not found");
        OSS::new(key_id, key_secret, endpoint, bucket)
    }
    pub fn format_host<S: AsRef<str>>(&self, bucket: S, key: S) -> String {
        let key = if key.as_ref().starts_with("/") {
            key.as_ref().to_string()
        } else {
            format!("/{}", key.as_ref())
        };
        if self.endpoint().starts_with("https") {
            format!(
                "https://{}.{}{}",
                bucket.as_ref(),
                self.endpoint().replacen("https://", "", 1),
                key,
            )
        } else {
            format!(
                "http://{}.{}{}",
                bucket.as_ref(),
                self.endpoint().replacen("http://", "", 1),
                key,
            )
        }
    }

    pub fn build_request<S: AsRef<str>>(&self, key: S, mut build: RequestBuilder) -> Result<(String, HeaderMap)> {
        let host = self.format_host(self.bucket(), key.as_ref().to_string());
        let mut header = HeaderMap::new();
        let date = self.date();
        header.insert(DATE, date.parse()?);
        build.headers.insert(DATE.to_string(), date);
        let key = key.as_ref();
        let authorization = self.oss_sign(
            key,
            &build,
        );
        header.insert(AUTHORIZATION, authorization.parse()?);
        Ok((host, header))
    }
    pub fn date(&self) -> String {
        let now: DateTime<Utc> = Utc::now();
        now.format("%a, %d %b %Y %T GMT").to_string()
    }
}

// use thiserror::Error;
//
// #[derive(Error, Debug, Display)]
// pub enum FileError {
//     IoError(#[from] std::io::Error),
// }
//
// #[derive(Error, Debug, Display)]
// pub enum MyError {
//     IoError(#[from] FileError)
// }


// impl From<std::io::Error> for MyError {
//     fn from(err: std::io::Error) -> Self {
//         MyError::IoError(FileError::IoError(err))
//     }
// }

#[cfg(test)]
mod tests {
    use std::io::Read;
    use chrono::{DateTime, Utc};
    use strum_macros::Display;
    use crate::object::ObjectAPI;

    use super::*;

    #[inline]
    fn init_log() {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .with_line_number(true)
            .init();
    }

    pub fn date() -> String {
        let now: chrono::DateTime<chrono::Local> = chrono::Local::now();
        now.format("%a, %d %b %Y %T GMT").to_string()
    }

    fn pp<S: AsRef<str>>(s: S) {
        println!("{}", s.as_ref());
    }


    fn open_file(file_name: &str) -> anyhow::Result<String, anyhow::Error> {
        let mut file = std::fs::File::open(file_name)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Ok(contents)
    }

    #[test]
    fn test_read_file() {
        open_file("a").unwrap();
    }

    #[test]
    fn test_sign() {
        let aa = &"".to_string();
        pp(aa);
        println!("{}", date());
        init_log();
        let oss = OSS::from_env();
        let mut build = RequestBuilder::new()
            .with_cdn("http://cdn.ipadump.com")
            .expire(60)
            .oss_download_speed_limit(30);
        oss.sign_download_url(
            "/ipas/cn/-10/ipadump.com_imem内存修改器_1.0.0.ipa",
            &mut build,
        );
    }
}