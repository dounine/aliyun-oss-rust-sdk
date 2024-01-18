use aliyun_oss_rust_sdk::oss::OSS;
use aliyun_oss_rust_sdk::request::RequestBuilder;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    //set log debug
    let oss = OSS::from_env();
    oss.open_debug();
    let build = RequestBuilder::default();
    let content = oss.get_object("/hello.txt", build).await.unwrap();
    println!("content: {}", String::from_utf8_lossy(content.as_slice()));
}
