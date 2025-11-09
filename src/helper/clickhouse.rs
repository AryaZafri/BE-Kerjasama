use reqwest::{redirect::Policy, ClientBuilder};
use serde_json::Value;
use anyhow::{anyhow, Context};

pub async fn query_ch(query_str: String) -> anyhow::Result<Value> {
    let host = std::env::var("CH_HOST").context("CH_HOST environment variable not set")?;
    let port = std::env::var("CH_PORT").context("CH_PORT environment variable not set")?;
    let username = std::env::var("CH_USERNAME").context("CH_USERNAME environment variable not set")?;
    let password = std::env::var("CH_PASSWORD").context("CH_PASSWORD environment variable not set")?;

    let url = format!(
        "http://{}:{}/?user={}&password={}",
        host, port, username, password
    );

    println!("🔗 ClickHouse URL: {}", url.replace(&password, "***"));
    println!("📤 Query: {}", query_str);

    let client = ClientBuilder::new()
        .cookie_store(true)
        .danger_accept_invalid_certs(true)
        .redirect(Policy::limited(20))
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .context("Failed to build HTTP client")?;

    let res = client
        .post(&url)
        .header("Content-Type", "text/plain")
        .body(query_str.clone())
        .send()
        .await
        .context("Failed to send request to ClickHouse")?;

    let status = res.status();
    let text = res.text().await.context("Failed to read response body")?;

    println!("📥 Response status: {}", status);

    // ✅ Check HTTP status code first
    if !status.is_success() {
        eprintln!("❌ ClickHouse returned error status: {}", status);
        eprintln!("❌ Error response: {}", text);
        
        return Err(anyhow!(
            "ClickHouse query failed with status {}: {}",
            status,
            text
        ));
    }

    // ✅ Try to parse as JSON
    match serde_json::from_str::<Value>(&text) {
        Ok(json) => {
            // ✅ Check if JSON contains error field (ClickHouse error format)
            if let Some(exception) = json.get("exception") {
                let error_msg = exception.as_str().unwrap_or("Unknown error");
                eprintln!("❌ ClickHouse exception: {}", error_msg);
                return Err(anyhow!("ClickHouse error: {}", error_msg));
            }
            
            Ok(json)
        }
        Err(_parse_err) => {
            // ✅ If not JSON, check if it's a plain error message
            if text.contains("Exception") || text.contains("Error") || text.contains("DB::Exception") {
                eprintln!("❌ ClickHouse error (plain text): {}", text);
                return Err(anyhow!("ClickHouse error: {}", text));
            }
            
            // If it's valid non-JSON response (like INSERT success), wrap it
            println!("⚠️  Non-JSON response: {}", text);
            Ok(serde_json::json!({ 
                "raw": text,
                "success": true 
            }))
        }
    }
}

// ✅ Optional: Helper function for INSERT/UPDATE queries that don't return JSON
pub async fn execute_ch(query_str: String) -> anyhow::Result<()> {
    let result = query_ch(query_str).await?;
    
    // Check if execution was successful
    if let Some(success) = result.get("success") {
        if success.as_bool() == Some(true) {
            return Ok(());
        }
    }
    
    // If we got JSON data back, that's also success
    if result.get("data").is_some() {
        return Ok(());
    }
    
    Ok(())
}

// ✅ Optional: Helper function for SELECT queries
pub async fn query_ch_json(query_str: String) -> anyhow::Result<Vec<Value>> {
    let result = query_ch(query_str).await?;
    
    if let Some(data) = result.get("data") {
        if let Some(array) = data.as_array() {
            return Ok(array.clone());
        }
    }
    
    Err(anyhow!("No data array found in response"))
}