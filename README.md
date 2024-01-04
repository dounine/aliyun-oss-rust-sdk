# aliyun-oss-rust-sdk
阿里云rust oss sdk

# 功能列表
1. 签名URL下载(支持自定义域名)支持限速下载。
```rust
let oss = OSS::new(
            "my_key_id",
            "my_key_secret",
            "oss-cn-shanghai.aliyuncs.com",
            "my_bucket",
        );
let build = RequestBuilder::new()
    .expire(60)
    .oss_download_speed_limit(30);
let download_url = oss.sign_url_with_cdn(
    "https://cdn.mydomain.com",
    "/ipas/cn/-10/imem内存修改器_1.0.0.ipa",
    build,
);
println!("download_url: {}", download_url);
```
2. 上传文件
3. 文件下载
4. 文件删除
5. 文件列表
