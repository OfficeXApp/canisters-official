// src/core/api/disks/aws_s3.rs

use std::collections::HashMap;

use base64::{Engine as _, engine::general_purpose};
use ic_cdk::api::management_canister::http_request::{http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod};
use serde::{Serialize, Deserialize};
use time::{Duration, OffsetDateTime};
use crate::{core::state::{disks::types::{AwsBucketAuth, DiskID}, drives::state::state::DRIVE_ID}, debug_log, rest::directory::types::DiskUploadResponse};
use num_traits::cast::ToPrimitive;

pub fn generate_s3_view_url(
    file_id: &str,
    file_extension: &str,  // Added parameter for file extension
    auth: &AwsBucketAuth,
    expires_in: Option<u64>,
    download_filename: Option<&str>,
    disk_id: DiskID
) -> String {
    let DEFAULT_EXPIRATION: u64 = 60 * 60 * 24; // 24 hours
    let current_time = ic_cdk::api::time();
    
    // Format dates
    let date = format_date(current_time);         // YYYYMMDD
    let date_time = format_datetime(current_time); // YYYYMMDDTHHMMSSZ
    
    // Query parameters
    let credential = format!("{}/{}/{}/s3/aws4_request", 
        auth.access_key, date, auth.region);
    
    let expiration = expires_in.unwrap_or(DEFAULT_EXPIRATION).to_string();

    // Host construction
    let host = format!("{}.s3.{}.amazonaws.com", auth.bucket, auth.region);

    // Construct the S3 key using the same format as upload
    let drive_id = DRIVE_ID.with(|id| id.clone());
    let s3_key = format!("{}/{}/{}/{}.{}", drive_id, disk_id, file_id, file_id, file_extension);

    // Create content disposition string if filename provided
    let content_disposition = Some("inline".to_string());
    // let content_disposition = download_filename.map(|filename| {
    //     let encoded_filename = url_encode(filename);
    //     format!("attachment; filename=\"{}\"", encoded_filename)
    // });

    // Create sorted query parameters
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
        .map(|(k, v)| format!("{}={}", url_encode(k), url_encode(v)))
        .collect::<Vec<_>>()
        .join("&");

    // Create canonical headers
    let canonical_headers = format!("host:{}\n", host);

    // Create canonical request
    let canonical_request = format!("GET\n/{}\n{}\n{}\nhost\nUNSIGNED-PAYLOAD",
        s3_key,
        canonical_query_string,
        canonical_headers
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
    let signing_key = derive_signing_key(&auth.secret_key, &date, &auth.region, "s3");
    let signature = hex::encode(hmac_sha256(&signing_key, string_to_sign.as_bytes()));

    // Construct final URL
    format!(
        "https://{}/{}?{}&X-Amz-Signature={}",
        host,
        s3_key,
        canonical_query_string,
        signature
    )
}

fn derive_signing_key(secret: &str, date: &str, region: &str, service: &str) -> Vec<u8> {
    let k_date = hmac_sha256(format!("AWS4{}", secret).as_bytes(), date.as_bytes());
    let k_region = hmac_sha256(&k_date, region.as_bytes());
    let k_service = hmac_sha256(&k_region, service.as_bytes());
    hmac_sha256(&k_service, b"aws4_request")
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


pub fn generate_s3_upload_url(
    file_id: &str,
    file_extension: &str,
    auth: &AwsBucketAuth,
    max_size: u64,
    expires_in: u64,
    disk_id: DiskID
) -> Result<DiskUploadResponse, String> {
    let current_time = ic_cdk::api::time();
    let expiration_time = current_time + (expires_in * 1_000_000_000);

    // Convert timestamps to required formats
    let date = format_date(current_time);         
    let date_time = format_datetime(current_time); 
    let expiration = format_iso8601(expiration_time);

    // Create the target key using fileId for both directory and filename
    let drive_id = DRIVE_ID.with(|id| id.clone());
    let target_key = format!("{}/{}/{}/{}.{}", drive_id, disk_id, file_id, file_id, file_extension);

    // Policy document with exact key instead of starts-with
    let policy = format!(
        r#"{{
            "expiration": "{}",
            "conditions": [
                {{"bucket": "{}"}},
                {{"key": "{}"}},
                {{"acl": "private"}},
                ["content-length-range", 0, {}],
                {{"x-amz-algorithm": "AWS4-HMAC-SHA256"}},
                {{"x-amz-credential": "{}/{}/{}/s3/aws4_request"}},
                {{"x-amz-date": "{}"}},
                {{"Content-Disposition": "inline"}}
            ]
        }}"#,
        expiration,
        auth.bucket,
        target_key,
        max_size,
        auth.access_key,
        date,
        auth.region,
        date_time
    );

    let policy_base64 = general_purpose::STANDARD.encode(policy);
    let signature = sign_policy(&policy_base64, &auth.secret_key, &date, &auth.region);

    // Construct fields map with exact key
    let mut fields = HashMap::new();
    fields.insert("key".to_string(), target_key);
    fields.insert("acl".to_string(), "private".to_string());
    fields.insert("x-amz-algorithm".to_string(), "AWS4-HMAC-SHA256".to_string());
    fields.insert(
        "x-amz-credential".to_string(), 
        format!("{}/{}/{}/s3/aws4_request", auth.access_key, date, auth.region)
    );
    fields.insert("x-amz-date".to_string(), date_time);
    fields.insert("policy".to_string(), policy_base64);
    fields.insert("x-amz-signature".to_string(), signature);
    fields.insert("Content-Disposition".to_string(), "inline".to_string());

    Ok(DiskUploadResponse {
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


pub async fn delete_s3_object(
    file_key: &str,
    auth: &AwsBucketAuth,
) -> Result<(), String> {
    debug_log!("Deleting S3 object: {} using endpoint: {}", file_key, auth.endpoint);
    
    // Determine the host for S3 - use endpoint if provided, otherwise use standard S3 format
    let host = if !auth.endpoint.is_empty() {
        let endpoint = auth.endpoint
            .trim_start_matches("https://")
            .trim_start_matches("http://")
            .trim_end_matches('/');
        endpoint.to_string()
    } else {
        format!("{}.s3.{}.amazonaws.com", auth.bucket, auth.region)
    };
    
    debug_log!("Host for signing: {}", host);
    
    let current_time = ic_cdk::api::time();
    let date = format_date(current_time);
    let date_time = format_datetime(current_time);
    
    // Build credential string
    let credential = format!("{}/{}/{}/s3/aws4_request", 
        auth.access_key, date, auth.region);
    
    // For S3, we need to use the ?delete query parameter with POST method
    // This is the S3 API method for deleting an object when DELETE HTTP method is not available
    let url = if !auth.endpoint.is_empty() {
        // If custom endpoint is provided
        format!("{}/{}/{}?delete", auth.endpoint.trim_end_matches('/'), auth.bucket, file_key)
    } else {
        // Standard S3 endpoint
        format!("https://{}.s3.{}.amazonaws.com/{}?delete", auth.bucket, auth.region, file_key)
    };
    
    debug_log!("Delete request URL: {}", url);
    
    // Create XML body for the delete operation
    let delete_body = format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<Delete>
  <Object>
    <Key>{}</Key>
  </Object>
  <Quiet>true</Quiet>
</Delete>"#, file_key);
    
    // Calculate Content-MD5 (required for POST requests with body)
    let content_md5 = {
        let digest = md5::compute(delete_body.as_bytes());
        base64::engine::general_purpose::STANDARD.encode(digest.0)
    };
    
    debug_log!("Content-MD5: {}", content_md5);
    
    // Calculate SHA256 hash of payload for signing
    let payload_hash = hex::encode(sha256_hash(delete_body.as_bytes()));
    debug_log!("Payload hash: {}", payload_hash);
    
    // Construct the canonical URI - for virtual-hosted-style S3, this is just the object key
    let canonical_uri = format!("/{}", file_key);
    let canonical_query_string = "delete=";
    
    // Headers for the request
    let headers = vec![
        HttpHeader {
            name: "Host".to_string(),
            value: host.clone(),
        },
        HttpHeader {
            name: "Content-Type".to_string(),
            value: "application/xml".to_string(),
        },
        HttpHeader {
            name: "Content-MD5".to_string(),
            value: content_md5.clone(),
        },
        HttpHeader {
            name: "Content-Length".to_string(),
            value: delete_body.len().to_string(),
        },
        HttpHeader {
            name: "x-amz-date".to_string(),
            value: date_time.clone(),
        },
        HttpHeader {
            name: "x-amz-content-sha256".to_string(),
            value: payload_hash.clone(),
        },
    ];
    
    // Create canonical headers string (alphabetically sorted by header name)
    let canonical_headers = format!(
        "content-length:{}\ncontent-md5:{}\ncontent-type:application/xml\nhost:{}\nx-amz-content-sha256:{}\nx-amz-date:{}\n",
        delete_body.len(),
        content_md5,
        host,
        payload_hash,
        date_time
    );
    
    let signed_headers = "content-length;content-md5;content-type;host;x-amz-content-sha256;x-amz-date";

    // Create canonical request
    let canonical_request = format!(
        "POST\n{}\n{}\n{}\n{}\n{}",
        canonical_uri,
        canonical_query_string,
        canonical_headers,
        signed_headers,
        payload_hash
    );

    debug_log!("Canonical request: {}", canonical_request);

    // Create string to sign
    let string_to_sign = format!(
        "AWS4-HMAC-SHA256\n{}\n{}/{}/s3/aws4_request\n{}",
        date_time,
        date,
        auth.region,
        hex::encode(sha256_hash(canonical_request.as_bytes()))
    );

    debug_log!("String to sign: {}", string_to_sign);

    // Calculate signature
    let signing_key = derive_signing_key(&auth.secret_key, &date, &auth.region, "s3");
    let signature = hex::encode(hmac_sha256(&signing_key, string_to_sign.as_bytes()));

    // Create Authorization header
    let authorization = format!(
        "AWS4-HMAC-SHA256 Credential={},SignedHeaders={},Signature={}",
        credential, signed_headers, signature
    );

    debug_log!("Authorization: {}", authorization);

    // Add Authorization header to headers vec
    let mut final_headers = headers;
    final_headers.push(HttpHeader {
        name: "Authorization".to_string(),
        value: authorization,
    });

    // Log all headers for debugging
    for header in &final_headers {
        debug_log!("Header: {} = {}", header.name, header.value);
    }
    
    // Create the HTTP request
    let request = CanisterHttpRequestArgument {
        url,
        method: HttpMethod::POST, // Using POST since ICP doesn't support DELETE
        headers: final_headers,
        body: Some(delete_body.into_bytes()),
        max_response_bytes: Some(4096),
        transform: None,
    };

    // Make the HTTP request
    let cycles: u128 = 100_000_000_000;
    
    debug_log!("Sending delete request...");
    
    match http_request(request, cycles).await {
        Ok((response,)) => {
            let status_u16: u16 = response.status.0.to_u64()
                .and_then(|n| {
                    if n <= u16::MAX as u64 {
                        Some(n as u16)
                    } else {
                        None
                    }
                })
                .unwrap_or(500);

            debug_log!("Delete response status: {}", status_u16);
            
            let response_body = String::from_utf8_lossy(&response.body);
            debug_log!("Delete response body: {}", response_body);
            
            // S3 DeleteObjects API returns 200 OK on success
            if status_u16 >= 200 && status_u16 < 300 {
                debug_log!("Object deleted successfully");
                Ok(())
            } else {
                let error_msg = format!("S3 delete failed with status {}: {}", 
                    status_u16, response_body);
                debug_log!("Delete failed: {}", error_msg);
                Err(error_msg)
            }
        },
        Err((code, msg)) => {
            let error_msg = format!("HTTP request failed: {:?} - {}", code, msg);
            debug_log!("Delete request error: {}", error_msg);
            Err(error_msg)
        }
    }
}