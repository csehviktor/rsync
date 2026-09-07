use chrono::Utc;
use hmac::{KeyInit, Mac};
use sha2::Digest;

use crate::persistance::config::Credentials;

// source: https://docs.aws.amazon.com/IAM/latest/UserGuide/reference_sigv-create-signed-request.html
const REGION: &str = "auto";
const SERVICE: &str = "s3";

type HmacSha256 = hmac::Hmac<sha2::Sha256>;

pub struct Signed {
    pub amz_date: String,
    pub authorization: String,
}

pub fn encode_uri_path(path: &str) -> String {
    path.split('/')
        .map(encode_uri_component)
        .collect::<Vec<_>>()
        .join("/")
}

pub fn canonical_query(params: &[(&str, &str)]) -> String {
    let mut encoded: Vec<String> = params
        .iter()
        .map(|(k, v)| format!("{}={}", encode_uri_component(k), encode_uri_component(v)))
        .collect();

    encoded.sort();
    encoded.join("&")
}

pub fn sign(
    method: &str,
    host: &str,
    uri: &str,
    query: &str,
    payload_hash: &str,
    credentials: &Credentials,
) -> Signed {
    let (amz_date, date) = timestamp();

    // canonical request
    let signed_headers = "host;x-amz-content-sha256;x-amz-date";
    let canonical_headers =
        format!("host:{host}\nx-amz-content-sha256:{payload_hash}\nx-amz-date:{amz_date}\n");
    let canonical_request =
        format!("{method}\n{uri}\n{query}\n{canonical_headers}\n{signed_headers}\n{payload_hash}");

    // string to sign
    let canonical_request_hash = hash(canonical_request.as_bytes());
    let scope = format!("{date}/{REGION}/{SERVICE}/aws4_request");
    let string_to_sign = format!("AWS4-HMAC-SHA256\n{amz_date}\n{scope}\n{canonical_request_hash}");

    let key = signing_key(credentials, &date);
    let signature = hex::encode(hmac(&key, string_to_sign.as_bytes()));

    let authorization = format!(
        "AWS4-HMAC-SHA256Credential={}/{scope},SignedHeaders=host;x-amz-content-sha256;x-amz-date,Signature={signature}",
        credentials.access_key_id
    );

    Signed {
        amz_date,
        authorization,
    }
}

pub fn hash(data: &[u8]) -> String {
    hex::encode(sha2::Sha256::digest(data))
}

fn hmac(key: &[u8], data: &[u8]) -> Vec<u8> {
    let mut mac = HmacSha256::new_from_slice(key).unwrap();
    mac.update(data);
    mac.finalize().into_bytes().to_vec()
}

fn signing_key(credentials: &Credentials, date: &str) -> Vec<u8> {
    let secret = format!("AWS4{}", credentials.secret_access_key);

    let key = hmac(secret.as_bytes(), date.as_bytes()); // date key
    let key = hmac(&key, REGION.as_bytes()); // date region key
    let key = hmac(&key, SERVICE.as_bytes()); // date region service key

    hmac(&key, b"aws4_request") // signing key
}

fn timestamp() -> (String, String) {
    let now = Utc::now();

    (
        now.format("%Y%m%dT%H%M%SZ").to_string(), // amz-date
        now.format("%Y%m%d").to_string(),         // date
    )
}

// aws uri-encoding: only unreserved characters stay unescaped
pub fn encode_uri_component(text: &str) -> String {
    let mut encoded = String::with_capacity(text.len());

    for byte in text.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char)
            }
            _ => encoded.push_str(&format!("%{byte:02X}")),
        }
    }

    encoded
}
