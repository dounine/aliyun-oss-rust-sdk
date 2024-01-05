# aliyun-oss-rust-sdk
[![Latest Version](https://img.shields.io/crates/v/aliyun-oss-rust-sdk.svg)](https://crates.io/crates/aliyun-oss-rust-sdk)

阿里云 © Alibaba Cloud Official Oss SDK(标准库)

# 使用指南

1. [文件下载](#文件下载)
2. [签名下载](#签名下载)
3. [签名上传](#签名上传)
4. [获取上传对象的policy](#获取上传对象的policy)
5. [上传本地文件](#上传本地文件)
6. [上传内存文件](#上传内存文件)
7. [文件删除](#文件删除)

## 文件下载
```rust
use aliyun_oss_rust_sdk::object::ObjectAPI;
use aliyun_oss_rust_sdk::oss::OSS;
use aliyun_oss_rust_sdk::request::RequestBuilder;

let oss = OSS::from_env();
let build = RequestBuilder::new();
let bytes = oss.get_object("/hello.txt", &build).unwrap();
println!("file content: {}", String::from_utf8_lossy(bytes.as_slice()));
```

### 签名下载
自定义域名/限速下载/过期时间/自定义content-type
```rust
use aliyun_oss_rust_sdk::oss::{OSS,RequestBuilder};
use aliyun_oss_rust_sdk::url::UrlApi;

let oss = OSS::new(
            "my_key_id",
            "my_key_secret",
            "oss-cn-shanghai.aliyuncs.com",
            "my_bucket",
            );
let build = RequestBuilder::new()
    .with_expire(60)
    //.with_cdn("https://mydomain.com") //使用cdn后，无法限制ip访问
    .oss_download_speed_limit(30);
let download_url = oss.sign_download_url(
    "/ipas/cn/-10/imem内存修改器_1.0.0.ipa",
    &build,
);
println!("download_url: {}", download_url);
```
## 签名上传
. 允许前端简单上传文件，精确控制请用功能4：获取上传对象的policy方式上传

. 自定义域名/限速上传/过期时间/自定义content-type
```rust
use aliyun_oss_rust_sdk::oss::{OSS, RequestBuilder};
use aliyun_oss_rust_sdk::url::UrlApi;

let oss = OSS::from_env();//也可以使用OSS::new()方法传递参数
let build = RequestBuilder::new()
   //.with_cdn("https://mydomain.com")
   .with_content_type("text/plain") //设置上传文件的content-type
   .with_expire(60); //60秒链接过期
let upload_url = oss.sign_upload_url(
    "tmp.txt",
    &build
    );
 println!("upload_url: {}", upload_url);
//使用postman测试上传即可，PS:要注意content-type要和build中的一致
```

## 获取上传对象的policy
用于前端直传可精确控制上传文件的类型、大小、过期时间、上传目录等
```rust
use aliyun_oss_rust_sdk::object::{ObjectAPI, PolicyBuilder};
use aliyun_oss_rust_sdk::oss::OSS;

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
```

## 上传本地文件
```rust
use aliyun_oss_rust_sdk::object::ObjectAPI;
use aliyun_oss_rust_sdk::oss::OSS;
use aliyun_oss_rust_sdk::request::RequestBuilder;

let oss = OSS::from_env();
let builder = RequestBuilder::new()
    .with_expire(60);
let file_path = "./hello.txt";
oss.put_object_from_file("/hello.txt", file_path, &builder).unwrap();
```
## 上传内存文件
```rust
use aliyun_oss_rust_sdk::object::ObjectAPI;
use aliyun_oss_rust_sdk::oss::OSS;
use aliyun_oss_rust_sdk::request::RequestBuilder;

let oss = OSS::from_env();
let builder = RequestBuilder::new()
    .with_expire(60);
let file_path = "./hello.txt";
let buffer = std::fs::read(file_path).unwrap();
oss.pub_object_from_buffer("/hello.txt", buffer.as_slice(), &builder).unwrap();
```
## 文件删除
```rust
use aliyun_oss_rust_sdk::object::ObjectAPI;
use aliyun_oss_rust_sdk::oss::OSS;
use aliyun_oss_rust_sdk::request::RequestBuilder;

let oss = OSS::from_env();
let builder = RequestBuilder::new()
   .with_expire(60);
oss.delete_object("/hello.txt", &builder).unwrap();
```