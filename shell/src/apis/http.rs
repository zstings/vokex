use serde_json::{json, Value};
use minreq::Method;

/// 根据状态码返回标准 HTTP 状态文本
fn status_text(code: i32) -> &'static str {
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

pub fn handle(method: &str, params: &Value) -> Result<Value, String> {
    match method {
        "http.request" => {
            let url = params.get("url").and_then(|v| v.as_str())
                .ok_or("Missing 'url' parameter")?;
            let http_method = params.get("method").and_then(|v| v.as_str()).unwrap_or("GET").to_uppercase();
            let headers = params.get("headers").cloned().unwrap_or(json!({}));
            let body = params.get("body").and_then(|v| v.as_str());
            let timeout = params.get("timeout").and_then(|v| v.as_u64()).unwrap_or(30);

            let method_enum = match http_method.as_str() {
                "GET" => Method::Get,
                "POST" => Method::Post,
                "PUT" => Method::Put,
                "DELETE" => Method::Delete,
                "PATCH" => Method::Patch,
                "HEAD" => Method::Head,
                "OPTIONS" => Method::Options,
                _ => return Err(format!("Unsupported HTTP method: {}", http_method)),
            };

            let mut request = minreq::Request::new(method_enum, url)
                .with_timeout(timeout);

            if let Some(obj) = headers.as_object() {
                for (key, val) in obj {
                    if let Some(s) = val.as_str() {
                        request = request.with_header(key, s);
                    }
                }
            }

            if let Some(b) = body {
                request = request.with_body(b);
            }

            let response = request.send()
                .map_err(|e| format!("HTTP request failed: {}", e))?;

            let status = response.status_code;
            let body_str = response.as_str().unwrap_or("").to_string();

            Ok(json!({
                "statusCode": status,
                "statusText": status_text(status),
                "headers": response.headers,
                "body": body_str,
                "ok": status >= 200 && status < 300
            }))
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
        let result = handle("http.request", &json!({}));
        assert!(result.is_err());
    }

    #[test]
    fn test_unsupported_method() {
        let result = handle("http.request", &json!({
            "url": "http://example.com",
            "method": "INVALID"
        }));
        assert!(result.is_err());
    }

    #[test]
    fn test_unknown_method() {
        assert!(handle("http.unknownMethod", &json!({})).is_err());
    }

    #[test]
    fn test_request_to_closed_port() {
        // 连接到一个未开放的端口，应快速失败
        let result = handle("http.request", &json!({
            "url": "http://127.0.0.1:1/",
            "method": "GET",
            "timeout": 2
        }));
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
