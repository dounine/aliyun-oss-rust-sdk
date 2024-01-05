//! # 阿里云OSS SDK
//!
//! 本项目是阿里云OSS的Rust SDK，基于HTTP API实现。
//!
//! # 功能列表：
//!
//! 1. 签名URL(支持自定义域名)支持限速下载。
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
//!
//! 2. 文件下载
//! ```rust
//! use aliyun_oss_rust_sdk::object::ObjectAPI;
//! use aliyun_oss_rust_sdk::oss::OSS;
//! use aliyun_oss_rust_sdk::request::RequestBuilder;
//! let oss = OSS::new(
//!     "my_key_id",
//!     "my_key_secret",
//!     "oss-cn-shanghai.aliyuncs.com",
//!     "my_bucket",
//!     );
//! let build = RequestBuilder::new();
//! let bytes = oss.get_object("/hello.txt", &build).unwrap();
//! println!("file content: {}", String::from_utf8_lossy(bytes.as_slice()));
//! ```
//! 3. 上传文件
//!
//! //todo待做
//!
//! 4. 文件删除
//!
//! //todo待做
//!
//! 5. 文件列表
//!
//! //todo待做
pub mod oss;
pub mod object;
pub mod request;
pub mod auth;
pub mod url;