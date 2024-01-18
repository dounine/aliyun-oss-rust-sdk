use std::collections::HashMap;
use std::fmt::{Display, Formatter};

pub type Seconds = i64;

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum RequestType {
    Get,
    Put,
    Post,
    Delete,
    Head,
}

impl Display for RequestType{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RequestType::Get => write!(f, "GET"),
            RequestType::Put => write!(f, "PUT"),
            RequestType::Post => write!(f, "POST"),
            RequestType::Delete => write!(f, "DELETE"),
            RequestType::Head => write!(f, "HEAD"),
        }
    }
}

unsafe impl Send for RequestType {}
unsafe impl Sync for RequestType {}

#[derive(Clone, Debug)]
pub struct RequestBuilder {
    pub cdn: Option<String>,
    pub https: bool,
    pub method: RequestType,
    pub expire: Seconds,
    pub headers: HashMap<String, String>,
    pub parameters: HashMap<String, String>,
    pub content_type: Option<String>,
    pub content_md5: Option<String>,
    pub oss_headers: HashMap<String, String>,
}

impl Default for RequestBuilder {
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl Send for RequestBuilder {}

unsafe impl Sync for RequestBuilder {}

impl RequestBuilder {
    pub fn new() -> Self {
        Self {
            cdn: None,
            https: true,
            method: RequestType::Get,
            expire: 60,
            headers: HashMap::new(),
            parameters: HashMap::new(),
            content_type: None,
            content_md5: None,
            oss_headers: HashMap::new(),
        }
    }
    pub fn with_http(mut self) -> Self {
        self.https = false;
        self
    }
    pub fn with_cdn<S: AsRef<str>>(mut self, cdn: S) -> Self {
        self.cdn = Some(cdn.as_ref().to_string());
        self
    }
    pub fn with_content_type<S: AsRef<str>>(mut self, content_type: S) -> Self {
        self.content_type = Some(content_type.as_ref().to_string());
        self
    }
    pub fn with_expire(mut self, expire: Seconds) -> Self {
        self.expire = expire;
        self
    }
    pub fn response_content_disposition<S: AsRef<str>>(mut self, file_name: S) -> Self {
        self.parameters.insert("response-content-disposition".to_string(), format!("attachment;filename={}", file_name.as_ref()));
        self
    }
    pub fn oss_signature_version2(mut self) -> Self {
        self.parameters.insert("x-oss-signature-version".to_string(), "OSS2".to_string());
        self
    }
    pub fn response_content_encoding<S: AsRef<String>>(mut self, encoding: S) -> Self {
        self.parameters.insert("response-content-encoding".to_string(), encoding.as_ref().to_string());
        self
    }
    pub fn oss_download_speed_limit<S: Into<i32>>(mut self, speed: S) -> Self {
        let speed = speed.into();
        assert!(speed >= 30, "speed must be greater than 30kb");
        self.parameters.insert("x-oss-traffic-limit".to_string(), (speed * 1024 * 8).to_string());
        self
    }
    pub fn oss_download_allow_ip<IP, S>(mut self, ip: IP, mask: S) -> Self
        where IP: AsRef<str>, S: Into<u8>
    {
        self.parameters.insert("x-oss-ac-source-ip".to_string(), ip.as_ref().to_string());
        self.parameters.insert("x-oss-ac-subnet-mask".to_string(), mask.into().to_string());
        self
    }
    pub fn oss_ac_forward_allow(mut self) -> Self {
        self.parameters.insert("x-oss-ac-forwarded-for".to_string(), "true".to_string());
        self
    }
    pub fn oss_header_put<S: AsRef<str>>(mut self, key: S, value: S) -> Self {
        self.oss_headers.insert(key.as_ref().to_string(), value.as_ref().to_string());
        self
    }
    pub fn parameters_put<S: AsRef<str>>(mut self, key: S, value: S) -> Self {
        self.parameters.insert(key.as_ref().to_string(), value.as_ref().to_string());
        self
    }
}
