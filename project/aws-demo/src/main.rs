#![allow(deprecated)]

use aws_sdk_s3::{Client, primitives::ByteStream};
use aws_types::region::Region;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), aws_sdk_s3::Error> {
    let config = aws_config::from_env().region(Region::new("us-east-2")).load().await;
    let client = Client::new(&config);

    let bucket = "fiuba-sauron";
    let key = "test1.jpg";
    let file_path = "images/test1.jpg";

    let body = ByteStream::from_path(Path::new(file_path)).await;

    let resp = client
        .put_object()
        .bucket(bucket)
        .key(key)
        .body(body.unwrap())
        .send()
        .await?;

    println!("Uploaded file with version ID: {:?}", resp.version_id);

    Ok(())
}