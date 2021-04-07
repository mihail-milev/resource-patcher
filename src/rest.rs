use reqwest::{Method, Certificate, blocking::{Client}};
use std::result::Result;

use crate::error_handler;

pub fn perform_get_request(url: &str, bearer_token: &str, cert: &str) -> Result<String, String> {
    let cert = error_handler!(Certificate::from_pem(cert.as_bytes()), "Unable to create GET request certificate");
    let client = error_handler!(Client::builder().add_root_certificate(cert).build(), "Unable to crete GET client");
    let request = error_handler!(
                    client.request(Method::GET, url)
                        .bearer_auth(bearer_token)
                        .build(), "Unable to create GET request".to_string());
    let result = error_handler!(client.execute(request), "Unable to execute GET request".to_string());
    if result.status().is_success() {
        return Ok(error_handler!(result.text(), "Unable to parse GET request result".to_string()));
    }
    return Err(error_handler!(result.text(), "Unable to parse GET request result".to_string()));
}

pub fn perform_patch_request(url: &str, bearer_token: &str, cert: &str, body: String) -> Result<String, String> {
    let cert = error_handler!(Certificate::from_pem(cert.as_bytes()), "Unable to create PATCH request certificate");
    let client = error_handler!(Client::builder().add_root_certificate(cert).build(), "Unable to crete PATCH client");
    let request = error_handler!(
                    client.request(Method::PATCH, url)
                        .bearer_auth(bearer_token)
                        .header("Content-Type", "application/strategic-merge-patch+json")
                        .body(body)
                        .build(), "Unable to create PATCH request".to_string());
    let result = error_handler!(client.execute(request), "Unable to execute PATCH request".to_string());
    if result.status().is_success() {
        return Ok(error_handler!(result.text(), "Unable to parse PATCH request result".to_string()));
    }
    return Err(error_handler!(result.text(), "Unable to parse PATCH request result".to_string()));
}
