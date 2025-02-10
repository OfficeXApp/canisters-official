use base64::{Engine as _, engine::general_purpose};
use serde::{Serialize, Deserialize};

use crate::core::state::disks::types::AwsBucketAuth;


pub fn generate_s3_upload_url(
    parent_folder_id: &str,
    auth: &AwsBucketAuth,
    max_size: u64,  // in bytes
    expires_in: u64 // in seconds
) -> String {
    let current_time = ic_cdk::api::time();
    let expiration_time = current_time + (expires_in * 1_000_000_000); // Convert seconds to nanos

    // Convert timestamps to required formats
    let date = format_date(current_time);         // YYYYMMDD
    let date_time = format_datetime(current_time); // YYYYMMDDTHHMMSSZ
    let expiration = format_iso8601(expiration_time);  // YYYY-MM-DDTHH:MM:SSZ

    // Policy document restricting uploads to folder
    let policy = format!(
        r#"{{
            "expiration": "{}",
            "conditions": [
                {{"bucket": "{}"}},
                ["starts-with", "$key", "{}/"],
                {{"acl": "private"}},
                ["content-length-range", 0, {}],
                {{"x-amz-algorithm": "AWS4-HMAC-SHA256"}},
                {{"x-amz-credential": "{}/{}/us-east-1/s3/aws4_request"}},
                {{"x-amz-date": "{}"}}
            ]
        }}"#,
        expiration,
        auth.bucket,
        parent_folder_id,
        max_size,
        auth.access_key,
        date,
        date_time
    );

    let policy_base64 = general_purpose::STANDARD.encode(policy);
    let signature = sign_policy(&policy_base64, &auth.secret_key, &date);

    format!(
        r#"{{
            "url": "{}/{}",
            "fields": {{
                "key": "{}/{{filename}}",
                "acl": "private",
                "x-amz-algorithm": "AWS4-HMAC-SHA256",
                "x-amz-credential": "{}/{}/us-east-1/s3/aws4_request",
                "x-amz-date": "{}",
                "policy": "{}",
                "x-amz-signature": "{}"
            }}
        }}"#,
        auth.endpoint,
        auth.bucket,
        parent_folder_id,
        auth.access_key,
        date,
        date_time,
        policy_base64,
        signature
    )
}

// Time formatting helpers using IC timestamp (nanoseconds)
fn format_date(time: u64) -> String {
    let seconds = (time / 1_000_000_000) as i64;
    let days = seconds / 86400;
    let year = 1970 + (days / 365);
    let month = ((days % 365) / 30) + 1;
    let day = (days % 365) % 30 + 1;
    
    format!("{:04}{:02}{:02}", year, month, day)
}

fn format_datetime(time: u64) -> String {
    let seconds = (time / 1_000_000_000) as i64;
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;
    
    let year = 1970 + (days / 365);
    let month = ((days % 365) / 30) + 1;
    let day = (days % 365) % 30 + 1;
    
    format!("{:04}{:02}{:02}T{:02}{:02}{:02}Z", 
        year, month, day, hours, minutes, secs)
}

fn format_iso8601(time: u64) -> String {
    let seconds = (time / 1_000_000_000) as i64;
    let days = seconds / 86400;
    let hours = (seconds / 3600) % 24;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;
    
    let year = 1970 + (days / 365);
    let month = ((days % 365) / 30) + 1;
    let day = (days % 365) % 30 + 1;
    
    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", 
        year, month, day, hours, minutes, secs)
}

fn sign_policy(policy: &str, secret: &str, date: &str) -> String {
    let date_key = hmac_sha256(
        format!("AWS4{}", secret).as_bytes(),
        date.as_bytes()
    );
    let region_key = hmac_sha256(&date_key, b"us-east-1");
    let service_key = hmac_sha256(&region_key, b"s3");
    let signing_key = hmac_sha256(&service_key, b"aws4_request");
    
    hex::encode(hmac_sha256(&signing_key, policy.as_bytes()))
}

fn hmac_sha256(key: &[u8], data: &[u8]) -> Vec<u8> {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    
    type HmacSha256 = Hmac<Sha256>;
    let mut mac = HmacSha256::new_from_slice(key)
        .expect("HMAC can take key of any size");
    mac.update(data);
    mac.finalize().into_bytes().to_vec()
}