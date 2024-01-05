//! # 阿里云OSS SDK
//!
//! 本项目是阿里云OSS的Rust SDK，基于HTTP API实现。
//!
//! # 功能列表：
//!
//! 1. 签名下载URL(支持自定义域名)支持限速下载。
//! ```rust
//! use aliyun_oss_rust_sdk::oss::{OSS,RequestBuilder};
//! use aliyun_oss_rust_sdk::url::UrlApi;
//!
//! let oss = OSS::new(
//!             "my_key_id",
//!             "my_key_secret",
//!             "oss-cn-shanghai.aliyuncs.com",
//!             "my_bucket",
//!             );
//! let build = RequestBuilder::new()
//!     .expire(60)
//!     //.with_cdn("https://mydomain.com")
//!     .oss_download_speed_limit(30);
//! let download_url = oss.sign_download_url(
//!     "/ipas/cn/-10/imem内存修改器_1.0.0.ipa",
//!     &build,
//! );
//! println!("download_url: {}", download_url);
//! ```
//! 2. 签名上传URL(支持自定义域名)支持限速上传。
//! ```rust
//! use aliyun_oss_rust_sdk::oss::{OSS, RequestBuilder};
//! use aliyun_oss_rust_sdk::url::UrlApi;
//!
//! let oss = OSS::from_env();//也可以使用OSS::new()方法传递参数
//! let build = RequestBuilder::new()
//!    //.with_cdn("https://mydomain.com")
//!    .with_content_type("text/plain") //设置上传文件的content-type
//!    .expire(60); //60秒链接过期
//! let upload_url = oss.sign_upload_url(
//!     "tmp.txt",
//!     &build
//!     );
//!  println!("upload_url: {}", upload_url);
//! //使用postman测试上传即可，PS:要注意content-type要和build中的一致
//! ```
//! 3. 文件下载
//! ```rust
//! use aliyun_oss_rust_sdk::object::ObjectAPI;
//! use aliyun_oss_rust_sdk::oss::OSS;
//! use aliyun_oss_rust_sdk::request::RequestBuilder;
//!
//! let oss = OSS::from_env();
//! let build = RequestBuilder::new();
//! let bytes = oss.get_object("/hello.txt", &build).unwrap();
//! println!("file content: {}", String::from_utf8_lossy(bytes.as_slice()));
//! ```
//! 4. 上传文件
//!
//! //todo待做
//!
//! 5. 文件删除
//!
//! //todo待做
//!
//! 6. 文件列表
//!
//! //todo待做
pub mod oss;
pub mod object;
pub mod request;
pub mod auth;
pub mod url;