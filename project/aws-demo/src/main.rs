use aws_config::BehaviorVersion;
use aws_sdk_s3::primitives::ByteStream;
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

    let result = is_incident(&s3_client, &rekognition_client, bucket, key, file_path).await;

    if result {
        println!("The file is an incident");
    } else {
        println!("The file is not an incident");
    }
}

async fn is_incident(s3_client: &aws_sdk_s3::Client, rekognition_client: &aws_sdk_rekognition::Client, bucket: &str, key: &str, file_path: &str) -> bool{
    let _ = upload_file(&s3_client, bucket, key, file_path).await;

    let s3_image = ImageBuilder::default().s3_object({
        S3Object::builder()
            .bucket(bucket.to_string())
            .name(key.to_string())
            .build()
    }).build();

    let general_labels_input = aws_sdk_rekognition::types::GeneralLabelsSettings::builder()
        .label_inclusion_filters("Vandalism")
        .label_inclusion_filters("Weapon")
        .label_inclusion_filters("Gun")
        .label_inclusion_filters("Fighting")
        .label_inclusion_filters("Knife")
        .build();

    let settings_input = aws_sdk_rekognition::types::DetectLabelsSettings::builder()
        .general_labels(general_labels_input)
        .build();

    let request = rekognition_client.detect_labels()
        .image(s3_image)
        .settings(settings_input);

    let response = request.send().await;

    if let Ok(response) = response {
        for label in response.labels(){
            if let Some(confidence) = label.confidence(){
                if confidence > 50.0 {
                    return true;
                }
            }
        }
        return false;
    } else {
        println!("Error: {:?}", response.err().unwrap());
        return false;
    }
}

async fn upload_file(client: &aws_sdk_s3::Client, bucket: &str, key: &str, file_path: &str) -> Result<(), aws_sdk_s3::Error> {
    let body = ByteStream::from_path(Path::new(file_path)).await;

    client.put_object()
        .bucket(bucket)
        .key(key)
        .body(body.unwrap())
        .send()
        .await?;

    Ok(())
}