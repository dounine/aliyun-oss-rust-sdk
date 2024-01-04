use std::collections::HashMap;
use strum_macros::{Display, EnumString};

type Seconds = i64;

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

pub struct RequestBuilder {
    pub method: RequestType,
    pub expire: Seconds,
    pub parameters: HashMap<String, String>,
    pub content_type: Option<String>,
    pub content_md5: Option<String>,
    pub oss_headers: HashMap<String, String>,
}

impl RequestBuilder {
    pub fn new() -> Self {
        RequestBuilder {
            method: RequestType::Get,
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
    pub fn response_content_disposition<S: AsRef<str>>(mut self, file_name: S) -> Self {
        self.parameters.insert("response-content-disposition".to_string(), format!("attachment;filename={}", file_name.as_ref()));
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
        where IP: AsRef<str>, S: Into<i32>
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
}
