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
    return LOCAL_DEV_MODE;
}