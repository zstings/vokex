use serde_json::{json, Value};
use std::io::{BufRead, BufReader};
use std::thread;
use std::time::Duration;
use ureq::tls::{TlsConfig, TlsProvider, RootCerts};
use ureq::typestate::{WithBody, WithoutBody};

/// 根据状态码返回标准 HTTP 状态文本
fn status_text(code: u16) -> &'static str {
    match code {
        100 => "Continue",
        101 => "Switching Protocols",
        102 => "Processing",
        103 => "Early Hints",
        200 => "OK",
        201 => "Created",
        202 => "Accepted",
        203 => "Non-Authoritative Information",
        204 => "No Content",
        205 => "Reset Content",
        206 => "Partial Content",
        207 => "Multi-Status",
        208 => "Already Reported",
        226 => "IM Used",
        300 => "Multiple Choices",
        301 => "Moved Permanently",
        302 => "Found",
        303 => "See Other",
        304 => "Not Modified",
        305 => "Use Proxy",
        307 => "Temporary Redirect",
        308 => "Permanent Redirect",
        400 => "Bad Request",
        401 => "Unauthorized",
        402 => "Payment Required",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        406 => "Not Acceptable",
        407 => "Proxy Authentication Required",
        408 => "Request Timeout",
        409 => "Conflict",
        410 => "Gone",
        411 => "Length Required",
        412 => "Precondition Failed",
        413 => "Payload Too Large",
        414 => "URI Too Long",
        415 => "Unsupported Media Type",
        416 => "Range Not Satisfiable",
        417 => "Expectation Failed",
        418 => "I'm a Teapot",
        421 => "Misdirected Request",
        422 => "Unprocessable Entity",
        423 => "Locked",
        424 => "Failed Dependency",
        425 => "Too Early",
        426 => "Upgrade Required",
        428 => "Precondition Required",
        429 => "Too Many Requests",
        431 => "Request Header Fields Too Large",
        451 => "Unavailable For Legal Reasons",
        500 => "Internal Server Error",
        501 => "Not Implemented",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        504 => "Gateway Timeout",
        505 => "HTTP Version Not Supported",
        506 => "Variant Also Negotiates",
        507 => "Insufficient Storage",
        508 => "Loop Detected",
        510 => "Not Extended",
        511 => "Network Authentication Required",
        _ => "Unknown",
    }
}

/// 构建 ureq Agent，设置超时 + native-tls（使用系统证书存储）
fn build_agent(timeout: u64) -> ureq::Agent {
    let tls_config = TlsConfig::builder()
        .provider(TlsProvider::NativeTls)
        .root_certs(RootCerts::PlatformVerifier)
        .build();

    let config = ureq::Agent::config_builder()
        .tls_config(tls_config)
        .timeout_global(Some(Duration::from_secs(timeout)))
        .build();

    ureq::Agent::new_with_config(config)
}

/// 发送请求（带 body 或不带 body）
fn send_request(
    request: ureq::RequestBuilder<WithBody>,
    body: Option<&str>,
) -> Result<ureq::http::Response<ureq::Body>, ureq::Error> {
    match body {
        Some(b) => request.send(b),
        None => request.send_empty(),
    }
}

/// 发送无 body 请求
fn send_request_no_body(
    request: ureq::RequestBuilder<WithoutBody>,
) -> Result<ureq::http::Response<ureq::Body>, ureq::Error> {
    request.call()
}

/// 构建请求，应用 headers
fn apply_headers(
    mut request: ureq::RequestBuilder<WithBody>,
    headers: &Value,
) -> ureq::RequestBuilder<WithBody> {
    if let Some(obj) = headers.as_object() {
        for (key, val) in obj {
            if let Some(s) = val.as_str() {
                request = request.header(key.as_str(), s);
            }
        }
    }
    request
}

/// 构建无 body 请求，应用 headers
fn apply_headers_no_body(
    mut request: ureq::RequestBuilder<WithoutBody>,
    headers: &Value,
) -> ureq::RequestBuilder<WithoutBody> {
    if let Some(obj) = headers.as_object() {
        for (key, val) in obj {
            if let Some(s) = val.as_str() {
                request = request.header(key.as_str(), s);
            }
        }
    }
    request
}

/// 提取响应头为 JSON Map
fn extract_headers(response: &ureq::http::Response<ureq::Body>) -> serde_json::Map<String, Value> {
    let mut map = serde_json::Map::new();
    for (name, value) in response.headers() {
        if let Ok(v) = value.to_str() {
            map.insert(name.as_str().to_string(), Value::String(v.to_string()));
        }
    }
    map
}

/// 处理普通（非流式）请求
fn handle_normal_request(
    agent: ureq::Agent,
    method: &str,
    url: &str,
    headers: &Value,
    body: Option<&str>,
) -> Result<Value, String> {
    let response = match method {
        "GET" => {
            let req = agent.get(url);
            let req = apply_headers_no_body(req, headers);
            send_request_no_body(req)
        }
        "HEAD" => {
            let req = agent.head(url);
            let req = apply_headers_no_body(req, headers);
            send_request_no_body(req)
        }
        "OPTIONS" => {
            let req = agent.delete(url); // ureq 3.x 没有 options 方法，用 request
            let req = apply_headers_no_body(req, headers);
            send_request_no_body(req)
        }
        "POST" => {
            let req = agent.post(url);
            let req = apply_headers(req, headers);
            send_request(req, body)
        }
        "PUT" => {
            let req = agent.put(url);
            let req = apply_headers(req, headers);
            send_request(req, body)
        }
        "DELETE" => {
            let req = agent.delete(url);
            let req = apply_headers_no_body(req, headers);
            send_request_no_body(req)
        }
        "PATCH" => {
            let req = agent.patch(url);
            let req = apply_headers(req, headers);
            send_request(req, body)
        }
        _ => return Err(format!("Unsupported HTTP method: {}", method)),
    }
    .map_err(|e| format!("HTTP request failed: {}", e))?;

    let status = response.status().as_u16();
    let headers_map = extract_headers(&response);
    let body_str = response.into_body().read_to_string().unwrap_or_default();

    Ok(json!({
        "statusCode": status,
        "statusText": status_text(status),
        "headers": headers_map,
        "body": body_str,
        "ok": status >= 200 && status < 300
    }))
}

/// 处理流式（SSE）请求
fn handle_stream_request(
    agent: ureq::Agent,
    method: &str,
    url: &str,
    headers: &Value,
    body: Option<&str>,
    task_id: u64,
    window_id: u32,
) -> Result<Value, String> {
    let response = match method {
        "GET" => {
            let req = agent.get(url);
            let req = apply_headers_no_body(req, headers);
            send_request_no_body(req)
        }
        "HEAD" => {
            let req = agent.head(url);
            let req = apply_headers_no_body(req, headers);
            send_request_no_body(req)
        }
        "POST" => {
            let req = agent.post(url);
            let req = apply_headers(req, headers);
            send_request(req, body)
        }
        "PUT" => {
            let req = agent.put(url);
            let req = apply_headers(req, headers);
            send_request(req, body)
        }
        "DELETE" => {
            let req = agent.delete(url);
            let req = apply_headers_no_body(req, headers);
            send_request_no_body(req)
        }
        "PATCH" => {
            let req = agent.patch(url);
            let req = apply_headers(req, headers);
            send_request(req, body)
        }
        _ => return Err(format!("Unsupported HTTP method: {}", method)),
    }
    .map_err(|e| format!("HTTP request failed: {}", e))?;

    // 立即提取响应元数据
    let status = response.status().as_u16();
    let status_text_str = status_text(status);
    let headers_map = extract_headers(&response);

    // 发送 response-start 事件
    crate::ipc::emit_via_proxy(
        window_id,
        format!("http.response-start.{}", task_id),
        json!({
            "statusCode": status,
            "statusText": status_text_str,
            "headers": headers_map,
        }),
    );

    // 启动后台线程逐行读取响应体
    let task_id_clone = task_id;
    let window_id_clone = window_id;
    thread::spawn(move || {
        let reader = response.into_body().into_reader();
        let mut buf_reader = BufReader::new(reader);
        let mut line = String::new();

        loop {
            line.clear();
            match buf_reader.read_line(&mut line) {
                Ok(0) => break, // EOF
                Ok(_) => {
                    crate::ipc::emit_via_proxy(
                        window_id_clone,
                        format!("http.chunk.{}", task_id_clone),
                        json!({ "data": line.clone() }),
                    );
                }
                Err(e) => {
                    crate::ipc::emit_via_proxy(
                        window_id_clone,
                        format!("http.error.{}", task_id_clone),
                        json!({ "error": e.to_string() }),
                    );
                    return;
                }
            }
        }

        // 流结束
        crate::ipc::emit_via_proxy(
            window_id_clone,
            format!("http.end.{}", task_id_clone),
            json!({}),
        );
    });

    Ok(json!({ "taskId": task_id }))
}

pub fn handle(method: &str, params: &Value, window_id: u32) -> Result<Value, String> {
    match method {
        "http.request" => {
            let url = params.get("url").and_then(|v| v.as_str())
                .ok_or("Missing 'url' parameter")?;
            let http_method = params.get("method").and_then(|v| v.as_str()).unwrap_or("GET").to_uppercase();
            let headers = params.get("headers").cloned().unwrap_or(json!({}));
            let body = params.get("body").and_then(|v| v.as_str());
            let timeout = params.get("timeout").and_then(|v| v.as_u64()).unwrap_or(30);
            let stream = params.get("stream").and_then(|v| v.as_bool()).unwrap_or(false);
            let task_id = params.get("taskId").and_then(|v| v.as_u64()).unwrap_or(0);

            let agent = build_agent(timeout);

            if stream {
                handle_stream_request(agent, &http_method, url, &headers, body, task_id, window_id)
            } else {
                handle_normal_request(agent, &http_method, url, &headers, body)
            }
        }
        _ => Err(format!("Unknown method: {}", method)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_missing_url() {
        let result = handle("http.request", &json!({}), 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_unsupported_method() {
        let result = handle("http.request", &json!({
            "url": "http://example.com",
            "method": "INVALID"
        }), 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_unknown_method() {
        assert!(handle("http.unknownMethod", &json!({}), 0).is_err());
    }

    #[test]
    fn test_request_to_closed_port() {
        let result = handle("http.request", &json!({
            "url": "http://127.0.0.1:1/",
            "method": "GET",
            "timeout": 2
        }), 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_status_text_mapping() {
        assert_eq!(status_text(200), "OK");
        assert_eq!(status_text(404), "Not Found");
        assert_eq!(status_text(500), "Internal Server Error");
        assert_eq!(status_text(999), "Unknown");
    }
}
