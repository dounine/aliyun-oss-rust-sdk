use anyhow::{anyhow, Result};
use base64::Engine;
use base64::engine::general_purpose;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use tracing::debug;
use crate::oss::{API, OSS, OSSInfo};
use crate::request::{RequestBuilder, RequestType, Seconds};
use crate::util::read_file;

pub trait ObjectAPI {
    /// 获取对象
    ///
    /// # 使用例子
    ///
    /// ```rust
    /// use aliyun_oss_rust_sdk::object::ObjectAPI;
    /// use aliyun_oss_rust_sdk::oss::OSS;
    /// use aliyun_oss_rust_sdk::request::RequestBuilder;
    /// let oss = OSS::from_env();
    /// let build = RequestBuilder::new();
    /// let bytes = oss.get_object("/hello.txt", build).unwrap();
    /// println!("file content: {}", String::from_utf8_lossy(bytes.as_slice()));
    /// ```
    fn get_object<S: AsRef<str>>(
        &self,
        key: S,
        build: &RequestBuilder,
    ) -> Result<Vec<u8>>;

    /// 获取上传对象的policy
    /// # 使用例子
    /// ```rust
    /// use aliyun_oss_rust_sdk::object::{ObjectAPI, PolicyBuilder};
    /// use aliyun_oss_rust_sdk::oss::OSS;
    /// let oss = OSS::from_env();
    /// let policy_builder = PolicyBuilder::new()
    ///             .with_expire(60 * 60)//1个小时过期
    ///             .with_upload_dir("upload/mydir/")//上传目录
    ///             .with_content_type("text/plain")//只允许上传文本.txt
    ///            .with_max_upload_size(100 * 1024 * 1024);//只允许文件上传大小1G以内
    /// let policy = oss.get_upload_object_policy(&policy_builder).unwrap();
    /// println!("policy: {:?}", policy);
    /// //使用postman测试上传
    /// //form-data的参数为OSSAccessKeyId、policy、signature、success_action_status、key、file
    /// //key为上传的文件名包含路径、例如：upload/mydir/test.txt
    /// //file为上传的文件，类型跟with_content_type一致
    /// ```
    fn get_upload_object_policy(&self, build: &PolicyBuilder) -> Result<PolicyResp>;


    /// 上传文件(本地文件)
    /// # 使用例子
    /// ```rust
    /// use aliyun_oss_rust_sdk::object::ObjectAPI;
    /// use aliyun_oss_rust_sdk::oss::OSS;
    /// use aliyun_oss_rust_sdk::request::RequestBuilder;
    /// let oss = OSS::from_env();
    /// let builder = RequestBuilder::new()
    ///     .with_expire(60);
    /// let file_path = "./hello.txt";
    /// oss.put_object_from_file("/hello.txt", file_path, &builder).unwrap();
    /// ```
    fn put_object_from_file<S: AsRef<str>>(
        &self,
        key: S,
        file_path: S,
        build: &RequestBuilder,
    ) -> Result<()>;

    /// 上传文件(内存)
    /// # 使用例子
    /// ```rust
    /// use aliyun_oss_rust_sdk::object::ObjectAPI;
    /// use aliyun_oss_rust_sdk::oss::OSS;
    /// use aliyun_oss_rust_sdk::request::RequestBuilder;
    /// let oss = OSS::from_env();
    /// let builder = RequestBuilder::new()
    ///     .with_expire(60);
    /// let file_path = "./hello.txt";
    /// let buffer = std::fs::read(file_path).unwrap();
    /// oss.pub_object_from_buffer("/hello.txt", buffer.as_slice(), &builder).unwrap();
    /// ```
    fn pub_object_from_buffer<S: AsRef<str>>(
        &self,
        key: S,
        buffer: &[u8],
        build: &RequestBuilder,
    ) -> Result<()>;

    /// 删除文件
    /// # 使用例子
    /// ```rust
    /// use aliyun_oss_rust_sdk::object::ObjectAPI;
    /// use aliyun_oss_rust_sdk::oss::OSS;
    /// use aliyun_oss_rust_sdk::request::RequestBuilder;
    /// let oss = OSS::from_env();
    /// let builder = RequestBuilder::new()
    ///    .with_expire(60);
    /// oss.delete_object("/hello.txt", &builder).unwrap();
    /// ```
    fn delete_object<S: AsRef<str>>(
        &self,
        key: S,
        build: &RequestBuilder,
    ) -> Result<()>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyResp {
    access_id: String,
    host: String,
    policy: String,
    signature: String,
    success_action_status: u8,
}

unsafe impl Send for PolicyResp {}
unsafe impl Sync for PolicyResp {}

/// Policy构建器
/// # 使用例子
/// ```rust
///
#[derive(Debug, Clone)]
pub struct PolicyBuilder {
    expire: Seconds,
    upload_dir: String,
    content_type: String,
    max_upload_size: i64,
}

unsafe impl Send for PolicyBuilder {}
unsafe impl Sync for PolicyBuilder {}

impl Default for PolicyBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl PolicyBuilder {
    pub fn new() -> Self {
        Self {
            expire: 60,//60秒
            upload_dir: "".to_string(),
            content_type: "text/plain".to_string(),//文本.txt
            max_upload_size: 100 * 1024 * 1024,//100m
        }
    }
    pub fn with_expire(mut self, expire: Seconds) -> Self {
        self.expire = expire;
        self
    }
    pub fn with_upload_dir<S: AsRef<str>>(mut self, upload_dir: S) -> Self {
        self.upload_dir = upload_dir.as_ref().to_string();
        self
    }
    pub fn with_content_type<S: AsRef<str>>(mut self, content_type: S) -> Self {
        self.content_type = content_type.as_ref().to_string();
        self
    }
    pub fn with_max_upload_size(mut self, max_upload_size: i64) -> Self {
        self.max_upload_size = max_upload_size;
        self
    }
}

impl ObjectAPI for OSS {
    fn get_object<S: AsRef<str>>(&self, key: S, build: &RequestBuilder) -> Result<Vec<u8>> {
        let key = self.format_key(key);
        let (url, headers) = self.build_request(key.as_str(), build)?;
        debug!("get object url: {} headers: {:?}", url,headers);
        let client = reqwest::blocking::Client::new();
        let response = client.get(url)
            .headers(headers).send()?;
        return if response.status().is_success() {
            let result = response.bytes()?;
            Ok(result.to_vec())
        } else {
            let status = response.status();
            let result = response.text()?;
            debug!("get object status: {} error: {}", status,result);
            Err(anyhow!(format!("get object status: {} error: {}", status,result)))
        };
    }

    fn get_upload_object_policy(&self, build: &PolicyBuilder) -> Result<PolicyResp> {
        let date = chrono::Local::now().naive_local() + chrono::Duration::seconds(build.expire);
        let date_str = date.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();
        let mut json_data = r#"
        {
            "expiration": "{time}",
            "conditions": [
                {"bucket": "{bucket}" },
                ["content-length-range", 1, {size}],
                ["eq", "$success_action_status", "{success_action_status}"],
                ["starts-with", "$key", "{prefix}"],
                ["in", "$content-type", ["{content_type}"]]
            ]
        }
        "#.to_string();
        let success_action_status = 200;
        json_data = json_data.replacen("{time}", &date_str, 1);
        json_data = json_data.replacen("{bucket}", &self.bucket(), 1);
        //limit 1GB bytes
        json_data = json_data.replacen("{size}", &build.max_upload_size.to_string(), 1);//允许上传的最大文件大小
        //success status
        json_data = json_data.replacen("{success_action_status}", success_action_status.to_string().as_str(), 1);
        json_data = json_data.replacen("{prefix}", &build.upload_dir, 1);//只允许上传到哪个目录上
        //text file
        json_data = json_data.replacen("{content_type}", &build.content_type, 1);
        //只允许上传哪个类型文件
        debug!("policy json: {}", json_data);
        let base64_policy = general_purpose::STANDARD.encode(json_data.as_bytes());
        let mut hasher: Hmac<sha1::Sha1> = Hmac::new_from_slice(self.key_secret().as_bytes()).unwrap();
        hasher.update(base64_policy.as_bytes());
        let signature = general_purpose::STANDARD.encode(&hasher.finalize().into_bytes());
        Ok(PolicyResp {
            access_id: self.key_id().to_string(),
            host: format!("https://{}.{}", self.bucket(), self.endpoint()),
            policy: base64_policy,
            signature,
            success_action_status,
        })
    }

    fn put_object_from_file<S: AsRef<str>>(&self, key: S, file_path: S, build: &RequestBuilder) -> Result<()> {
        let buffer = read_file(file_path)?;
        let mut build = build.clone();
        build.method = RequestType::Put;
        let key = self.format_key(key);
        let (url, headers) = self.build_request(key.as_str(), &build)?;
        debug!("put object from file: {} headers: {:?}", url,headers);
        let client = reqwest::blocking::Client::new();
        let response = client.put(url)
            .headers(headers)
            .body(buffer)
            .send()?;
        return if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status();
            let result = response.text()?;
            debug!("get object status: {} error: {}", status,result);
            Err(anyhow!(format!("get object status: {} error: {}", status,result)))
        };
    }

    fn pub_object_from_buffer<S: AsRef<str>>(&self, key: S, buffer: &[u8], build: &RequestBuilder) -> Result<()> {
        let mut build = build.clone();
        build.method = RequestType::Put;
        let key = self.format_key(key);
        let (url, headers) = self.build_request(key.as_str(), &build)?;
        debug!("put object from file: {} headers: {:?}", url,headers);
        let client = reqwest::blocking::Client::new();
        let response = client.put(url)
            .headers(headers)
            .body(buffer.to_owned())
            .send()?;
        return if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status();
            let result = response.text()?;
            debug!("get object status: {} error: {}", status,result);
            Err(anyhow!(format!("get object status: {} error: {}", status,result)))
        };
    }

    fn delete_object<S: AsRef<str>>(&self, key: S, build: &RequestBuilder) -> Result<()> {
        let mut build = build.clone();
        build.method = RequestType::Delete;
        let key = self.format_key(key);
        let (url, headers) = self.build_request(key.as_str(), &build)?;
        debug!("put object from file: {} headers: {:?}", url,headers);
        let client = reqwest::blocking::Client::new();
        let response = client.delete(url)
            .headers(headers)
            .send()?;
        return if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status();
            let result = response.text()?;
            debug!("get object status: {} error: {}", status,result);
            Err(anyhow!(format!("get object status: {} error: {}", status,result)))
        };
    }
}

#[cfg(test)]
mod tests {
    use crate::object::{ObjectAPI, PolicyBuilder};
    use crate::oss::OSS;
    use crate::request::RequestBuilder;

    #[inline]
    fn init_log() {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .with_line_number(true)
            .init();
    }

    #[test]
    fn test_get_upload_object_policy() {
        init_log();
        let oss = OSS::from_env();
        let policy_builder = PolicyBuilder::new()
            .with_expire(60 * 60)//1个小时过期
            .with_upload_dir("upload/mydir/")//上传目录
            .with_content_type("text/plain")//只允许上传文本.txt
            .with_max_upload_size(100 * 1024 * 1024);//只允许文件上传大小1G以内
        let policy = oss.get_upload_object_policy(&policy_builder).unwrap();
        println!("policy: {:?}", policy);
        //使用postman测试上传
        //form-data的参数为OSSAccessKeyId、policy、signature、success_action_status、key、file
        //key为上传的文件名包含路径、例如：upload/mydir/test.txt
        //file为上传的文件，类型跟with_content_type一致
    }

    #[test]
    fn test_put_object_from_file() {
        init_log();
        let oss = OSS::from_env();
        let builder = RequestBuilder::new()
            .with_expire(60);
        let file_path = "./Cargo.toml";
        oss.put_object_from_file("/cargo.toml", file_path, &builder).unwrap();
    }

    #[test]
    fn test_put_object_from_buffer() {
        init_log();
        let oss = OSS::from_env();
        let builder = RequestBuilder::new()
            .with_expire(60);
        let file_path = "./Cargo.toml";
        let buffer = std::fs::read(file_path).unwrap();
        oss.pub_object_from_buffer("/cargo.toml", buffer.as_slice(), &builder).unwrap();
    }

    #[test]
    fn test_delete_object() {
        init_log();
        let oss = OSS::from_env();
        let builder = RequestBuilder::new()
            .with_expire(60);
        oss.delete_object("/cargo.toml", &builder).unwrap();
    }

    #[test]
    fn test_get_object() {
        init_log();
        let oss = OSS::from_env();
        let build = RequestBuilder::new()
            .with_cdn("http://cdn.ipadump.com");
        let bytes = oss.get_object("/hello.txt", &build).unwrap();
        println!("file content: {}", String::from_utf8_lossy(bytes.as_slice()));
    }
}

