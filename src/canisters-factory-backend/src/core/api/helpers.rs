// src/core/api/helpers.rs

use crate::{DEPLOYMENT_STAGE, _DEPLOYMENT_STAGING};

pub fn get_appropriate_url_endpoint() -> String {
    if _DEPLOYMENT_STAGING == DEPLOYMENT_STAGE::Production {
        // In production, use the standard IC URL format
        format!("https://{}.icp0.io", ic_cdk::api::id().to_text())
    } else if _DEPLOYMENT_STAGING == DEPLOYMENT_STAGE::StagingPublicTestnet {
        format!("https://{}.icp-testnet.click", ic_cdk::api::id().to_text())
    } else  {
        // For local development, use the correct local format with canister ID
        let canister_id = ic_cdk::api::id().to_text();
        
        // Use the configured local port if available
        let port = option_env!("IC_LOCAL_PORT").unwrap_or("8000");
        
        // In local development, URLs are typically structured like:
        // http://{canister_id}.localhost:{port}
        format!("http://{}.localhost:{}", canister_id, port)
    }
}

