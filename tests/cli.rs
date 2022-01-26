use std::{collections::HashMap, fs::read_to_string, time::Duration};

use assert_cmd::Command;
use predicates::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize)]
struct Response {
    protocol: String,
    status_code: u64,
    headers: HashMap<String, String>,
    content: String,
}

#[derive(Serialize, Deserialize)]
struct JsonResponse {
    protocol: String,
    status_code: u64,
    headers: HashMap<String, String>,
    content: Value,
}

#[test]
fn parse_single_response() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("hj")?;

    cmd.timeout(Duration::from_secs(1))
        .write_stdin(read_to_string("tests/response-single.txt")?.as_bytes());

    let assert = cmd.assert().success().stderr(predicate::str::is_empty());
    let output_str = assert.get_output().stdout.clone();
    let result: Response = serde_json::from_slice(&output_str)?;

    assert_eq!(result.protocol, "HTTP/3");
    assert_eq!(result.status_code, 200);
    assert_eq!(result.headers["content-type"], "text/html");
    assert!(result.content.contains("<!DOCTYPE html>"));

    Ok(())
}

#[test]
fn parse_multi_responses() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("hj")?;

    cmd.timeout(Duration::from_secs(1))
        .arg("--array")
        .write_stdin(read_to_string("tests/response-multi.txt")?.as_bytes());

    let assert = cmd.assert().success().stderr(predicate::str::is_empty());
    let output_str = assert.get_output().stdout.clone();
    let results: Vec<Response> = serde_json::from_slice(&output_str)?;

    assert_eq!(results.len(), 3);

    for result in results {
        assert_eq!(result.protocol, "HTTP/3");
        assert_eq!(result.status_code, 200);
        assert_eq!(result.headers["content-type"], "text/html");
        assert!(result.content.contains("<!DOCTYPE html>"));
    }

    Ok(())
}

#[test]
fn parse_curl_sv_as_raw() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("hj")?;

    // curl -sv https://httpbin.org/get > tests/response-curl-sv-httpbin.txt 2>&1
    cmd.timeout(Duration::from_secs(1))
        .arg("--raw")
        .write_stdin(read_to_string("tests/response-curl-sv-httpbin.txt")?.as_bytes());

    let assert = cmd.assert().success().stderr(predicate::str::is_empty());
    let output_str = assert.get_output().stdout.clone();
    let result: Response = serde_json::from_slice(&output_str)?;

    assert_eq!(result.protocol, "HTTP/1.1");
    assert_eq!(result.status_code, 200);
    assert_eq!(result.headers["content-type"], "application/json");

    Ok(())
}

#[test]
fn parse_curl_sv_as_json() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("hj")?;

    // curl -sv https://httpbin.org/get > tests/response-curl-sv-httpbin.txt 2>&1
    cmd.timeout(Duration::from_secs(1))
        .write_stdin(read_to_string("tests/response-curl-sv-httpbin.txt")?.as_bytes());

    let assert = cmd.assert().success().stderr(predicate::str::is_empty());
    let output_str = assert.get_output().stdout.clone();
    let result: JsonResponse = serde_json::from_slice(&output_str)?;

    assert_eq!(result.protocol, "HTTP/1.1");
    assert_eq!(result.status_code, 200);
    assert_eq!(result.headers["content-type"], "application/json");
    assert_eq!(result.content["url"], "https://httpbin.org/get");

    Ok(())
}

#[test]
fn parse_curl_sv_as_json_jsonplaceholder() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("hj")?;

    // curl -sv https://jsonplaceholder.typicode.com/todos/1 2>&1 | tee tests/response-jsonplaceholder.txt
    cmd.timeout(Duration::from_secs(1))
        .write_stdin(read_to_string("tests/response-jsonplaceholder.txt")?.as_bytes());

    let assert = cmd.assert().success().stderr(predicate::str::is_empty());
    let output_str = assert.get_output().stdout.clone();
    let result: JsonResponse = serde_json::from_slice(&output_str)?;

    assert_eq!(result.protocol, "HTTP/1.1");
    assert_eq!(result.status_code, 200);
    assert_eq!(
        result.headers["content-type"],
        "application/json; charset=utf-8"
    );
    assert_eq!(result.content["id"], 1);

    Ok(())
}
