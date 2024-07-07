#![allow(deprecated)]

use aws_sdk_s3::Client;
use aws_types::region::Region;

#[tokio::main]
async fn main() -> Result<(), aws_sdk_s3::Error> {
    let config = aws_config::from_env().region(Region::new("us-east-2")).load().await;
    let client = Client::new(&config);

    let resp = client.list_buckets().send().await?;
    let buckets = resp.buckets();
    let num_buckets = buckets.len();

    for bucket in buckets {
        println!("{}", bucket.name().unwrap_or_default());
    }

    println!();
    println!("Found {} buckets.", num_buckets);

    Ok(())
}
