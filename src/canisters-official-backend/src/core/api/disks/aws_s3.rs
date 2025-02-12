// src/core/api/disks/aws_s3.rs

use std::collections::HashMap;

use base64::{Engine as _, engine::general_purpose};
use ic_cdk::api::management_canister::http_request::{http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod};
use serde::{Serialize, Deserialize};
use time::{Duration, OffsetDateTime};
use crate::core::state::disks::types::AwsBucketAuth;
use num_traits::cast::ToPrimitive;

 pub fn generate_s3_view_url(
        file_key: &str,
        auth: &AwsBucketAuth,
        expires_in: Option<u64>,
        download_filename: Option<&str>  // New parameter
    ) -> String {

    let DEFAULT_EXPIRATION: u64 = 3600; // 1 hour in seconds

    let current_time = ic_cdk::api::time();
    
    // Format dates
    let date = format_date(current_time);         // YYYYMMDD
    let date_time = format_datetime(current_time); // YYYYMMDDTHHMMSSZ
    
    // Construct canonical request components
    let http_method = "GET";
    let canonical_uri = format!("/{}/{}", auth.bucket, file_key);
    
    // Query parameters
    let credential = format!("{}/{}/{}/s3/aws4_request", 
        auth.access_key, date, auth.region);
    
    let expiration = expires_in.unwrap_or(DEFAULT_EXPIRATION).to_string();


    // Create content disposition string if filename provided
    let content_disposition = download_filename.map(|filename| {
        let encoded_filename = url_encode(filename);
        format!("attachment; filename=\"{}\"", encoded_filename)
    });

    // Create query parameters including content-disposition if filename provided
    let mut query_params = vec![
        ("X-Amz-Algorithm", "AWS4-HMAC-SHA256"),
        ("X-Amz-Credential", &credential),
        ("X-Amz-Date", &date_time),
        ("X-Amz-Expires", &expiration),
        ("X-Amz-SignedHeaders", "host")
    ];
     

    // Add content-disposition if present
    if let Some(ref disposition) = content_disposition {
        query_params.push(("response-content-disposition", disposition));
    }
    
    // Sort query parameters
    query_params.sort_by(|a, b| a.0.cmp(b.0));
    
    // Create canonical query string
    let canonical_query_string = query_params
        .iter()
        .map(|(k, v)| format!("{}={}", 
            url_encode(k), 
            url_encode(v)))
        .collect::<Vec<_>>()
        .join("&");
    
    // Create canonical headers
    let host = format!("{}.s3.{}.amazonaws.com", auth.bucket, auth.region);
    let canonical_headers = format!("host:{}\n", host);
    
    // Create canonical request
    let canonical_request = format!("{}\n{}\n{}\n{}\n{}\n{}",
        http_method,
        canonical_uri,
        canonical_query_string,
        canonical_headers,
        "host",  // signed headers
        "UNSIGNED-PAYLOAD"
    );
    
    // Create string to sign
    let string_to_sign = format!(
        "AWS4-HMAC-SHA256\n{}\n{}/{}/s3/aws4_request\n{}",
        date_time,
        date,
        auth.region,
        hex::encode(sha256_hash(canonical_request.as_bytes()))
    );
    
    // Calculate signature
    let signature = sign_policy(&string_to_sign, &auth.secret_key, &date, &auth.region);
    
    // Construct final URL
    format!(
        "https://{}/{}?{}&X-Amz-Signature={}",
        host,
        file_key,
        canonical_query_string,
        signature
    )
}

// URL encode function that follows AWS rules
fn url_encode(s: &str) -> String {
    let mut encoded = String::new();
    for c in s.chars() {
        match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '_' | '-' | '~' | '.' => encoded.push(c),
            _ => {
                encoded.push_str(&format!("%{:02X}", c as u8));
            }
        }
    }
    encoded
}

// SHA256 hash function
fn sha256_hash(data: &[u8]) -> Vec<u8> {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S3UploadResponse {
    pub url: String,
    pub fields: HashMap<String, String>,
}

pub fn generate_s3_upload_url(
    parent_folder_id: &str,
    auth: &AwsBucketAuth,
    max_size: u64,
    expires_in: u64
) -> Result<S3UploadResponse, String> {
    let current_time = ic_cdk::api::time();
    let expiration_time = current_time + (expires_in * 1_000_000_000);

    // Convert timestamps to required formats
    let date = format_date(current_time);         
    let date_time = format_datetime(current_time); 
    let expiration = format_iso8601(expiration_time);

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
                {{"x-amz-credential": "{}/{}/{}/s3/aws4_request"}},
                {{"x-amz-date": "{}"}}
            ]
        }}"#,
        expiration,
        auth.bucket,
        parent_folder_id,
        max_size,
        auth.access_key,
        date,
        auth.region,
        date_time
    );

    let policy_base64 = general_purpose::STANDARD.encode(policy);
    let signature = sign_policy(&policy_base64, &auth.secret_key, &date, &auth.region);

    // Construct fields map
    let mut fields = HashMap::new();
    fields.insert("key".to_string(), format!("{}/{{filename}}", parent_folder_id));
    fields.insert("acl".to_string(), "private".to_string());
    fields.insert("x-amz-algorithm".to_string(), "AWS4-HMAC-SHA256".to_string());
    fields.insert(
        "x-amz-credential".to_string(), 
        format!("{}/{}/{}/s3/aws4_request", auth.access_key, date, auth.region)
    );
    fields.insert("x-amz-date".to_string(), date_time);
    fields.insert("policy".to_string(), policy_base64);
    fields.insert("x-amz-signature".to_string(), signature);

    Ok(S3UploadResponse {
        url: format!("{}/{}", auth.endpoint, auth.bucket),
        fields,
    })
}


fn format_date(time: u64) -> String {
    let nanoseconds = time as i64;
    let seconds = nanoseconds / 1_000_000_000;
    let nanos_remainder = nanoseconds % 1_000_000_000;
    
    let dt = OffsetDateTime::from_unix_timestamp(seconds)
        .unwrap()
        .saturating_add(Duration::nanoseconds(nanos_remainder));
    
    format!("{:04}{:02}{:02}", 
        dt.year(), dt.month() as u8, dt.day())
}

fn format_datetime(time: u64) -> String {
    let nanoseconds = time as i64;
    let seconds = nanoseconds / 1_000_000_000;
    let nanos_remainder = nanoseconds % 1_000_000_000;
    
    let dt = OffsetDateTime::from_unix_timestamp(seconds)
        .unwrap()
        .saturating_add(Duration::nanoseconds(nanos_remainder));
    
    format!("{:04}{:02}{:02}T{:02}{:02}{:02}Z",
        dt.year(), dt.month() as u8, dt.day(),
        dt.hour(), dt.minute(), dt.second())
}

fn format_iso8601(time: u64) -> String {
    let nanoseconds = time as i64;
    let seconds = nanoseconds / 1_000_000_000;
    let nanos_remainder = nanoseconds % 1_000_000_000;
    
    let dt = OffsetDateTime::from_unix_timestamp(seconds)
        .unwrap()
        .saturating_add(Duration::nanoseconds(nanos_remainder));
    
    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        dt.year(), dt.month() as u8, dt.day(),
        dt.hour(), dt.minute(), dt.second())
}

fn sign_policy(policy: &str, secret: &str, date: &str, region: &str) -> String {
    let date_key = hmac_sha256(
        format!("AWS4{}", secret).as_bytes(),
        date.as_bytes()
    );
    let region_key = hmac_sha256(&date_key, region.as_bytes());
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



pub async fn copy_s3_object(
    source_key: &str,
    destination_key: &str, 
    auth: &AwsBucketAuth,
) -> Result<(), String> {
    let host = format!("{}.s3.{}.amazonaws.com", auth.bucket, auth.region);
    let current_time = ic_cdk::api::time();
    let date = format_date(current_time);
    let date_time = format_datetime(current_time);

    // Create the canonical request components for AWS Signature V4
    let credential = format!("{}/{}/{}/s3/aws4_request", 
        auth.access_key, date, auth.region);

    // Create copy source header
    let copy_source = format!("{}/{}", auth.bucket, source_key);
    
    // Construct the request headers
    let headers = vec![
        HttpHeader {
            name: "Host".to_string(),
            value: host.clone(),
        },
        HttpHeader {
            name: "x-amz-date".to_string(),
            value: date_time.clone(),
        },
        HttpHeader {
            name: "x-amz-copy-source".to_string(),
            value: copy_source.clone(),
        },
        HttpHeader {
            name: "x-amz-content-sha256".to_string(),
            value: "UNSIGNED-PAYLOAD".to_string(),
        },
    ];

    // Create canonical request
    let canonical_uri = format!("/{}", destination_key);
    let canonical_headers = format!(
        "host:{}\nx-amz-content-sha256:UNSIGNED-PAYLOAD\nx-amz-copy-source:{}\nx-amz-date:{}\n",
        host, copy_source, date_time
    );
    let signed_headers = "host;x-amz-content-sha256;x-amz-copy-source;x-amz-date";

    let canonical_request = format!("{}\n{}\n\n{}\n{}\n{}",
        "PUT",
        canonical_uri,
        canonical_headers,
        signed_headers,
        "UNSIGNED-PAYLOAD"
    );

    // Create string to sign
    let string_to_sign = format!(
        "AWS4-HMAC-SHA256\n{}\n{}/{}/s3/aws4_request\n{}",
        date_time,
        date,
        auth.region,
        hex::encode(sha256_hash(canonical_request.as_bytes()))
    );

    // Calculate signature
    let signature = sign_policy(&string_to_sign, &auth.secret_key, &date, &auth.region);

    // Create Authorization header
    let authorization = format!(
        "AWS4-HMAC-SHA256 Credential={},SignedHeaders={},Signature={}",
        credential, signed_headers, signature
    );

    // Add Authorization header to headers vec
    let mut final_headers = headers;
    final_headers.push(HttpHeader {
        name: "Authorization".to_string(),
        value: authorization,
    });

    // Create the HTTP request
    let request = CanisterHttpRequestArgument {
        url: format!("https://{}/{}", host, destination_key),
        method: HttpMethod::POST,
        headers: final_headers,
        body: None,
        max_response_bytes: Some(2048),
        transform: None,
    };

    // Make the HTTP request using IC management canister
    let cycles: u128 = 100_000_000_000;
    
    match http_request(request, cycles).await {
        Ok((response,)) => {
            let status_u16: u16 = response.status.0.to_u64() // Convert BigUint to u64 first
                .and_then(|n| {
                    if n <= u16::MAX as u64 {
                        Some(n as u16) // Safely narrow to u16
                    } else {
                        None // Handle overflow
                    }
                })
                .unwrap_or(500); // Fallback to 500 if conversion fails

            if status_u16 < 200 || status_u16 >= 300 {
                Err(format!("S3 copy failed with status {}: {}", 
                    status_u16,
                    String::from_utf8_lossy(&response.body)))
            } else {
                Ok(())
            }
        },
        Err((code, msg)) => Err(format!("HTTP request failed: {:?} - {}", code, msg))
    }
}
