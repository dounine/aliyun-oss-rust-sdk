# aliyun-oss-rust-sdk
> 阿里云 © Alibaba Cloud Official Oss SDK(标准库)

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
## 签名URL(下载)
自定义域名/限速下载/过期时间/自定义content-type
```rust
use aliyun_oss_rust_sdk::oss::{OSS,RequestBuilder};
use aliyun_oss_rust_sdk::url::UrlApi;

let oss = OSS::new(
    "my_key_id",
    "my_key_secret",
    "oss-cn-shanghai.aliyuncs.com",
    "my_bucket",
);//也可以使用OSS::from_env()方法从环境变量中获取参数
let build = RequestBuilder::new()
    //.with_cdn("https://mydomain.com") //使用cdn后，无法限制ip访问
    .expire(60) //60秒链接过期
    .oss_download_speed_limit(30);//限速30kb
let download_url = oss.sign_download_url(
    "/mydir/hello.txt",
   &build
); 
println!("download_url: {}", download_url);
```
## 签名URL(上传)
. 允许前端简单上传文件，精确控制请用功能4：获取上传对象的policy方式上传

. 自定义域名/限速上传/过期时间/自定义content-type
```rust
use aliyun_oss_rust_sdk::oss::{OSS, RequestBuilder};
use aliyun_oss_rust_sdk::url::UrlApi;

let oss = OSS::from_env();//也可以使用OSS::new()方法传递参数
let build = RequestBuilder::new()
   //.with_cdn("https://mydomain.com")
   .with_content_type("text/plain") //设置上传文件的content-type
   .expire(60); //60秒链接过期
let upload_url = oss.sign_upload_url(
    "tmp.txt",
    &build
    );
println!("upload_url: {}", upload_url);
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
## 上传文件
- [ ] 待做
## 文件删除
- [ ] 待做
## 文件列表
- [ ] 待做
