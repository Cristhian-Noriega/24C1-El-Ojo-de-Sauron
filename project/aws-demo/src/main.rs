use aws_config::BehaviorVersion;
use aws_sdk_s3::{Client, primitives::ByteStream};
use aws_sdk_rekognition::types::{S3Object, builders::ImageBuilder};
use std::path::Path;

#[tokio::main]
async fn main() {
    let config = aws_config::defaults(BehaviorVersion::v2024_03_28()).region("us-east-2").load().await;
    let s3_client = aws_sdk_s3::Client::new(&config);
    let rekognition_client = aws_sdk_rekognition::Client::new(&config);
    

    let bucket = "fiuba-sauron";
    let key = "test.jpg";
    let file_path = "images/test.jpg";

    let _ = upload_file(&s3_client, bucket, key, file_path).await;
    let _ = scan_file(&rekognition_client, bucket, key).await;
}

async fn upload_file(client: &Client, bucket: &str, key: &str, file_path: &str) -> Result<(), aws_sdk_s3::Error> {
    let body = ByteStream::from_path(Path::new(file_path)).await;

    client.put_object()
        .bucket(bucket)
        .key(key)
        .body(body.unwrap())
        .send()
        .await?;

    println!("The file was uploaded successfully");

    Ok(())
}

async fn scan_file(client: &aws_sdk_rekognition::Client, bucket: &str, key: &str) -> Result<(), aws_sdk_rekognition::Error> {
    let s3_image = ImageBuilder::default().s3_object({
        S3Object::builder()
            .bucket(bucket.to_string())
            .name(key.to_string())
            .build()
    }).build();

    let request = client.detect_labels()
        .image(s3_image)
        .max_labels(10)
        .min_confidence(10.0);

    let response = request.send().await?;

    println!("Detected labels:");
    for label in response.labels.unwrap() {
        println!("  {} (confidence: {})", label.name.unwrap(), label.confidence.unwrap());
    }

    Ok(())
}