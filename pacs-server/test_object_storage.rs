use pacs_server::application::services::ObjectStorageServiceFactory;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing Object Storage initialization...");
    
    // Test MinIO connection
    let object_storage = ObjectStorageServiceFactory::create(
        "minio",
        "pacs-masks-dev",
        "us-east-1",
        "http://localhost:9000",
        "minioadmin",
        "minioadmin",
    ).await?;
    
    println!("✅ Object Storage initialized successfully!");
    println!("Provider: MinIO");
    println!("Bucket: pacs-masks-dev");
    println!("Endpoint: http://localhost:9000");
    
    Ok(())
}
