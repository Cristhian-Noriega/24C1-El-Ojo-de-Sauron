use aws_config::BehaviorVersion;
use aws_sdk_rekognition::types::{builders::ImageBuilder, S3Object};
use aws_sdk_s3::primitives::ByteStream;
use std::path::Path;

const CONFIDENCE_THRESHOLD: f32 = 50.0;

#[tokio::main]
async fn main() {
    let config: aws_types::SdkConfig = aws_config::defaults(BehaviorVersion::v2024_03_28())
        .region("us-east-2")
        .load()
        .await;
    let s3_client = aws_sdk_s3::Client::new(&config);
    let rekognition_client = aws_sdk_rekognition::Client::new(&config);

    let bucket = "fiuba-sauron";
    let key = "test.jpg";
    let file_path = "images/test.jpg";

    is_incident(&s3_client, &rekognition_client, bucket, key, file_path).await;
}

pub async fn is_incident(
    s3_client: &aws_sdk_s3::Client,
    rekognition_client: &aws_sdk_rekognition::Client,
    bucket: &str,
    key: &str,
    file_path: &str,
) -> bool {
    let _ = upload_file(&s3_client, bucket, key, file_path).await;

    let s3_image = ImageBuilder::default()
        .s3_object({
            S3Object::builder()
                .bucket(bucket.to_string())
                .name(key.to_string())
                .build()
        })
        .build();

    let general_labels_input = aws_sdk_rekognition::types::GeneralLabelsSettings::builder()
        .label_inclusion_filters("Fighting")
        .label_inclusion_filters("Chasing")
        .label_category_inclusion_filters("Weapons and Military")
        .label_category_inclusion_filters("Damage Detection")
        .label_category_inclusion_filters("Public Safety")
        .build();

    let settings_input = aws_sdk_rekognition::types::DetectLabelsSettings::builder()
        .general_labels(general_labels_input)
        .build();

    let request = rekognition_client
        .detect_labels()
        .image(s3_image)
        .settings(settings_input);

    let response = request.send().await;

    if let Ok(response) = response {
        for label in response.labels() {
            if let Some(confidence) = label.confidence() {
                if confidence > CONFIDENCE_THRESHOLD {
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

async fn upload_file(
    client: &aws_sdk_s3::Client,
    bucket: &str,
    key: &str,
    file_path: &str,
) -> Result<(), aws_sdk_s3::Error> {
    let body = ByteStream::from_path(Path::new(file_path)).await;

    client
        .put_object()
        .bucket(bucket)
        .key(key)
        .body(body.unwrap())
        .send()
        .await?;

    Ok(())
}
