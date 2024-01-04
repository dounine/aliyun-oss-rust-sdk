use std::collections::HashMap;
use base64::{engine::general_purpose, Engine as _};
use hmac::{Hmac, Mac};
use strum_macros::{Display, EnumString};

/// OSS配置
pub struct OSS {
    key_id: String,
    key_secret: String,
    endpoint: String,
    bucket: String,
}

pub struct RequestBuilder {
    expire: Seconds,
    parameters: HashMap<String, String>,
    content_type: Option<String>,
    content_md5: Option<String>,
    oss_headers: HashMap<String, String>,
}

impl RequestBuilder {
    pub fn new() -> Self {
        RequestBuilder {
            expire: 60,
            parameters: HashMap::new(),
            content_type: None,
            content_md5: None,
            oss_headers: HashMap::new(),
        }
    }
    pub fn expire(mut self, expire: Seconds) -> Self {
        self.expire = expire;
        self
    }
    pub fn response_content_disposition(mut self, file_name: &str) -> Self {
        self.parameters.insert("response-content-disposition".to_string(), format!("attachment;filename={}", file_name));
        self
    }
    pub fn response_content_encoding(mut self, encoding: &str) -> Self {
        self.parameters.insert("response-content-encoding".to_string(), encoding.to_string());
        self
    }
    pub fn oss_download_speed_limit(mut self, speed: i32) -> Self {
        assert!(speed >= 30, "speed must be greater than 30kb");
        self.parameters.insert("x-oss-traffic-limit".to_string(), (speed * 1024 * 8).to_string());
        self
    }
    pub fn oss_download_allow_ip(mut self, ip: &str, mask: i32) -> Self {
        self.parameters.insert("x-oss-ac-source-ip".to_string(), ip.to_string());
        self.parameters.insert("x-oss-ac-subnet-mask".to_string(), mask.to_string());
        self
    }
    pub fn oss_ac_forward_allow(mut self) -> Self {
        self.parameters.insert("x-oss-ac-forwarded-for".to_string(), "true".to_string());
        self
    }
    pub fn oss_header_put(mut self, key: &str, value: &str) -> Self {
        self.oss_headers.insert(key.to_string(), value.to_string());
        self
    }
}

type Seconds = i64;

pub trait OSSInfo {
    fn endpoint(&self) -> String;
    fn bucket(&self) -> String;
}

pub trait API {
    fn sign_url(&self, key: &str, build: RequestBuilder) -> String;
    fn key_urlencode(&self, key: &str) -> String {
        key
            .split("/")
            .map(|x| urlencoding::encode(x))
            .collect::<Vec<_>>()
            .join("/")
    }
}

pub trait OSSAPI: OSSInfo + API {
    /// 签名URL,分享下载
    /// # 使用例子
    ///
    /// ```
    /// use aliyun_oss_rust_sdk::oss::{OSS, RequestBuilder};
    /// use aliyun_oss_rust_sdk::OSSAPI;
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
    /// use aliyun_oss_rust_sdk::oss::{OSS, RequestBuilder};
    /// use aliyun_oss_rust_sdk::OSSAPI;
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
        format!("{}{}", cdn, self.sign_url(key, build))
    }
}

impl OSSAPI for OSS {}

pub trait AuthAPI {
    fn sign(
        &self,
        verb: &str,
        object: &str,
        oss_resources: &str,
        headers: &HashMap<String, String>,
        build: &RequestBuilder,
    ) -> String;
}

impl OSSInfo for OSS {
    fn endpoint(&self) -> String {
        self.endpoint.clone()
    }
    fn bucket(&self) -> String {
        self.bucket.clone()
    }
}

impl API for OSS {
    fn sign_url(&self, key: &str, build: RequestBuilder) -> String {
        let object = if key.starts_with("/") {
            key.to_string()
        } else {
            format!("/{}", key)
        };
        let mut header = HashMap::new();
        let expiration = chrono::Local::now().naive_local() + chrono::Duration::seconds(build.expire);
        header.insert("Date".to_string(), expiration.timestamp().to_string());
        let signature = self.sign(
            RequestType::Get.to_string().as_str(),
            object.as_str(),
            "",
            &header,
            &build,
        );
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
}

impl<'a> AuthAPI for OSS {
    fn sign(
        &self,
        verb: &str,
        key: &str,
        oss_resources: &str,
        headers: &HashMap<String, String>,
        build: &RequestBuilder,
    ) -> String {
        let date = headers
            .get("Date")
            .map_or("", |x| x);
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

        let mut oss_resource_str = get_oss_resource_str(self.bucket.as_str(), key, oss_resources);
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
            verb,
            build.content_md5.clone().unwrap_or_default(),
            build.content_type.clone().unwrap_or_default(),
            date,
            oss_header_str,
            oss_resource_str,
        );
        println!("sign_str: {}", sign_str);
        let mut hasher: Hmac<sha1::Sha1> = Hmac::new_from_slice(self.key_secret.as_bytes()).unwrap();
        hasher.update(sign_str.as_bytes());

        general_purpose::STANDARD.encode(&hasher.finalize().into_bytes())
    }
}

impl<'a> OSS {
    pub fn new<S>(key_id: S, key_secret: S, endpoint: S, bucket: S) -> Self
        where
            S: Into<String>,
    {
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

#[inline]
fn get_oss_resource_str(bucket: &str, key: &str, oss_resources: &str) -> String {
    let oss_resources = if oss_resources != "" {
        String::from("?") + oss_resources
    } else {
        String::new()
    };
    if bucket == "" {
        format!("/{}{}", bucket, oss_resources)
    } else {
        format!("/{}{}{}", bucket, key, oss_resources)
    }
}

#[derive(EnumString, Display)]
pub enum RequestType {
    #[strum(serialize = "GET")]
    Get,
    #[strum(serialize = "PUT")]
    Put,
    #[strum(serialize = "POST")]
    Post,
    #[strum(serialize = "DELETE")]
    Delete,
    #[strum(serialize = "HEAD")]
    Head,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign() {
        let oss = OSS::new(
            "my_key_id",
            "my_key_secret",
            "oss-cn-shanghai.aliyuncs.com",
            "my_bucket",
        );
        let build = RequestBuilder::new()
            .expire(60)
            .oss_download_speed_limit(30);
        let download_url = oss.sign_url_with_cdn(
            "http://cdn.ipadump.com",
            "/ipas/cn/-10/ipadump.com_imem内存修改器_1.0.0.ipa",
            build,
        );
        println!("download_url: {}", download_url);
    }
}