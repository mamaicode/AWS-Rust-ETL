use aws_config::BehaviorVersion;
use aws_lambda_events::event::s3::S3Event;
use lambda_runtime::{service_fn, tracing, Error, LambdaEvent};

use tokio::io::AsyncReadExt;

/// Main function
#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .without_time()
        .init();

    let config = aws_config::load_defaults(BehaviorVersion::latest()).await;
    let s3_client = aws_sdk_s3::Client::new(&config);
    let textract_client = aws_sdk_textract::Client::new(&config);

    let func = service_fn(|request: LambdaEvent<S3Event>| {
        function_handler(&s3_client, &textract_client, request)
    });
    lambda_runtime::run(func).await?;
    Ok(())
}

async fn function_handler(
    s3_client: &aws_sdk_s3::Client,
    textract_client: &aws_sdk_textract::Client,
    request: LambdaEvent<S3Event>,
) -> Result<(), Error> {
    tracing::info!(records = ?request.payload.records.len(), "Received request from SQS");
    if request.payload.records.is_empty() {
        tracing::info!("Empty S3 event received");
        return Ok(());
    }

    let bucket = request.payload.records[0]
        .s3
        .bucket
        .name
        .as_ref()
        .expect("Bucket name to exist");
    let key = request.payload.records[0]
        .s3
        .object
        .key
        .as_ref()
        .expect("Object key to exist");

    tracing::info!("Request is for {} and object {}", bucket, key);

    let s3_get_object_result = s3_client
        .get_object()
        .bucket(bucket)
        .key(key)
        .send()
        .await?;

    // Extract image bytes from S3 object ?
    let mut body = s3_get_object_result.body.into_async_read();
    let mut image_bytes = Vec::new();
    body.read_to_end(&mut image_bytes).await?;

    // textract
    let response = textract_client
        .start_document_text_detection()
        .job_tag("testtag")
        .send()
        .await?;
    tracing::info!("Textract response: {:?}", response);

    Ok(())
}