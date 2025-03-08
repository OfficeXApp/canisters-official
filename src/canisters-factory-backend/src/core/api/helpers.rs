// src/core/api/helpers.rs

use crate::LOCAL_DEV_MODE;


pub fn get_appropriate_url_endpoint() -> String {
    if is_local_environment() {
        // For local development, use the correct local format with canister ID
        let canister_id = ic_cdk::api::id().to_text();
        
        // Use the configured local port if available
        let port = option_env!("IC_LOCAL_PORT").unwrap_or("8000");
        
        // In local development, URLs are typically structured like:
        // http://{canister_id}.localhost:{port}
        format!("http://{}.localhost:{}", canister_id, port)
    } else {
        // In production, use the standard IC URL format
        format!("https://{}.icp0.io", ic_cdk::api::id().to_text())
    }
}


pub fn is_local_environment() -> bool {
    // Approach 1: Check for an environment variable at build time
    #[cfg(feature = "local")]
    {
        return true;
    }
    
    // Approach 2: Use a compile-time environment variable
    #[cfg(feature = "local_env")]
    {
        return true;
    }
    
    // Approach 3: Define a constant in your code that you can change
    if LOCAL_DEV_MODE {
        return true;
    }
    
    // If none of the above conditions are met, check if we're running in a test environment
    let time_nanos = ic_cdk::api::time();
    // In test/local environment, the time is often set to a specific value or starts from a low number
    if time_nanos < 1_000_000_000_000_000_000 { // If time is before ~2001 (very rough estimate)
        return true;
    }
    
    // If we can't determine for sure, assume production
    false
}