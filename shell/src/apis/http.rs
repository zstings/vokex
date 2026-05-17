use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Read};
use std::thread;
use std::time::Duration;

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

/// 构建 ureq Agent，设置超时
fn build_agent(timeout: u64) -> ureq::Agent {
    let builder = ureq::AgentBuilder::new();
    builder.timeout(Duration::from_secs(timeout)).build()
}

/// 构建请求，应用 headers
fn build_request(agent: &ureq::Agent, method: &str, url: &str, headers: &Value) -> Result<ureq::Request, String> {
    let mut request = match method {
        "GET" => agent.get(url),
        "POST" => agent.post(url),
        "PUT" => agent.put(url),
        "DELETE" => agent.delete(url),
        "PATCH" => agent.patch(url),
        "HEAD" => agent.head(url),
        "OPTIONS" => agent.request("OPTIONS", url),
        _ => return Err(format!("Unsupported HTTP method: {}", method)),
    };

    if let Some(obj) = headers.as_object() {
        for (key, val) in obj {
            if let Some(s) = val.as_str() {
                request = request.set(key, s);
            }
        }
    }

    Ok(request)
}

/// 提取响应头为 JSON Map
fn extract_headers(response: &ureq::Response) -> serde_json::Map<String, Value> {
    let mut map = serde_json::Map::new();
    for name in response.headers_names() {
        if let Some(value) = response.header(&name) {
            map.insert(name, Value::String(value.to_string()));
        }
    }
    map
}

/// 发送请求（带 body 或不带 body）
fn send_request(request: ureq::Request, body: Option<&str>) -> Result<ureq::Response, String> {
    match body {
        Some(b) => request.send_string(b).map_err(|e| format!("HTTP request failed: {}", e)),
        None => request.call().map_err(|e| format!("HTTP request failed: {}", e)),
    }
}

/// 处理普通（非流式）请求
fn handle_normal_request(
    agent: ureq::Agent,
    method: &str,
    url: &str,
    headers: &Value,
    body: Option<&str>,
) -> Result<Value, String> {
    let request = build_request(&agent, method, url, headers)?;
    let response = send_request(request, body)?;

    let status = response.status();
    let headers_map = extract_headers(&response);
    let body_str = response.into_string().unwrap_or_default();

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
    let request = build_request(&agent, method, url, headers)?;
    let response = send_request(request, body)?;

    // 立即提取响应元数据
    let status = response.status();
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
        let reader: Box<dyn Read + Send> = response.into_reader();
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
