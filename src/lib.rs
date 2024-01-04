//! # 阿里云OSS SDK
//!
//! 本项目是阿里云OSS的Rust SDK，基于HTTP API实现。
//!
//! # 已实现功能列表：
//!
//! 1、签名URL(支持自定义域名)支持限速下载。
//!
//! 2、上传文件
//!
//! 3、文件下载
//!
//! 4、文件删除
//!
//! 5、文件列表
pub mod oss;
pub mod object;
pub mod request;
pub mod error;
pub mod auth;