# aliyun-oss-rust-sdk
阿里云rust oss sdk

# 功能列表
1. 签名URL下载(支持自定义域名)支持限速下载。
```rust
use aliyun_oss_rust_sdk::oss::{OSS, OSSAPI, RequestBuilder};
let oss = OSS::from_env();//也可以使用OSS::new()方法传递参数
let mut build = RequestBuilder::new()
    //.with_cdn("https://mydomain.com")
    .expire(60) //60秒链接过期
    .oss_download_speed_limit(30);//限速30kb
let download_url = oss.sign_download_url(
    "/ipas/cn/-10/ipadump.com_imem内存修改器_1.0.0.ipa",
   &mut build
); 
println!("download_url: {}", download_url);
```
2. 文件下载
```rust
use aliyun_oss_rust_sdk::object::ObjectAPI;
use aliyun_oss_rust_sdk::oss::OSS;
use aliyun_oss_rust_sdk::request::RequestBuilder;
let oss = OSS::new(
    "my_key_id",
    "my_key_secret",
    "oss-cn-shanghai.aliyuncs.com",
    "my_bucket",
    );
let build = RequestBuilder::new();
let bytes = oss.get_object("/hello.txt", build).unwrap();
println!("file content: {}", String::from_utf8_lossy(bytes.as_slice()));
```
3. 上传文件
4. 文件删除
5. 文件列表
