use std::collections::HashMap;
use reqwest::header::DATE;
use tracing::debug;
use crate::auth::AuthAPI;
use crate::oss::{API, OSS, OSSInfo};
use crate::request::{RequestBuilder, RequestType};

pub trait UrlApi: OSSInfo + API {
    /// 获取签名下载URL
    ///
    /// # 使用例子
    ///
    /// ```
    /// use aliyun_oss_rust_sdk::oss::{OSS, RequestBuilder};
    /// use aliyun_oss_rust_sdk::url::UrlApi;
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
    fn sign_download_url<S: AsRef<str>>(&self, key: S, build: &RequestBuilder) -> String;

    /// 获取签名上传URL
    ///
    /// # 使用例子
    ///
    /// ```
    /// use aliyun_oss_rust_sdk::oss::{OSS, RequestBuilder};
    /// use aliyun_oss_rust_sdk::url::UrlApi;
    /// let oss = OSS::from_env();//也可以使用OSS::new()方法传递参数
    /// let build = RequestBuilder::new()
    ///    //.with_cdn("https://mydomain.com")
    ///    .with_content_type("text/plain") //设置上传文件的content-type
    ///    .expire(60); //60秒链接过期
    /// let upload_url = oss.sign_upload_url(
    ///     "tmp.txt",
    ///     &build
    ///     );
    ///  println!("upload_url: {}", upload_url);
    /// //使用postman测试上传即可，PS:要注意content-type要和build中的一致
    /// ```
    fn sign_upload_url<S: AsRef<str>>(&self, key: S, build: &RequestBuilder) -> String;
    fn sign_url<S: AsRef<str>>(&self, key: S, build: &RequestBuilder) -> String;
}

impl UrlApi for OSS {
    fn sign_download_url<S: AsRef<str>>(&self, key: S, build: &RequestBuilder) -> String {
        let sign = self.sign_url(key.as_ref(), build);
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

    fn sign_upload_url<S: AsRef<str>>(&self, key: S, build: &RequestBuilder) -> String {
        let mut build = build.clone();
        build.method = RequestType::Put;
        let sign = self.sign_url(key.as_ref(), &build);
        if let Some(cdn) = &build.cdn {
            let download_url = format!("{}{}", cdn, sign);
            debug!("upload_url: {}", download_url);
            download_url
        } else {
            let download_url = format!("{}.{}{}", self.bucket(), self.endpoint(), sign);
            debug!("upload_url: {}", download_url);
            download_url
        }
    }

    fn sign_url<S: AsRef<str>>(&self, key: S, build: &RequestBuilder) -> String {
        let mut build = build.clone();
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
        query_parameters.insert("OSSAccessKeyId".to_string(), self.key_id().to_string());
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

#[cfg(test)]
mod tests {
    use crate::oss::OSS;
    use crate::request::RequestBuilder;
    use crate::url::UrlApi;

    #[inline]
    fn init_log() {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .with_line_number(true)
            .init();
    }

    #[test]
    fn sign_download_url_test() {
        init_log();
        let oss = OSS::from_env();
        let build = RequestBuilder::new()
            .with_cdn("http://cdn.ipadump.com")
            .expire(60)
            .oss_download_speed_limit(30);
        oss.sign_download_url(
            "/ipas/cn/-10/ipadump.com_imem内存修改器_1.0.0.ipa",
            &build,
        );
    }

    #[test]
    fn sign_upload_url_test() {
        init_log();
        let oss = OSS::from_env();
        let build = RequestBuilder::new()
            .with_cdn("http://cdn.ipadump.com")
            .with_content_type("text/plain")
            .expire(600);
        oss.sign_upload_url(
            "tmp.txt",
            &build,
        );
    }
}