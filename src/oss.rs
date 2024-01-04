use std::collections::HashMap;

use tracing::debug;

use crate::auth::AuthAPI;
use crate::request::RequestBuilder;

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
    fn sign_url<S: AsRef<str>>(&self, key: S, build: RequestBuilder) -> String;
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
    /// 签名URL,分享下载
    /// # 使用例子
    ///
    /// ```
    /// use aliyun_oss_rust_sdk::oss::{OSS, OSSAPI, RequestBuilder};
    /// let oss = OSS::from_env();//也可以使用OSS::new()方法传递参数
    /// let build = RequestBuilder::new()
    ///    .expire(60) //60秒链接过期
    ///   .oss_download_speed_limit(30);//限速30kb
    /// let download_url = oss.sign_url_with_endpoint(
    ///     "/ipas/cn/-10/ipadump.com_imem内存修改器_1.0.0.ipa",
    ///     build
    ///     );
    ///  println!("download_url: {}", download_url);
    /// ```
    fn sign_url_with_endpoint(&self, key: &str, build: RequestBuilder) -> String {
        format!("{}.{}{}", self.bucket(), self.endpoint(), self.sign_url(key, build))
    }

    /// 签名URL,分享下载
    /// 使用自定义域名
    /// # 使用例子
    ///
    /// ```
    /// use aliyun_oss_rust_sdk::oss::{OSS, OSSAPI, RequestBuilder};
    /// let oss = OSS::from_env();//也可以使用OSS::new()方法传递参数
    /// let build = RequestBuilder::new()
    ///    .expire(60) //60秒链接过期
    ///   .oss_download_speed_limit(30);//限速30kb
    /// let download_url = oss.sign_url_with_cdn(
    ///     "https://mydomain.com",
    ///     "/ipas/cn/-10/ipadump.com_imem内存修改器_1.0.0.ipa",
    ///     build
    ///     );
    ///  println!("download_url: {}", download_url);
    /// ```
    fn sign_url_with_cdn(&self, cdn: &str, key: &str, build: RequestBuilder) -> String {
        let download_url = format!("{}{}", cdn, self.sign_url(key, build));
        debug!("download_url: {}", download_url);
        download_url
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
    fn sign_url<S: AsRef<str>>(&self, key: S, build: RequestBuilder) -> String {
        let key = self.format_key(key);
        let mut header = HashMap::new();
        let expiration = chrono::Local::now().naive_local() + chrono::Duration::seconds(build.expire);
        header.insert("Date".to_string(), expiration.timestamp().to_string());
        let signature = self.sign(
            &build.method,
            key.as_str(),
            &header,
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
}

#[cfg(test)]
mod tests {
    use crate::object::ObjectAPI;

    use super::*;

    #[inline]
    fn init_log() {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .with_line_number(true)
            .init();
    }

    #[test]
    fn test_sign() {
        init_log();
        let oss = OSS::from_env();
        let build = RequestBuilder::new()
            .expire(60)
            .oss_download_speed_limit(30);
        oss.sign_url_with_cdn(
            "http://cdn.ipadump.com",
            "/ipas/cn/-10/ipadump.com_imem内存修改器_1.0.0.ipa",
            build,
        );
    }

    #[test]
    fn test_oss_sign() {
        init_log();
        let oss = OSS::from_env();
        let build = RequestBuilder::new()
            .expire(60)
            .oss_download_speed_limit(30);
        oss.get_object("/hello.txt", build).expect("TODO: panic message");
    }
}