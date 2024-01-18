use base64::engine::general_purpose;
use base64::Engine;
use cfg_if::cfg_if;
use hmac::Hmac;
use reqwest::StatusCode;
use sha1::digest::Mac;
cfg_if! {
   if #[cfg(feature= "debug-print")] {
        use tracing::debug;
    }
}
use crate::entity::{PolicyBuilder, PolicyResp};
use crate::error::OssError;
use crate::oss::{OSSInfo, API, OSS};
use crate::request::{RequestBuilder, RequestType};
use crate::util;
use crate::util::read_file;

impl OSS {
    /// 获取对象
    ///
    /// # 使用例子
    ///
    /// ```rust
    /// use aliyun_oss_rust_sdk::oss::OSS;
    /// use aliyun_oss_rust_sdk::request::RequestBuilder;
    /// let oss = OSS::from_env();
    /// let build = RequestBuilder::new();
    /// let bytes = oss.get_object("/hello.txt", build).await.unwrap();
    /// println!("file content: {}", String::from_utf8_lossy(bytes.as_slice()));
    /// ```
    pub async fn get_object<S: AsRef<str>>(
        &self,
        key: S,
        build: RequestBuilder,
    ) -> Result<Vec<u8>, OssError> {
        let key = self.format_key(key);
        let (url, headers) = self
            .build_request(key.as_str(), build)
            .map_err(|e| OssError::Err(format!("build request error: {}", e)))?;
        cfg_if! {
           if #[cfg(feature= "debug-print")] {
                debug!("oss logget object url: {} headers: {:?}", url,headers);
            }
        }
        let client = reqwest::Client::new();
        let response = client.get(url).headers(headers).send().await?;
        return if response.status().is_success() {
            let result = response.bytes().await?;
            Ok(result.to_vec())
        } else {
            let status = response.status();
            let result = response.text().await?;
            cfg_if! {
               if #[cfg(feature= "debug-print")] {
                    debug!("oss log: get object status: {} error: {}", status,result);
                }
            }
            Err(OssError::Err(format!(
                "get object status: {} error: {}",
                status, result
            )))
        };
    }

    /// 获取上传对象的policy
    /// # 使用例子
    /// ```rust
    /// use aliyun_oss_rust_sdk::entity::PolicyBuilder;
    /// use aliyun_oss_rust_sdk::oss::OSS;
    /// let oss = OSS::from_env();
    /// let policy_builder = PolicyBuilder::new()
    ///             .with_expire(60 * 60)//1个小时过期
    ///             .with_upload_dir("upload/mydir/")//上传目录
    ///             .with_content_type("text/plain")//只允许上传文本.txt
    ///            .with_max_upload_size(100 * 1024 * 1024);//只允许文件上传大小1G以内
    /// let policy = oss.get_upload_object_policy(policy_builder).unwrap();
    /// println!("policy: {:?}", policy);
    /// //使用postman测试上传
    /// //form-data的参数为OSSAccessKeyId、policy、signature、success_action_status、key、file
    /// //key为上传的文件名包含路径、例如：upload/mydir/test.txt
    /// //file为上传的文件，类型跟with_content_type一致
    /// ```
    pub fn get_upload_object_policy(&self, build: PolicyBuilder) -> Result<PolicyResp, OssError> {
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
        "#
        .to_string();
        let success_action_status = 200;
        json_data = json_data.replacen("{time}", &date_str, 1);
        json_data = json_data.replacen("{bucket}", &self.bucket(), 1);
        //limit 1GB bytes
        json_data = json_data.replacen("{size}", &build.max_upload_size.to_string(), 1); //允许上传的最大文件大小
                                                                                         //success status
        json_data = json_data.replacen(
            "{success_action_status}",
            success_action_status.to_string().as_str(),
            1,
        );
        json_data = json_data.replacen("{prefix}", &build.upload_dir, 1); //只允许上传到哪个目录上
                                                                          //text file
        json_data = json_data.replacen("{content_type}", &build.content_type, 1);
        //只允许上传哪个类型文件
        cfg_if! {
           if #[cfg(feature= "debug-print")] {
                debug!("oss log: policy json: {}", json_data);
            }
        }
        let base64_policy = util::base64_encode(json_data.as_bytes());
        let mut hasher: Hmac<sha1::Sha1> = Hmac::new_from_slice(self.key_secret().as_bytes())
            .map_err(|_| OssError::Err("Hmac new from slice error".to_string()))?;
        hasher.update(base64_policy.as_bytes());
        let signature = util::base64_encode(&hasher.finalize().into_bytes());
        Ok(PolicyResp {
            access_id: self.key_id().to_string(),
            host: format!("https://{}.{}", self.bucket(), self.endpoint()),
            policy: base64_policy,
            signature,
            success_action_status,
        })
    }

    /// 上传文件(本地文件)
    /// # 使用例子
    /// ```rust
    /// use aliyun_oss_rust_sdk::oss::OSS;
    /// use aliyun_oss_rust_sdk::request::RequestBuilder;
    /// let oss = OSS::from_env();
    /// let builder = RequestBuilder::new()
    ///     .with_expire(60);
    /// let file_path = "./hello.txt";
    /// oss.put_object_from_file("/hello.txt", file_path, builder).await.unwrap();
    /// ```
    pub async fn put_object_from_file<S: AsRef<str>>(
        &self,
        key: S,
        file_path: S,
        build: RequestBuilder,
    ) -> Result<(), OssError> {
        let buffer = read_file(file_path)?;
        let mut build = build.clone();
        build.method = RequestType::Put;
        let key = self.format_key(key);
        let (url, headers) = self
            .build_request(key.as_str(), build)
            .map_err(|e| OssError::Err(format!("build request error: {}", e)))?;
        cfg_if! {
           if #[cfg(feature= "debug-print")] {
                debug!("oss log: put object from file: {} headers: {:?}", url,headers);
            }
        }
        let client = reqwest::Client::new();
        let response = client.put(url).headers(headers).body(buffer).send().await?;
        return if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status();
            let result = response.text().await?;
            cfg_if! {
               if #[cfg(feature= "debug-print")] {
                    debug!("oss log: get object status: {} error: {}", status,result);
                }
            }
            Err(OssError::Err(format!(
                "get object status: {} error: {}",
                status, result
            )))
        };
    }

    /// 上传文件(内存)
    /// # 使用例子
    /// ```rust
    /// use aliyun_oss_rust_sdk::oss::OSS;
    /// use aliyun_oss_rust_sdk::request::RequestBuilder;
    /// let oss = OSS::from_env();
    /// let builder = RequestBuilder::new()
    ///     .with_expire(60);
    /// let file_path = "./hello.txt";
    /// let buffer = std::fs::read(file_path).unwrap();
    /// oss.pub_object_from_buffer("/hello.txt", buffer.as_slice(), builder).await.unwrap();
    /// ```
    pub async fn pub_object_from_buffer<S: AsRef<str>>(
        &self,
        key: S,
        buffer: &[u8],
        build: RequestBuilder,
    ) -> Result<(), OssError> {
        let mut build = build.clone();
        build.method = RequestType::Put;
        let key = self.format_key(key);
        let (url, headers) = self
            .build_request(key.as_str(), build)
            .map_err(|e| OssError::Err(format!("build request error: {}", e)))?;
        cfg_if! {
           if #[cfg(feature= "debug-print")] {
                debug!("oss log: put object from file: {} headers: {:?}", url,headers);
            }
        }
        let client = reqwest::Client::new();
        let response = client
            .put(url)
            .headers(headers)
            .body(buffer.to_owned())
            .send()
            .await?;
        return if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status();
            let result = response.text().await?;
            cfg_if! {
               if #[cfg(feature= "debug-print")] {
                    debug!("oss log: get object status: {} error: {}", status,result);
                }
            }
            Err(OssError::Err(format!(
                "get object status: {} error: {}",
                status, result
            )))
        };
    }

    /// 删除文件
    /// # 使用例子
    /// ```rust
    /// use aliyun_oss_rust_sdk::oss::OSS;
    /// use aliyun_oss_rust_sdk::request::RequestBuilder;
    /// let oss = OSS::from_env();
    /// let builder = RequestBuilder::new()
    ///    .with_expire(60);
    /// oss.delete_object("/hello.txt", builder).await.unwrap();
    /// ```
    pub async fn delete_object<S: AsRef<str>>(
        &self,
        key: S,
        build: RequestBuilder,
    ) -> Result<(), OssError> {
        let mut build = build.clone();
        build.method = RequestType::Delete;
        let key = self.format_key(key);
        let (url, headers) = self
            .build_request(key.as_str(), build)
            .map_err(|e| OssError::Err(format!("build request error: {}", e)))?;
        cfg_if! {
           if #[cfg(feature= "debug-print")] {
                debug!("oss log: put object from file: {} headers: {:?}", url,headers);
            }
        }
        let client = reqwest::Client::new();
        let response = client.delete(url).headers(headers).send().await?;
        return if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status();
            let result = response.text().await?;
            cfg_if! {
               if #[cfg(feature= "debug-print")] {
                    debug!("oss log: get object status: {} error: {}", status,result);
                }
            }
            Err(OssError::Err(format!(
                "get object status: {} error: {}",
                status, result
            )))
        };
    }
}
