use aws_config::SdkConfig;
use aws_sdk_rekognition::types::{builders::ImageBuilder, S3Object};
use aws_sdk_s3::primitives::ByteStream;
use std::path::Path;

const BUCKET: &str = "fiuba-sauron";

/// Uses AWS Rekognition to determine if an image contains an incident
pub async fn is_incident(
    config: &SdkConfig,
    file_path: &str,
    confidence_threshold: f32,
) -> Option<String> {
    let s3_client = aws_sdk_s3::Client::new(config);
    let rekognition_client = aws_sdk_rekognition::Client::new(config);

    let file_name = match Path::new(file_path).file_name() {
        Some(file_name) => match file_name.to_str() {
            Some(file_name) => file_name,
            None => {
                println!("Error getting file name");
                return None;
            }
        },
        None => {
            println!("Error getting file name");
            return None;
        }
    };

    match upload_file(s3_client, BUCKET, file_path, file_name).await {
        Ok(_) => {}
        Err(e) => {
            println!("Error uploading file: {:?}", e);
            return None;
        }
    }

    let s3_image = ImageBuilder::default()
        .s3_object({
            S3Object::builder()
                .bucket(BUCKET.to_string())
                .name(file_name.to_string())
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

    let mut best_label: Option<String> = None;
    let mut best_confidence: f32 = confidence_threshold;

    if let Ok(response) = response {
        for label in response.labels() {
            match (label.name(), label.confidence()) {
                (Some(name), Some(confidence)) => {
                    if confidence > best_confidence {
                        best_label = Some(name.to_string());
                        best_confidence = confidence;
                    }
                }
                _ => continue,
            }
        }
        best_label
    } else {
        println!("Error: {:?}", response.err());
        None
    }
}

/// Uploads a file to an S3 bucket
async fn upload_file(
    client: aws_sdk_s3::Client,
    bucket: &str,
    file_path: &str,
    file_name: &str,
) -> Result<(), String> {
    let body = ByteStream::from_path(Path::new(file_path)).await;

    let body = match body {
        Ok(body) => body,
        Err(_) => {
            return Err("Error reading file".to_string());
        }
    };

    let result = client
        .put_object()
        .bucket(bucket)
        .key(file_name)
        .body(body)
        .send()
        .await;

    match result {
        Ok(_) => Ok(()),
        Err(_) => Err("Error uploading file".to_string()),
    }
}
