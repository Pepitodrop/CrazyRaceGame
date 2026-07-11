const MAX_HEADER_BYTES: usize = 16 * 1024;
const MAX_BODY_BYTES: usize = 8 * 1024;
const MAX_TARGET_BYTES: usize = 2_048;
const IO_TIMEOUT_SECONDS: u64 = 5;
const SECURITY_HEADERS: &str = concat!(
    "Cache-Control: no-store, max-age=0\r\n",
    "Pragma: no-cache\r\n",
    "Expires: 0\r\n",
    "Referrer-Policy: no-referrer\r\n",
    "X-Frame-Options: DENY\r\n",
    "X-XSS-Protection: 0\r\n",
    "Cross-Origin-Opener-Policy: same-origin\r\n",
    "Cross-Origin-Resource-Policy: same-origin\r\n",
    "Permissions-Policy: camera=(), microphone=(), geolocation=(), payment=(), usb=()\r\n",
    "Content-Security-Policy: default-src 'none'; style-src 'unsafe-inline'; form-action 'self'; base-uri 'none'; frame-ancestors 'none'\r\n"
);

fn run_healthcheck() -> bool {
    let address = env::var("BIND_ADDRESS").unwrap_or_else(|_| DEFAULT_BIND.to_string());
    let port = address.rsplit(':').next().unwrap_or("8080");
    let target = match format!("127.0.0.1:{port}").parse::<std::net::SocketAddr>() {
        Ok(target) => target,
        Err(_) => return false,
    };
    let mut stream = match TcpStream::connect_timeout(&target, Duration::from_secs(2)) {
        Ok(stream) => stream,
        Err(_) => return false,
    };
    if stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .is_err()
        || stream
            .set_write_timeout(Some(Duration::from_secs(2)))
            .is_err()
    {
        return false;
    }
    if stream
        .write_all(b"GET /health HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
        .is_err()
    {
        return false;
    }
    let mut response = String::new();
    stream.read_to_string(&mut response).is_ok()
        && response.starts_with("HTTP/1.1 200 OK")
        && response.contains("\r\n\r\nok")
}

fn read_request(stream: &mut TcpStream) -> Result<HttpRequest, RouteError> {
    stream
        .set_read_timeout(Some(Duration::from_secs(IO_TIMEOUT_SECONDS)))
        .map_err(|_| internal_error())?;
    stream
        .set_write_timeout(Some(Duration::from_secs(IO_TIMEOUT_SECONDS)))
        .map_err(|_| internal_error())?;

    let mut buffer = Vec::with_capacity(4_096);
    let mut chunk = [0u8; 4_096];
    let header_end = loop {
        if buffer.len() >= MAX_HEADER_BYTES {
            return Err((
                "431 Request Header Fields Too Large",
                "Request headers are too large.".to_string(),
            ));
        }
        let read_limit = chunk.len().min(MAX_HEADER_BYTES - buffer.len());
        let count = read_from_stream(stream, &mut chunk[..read_limit])?;
        if count == 0 {
            return Err((
                "400 Bad Request",
                "Connection closed before request headers completed.".to_string(),
            ));
        }
        buffer.extend_from_slice(&chunk[..count]);
        if let Some(position) = find_bytes(&buffer, b"\r\n\r\n") {
            break position + 4;
        }
    };

    let (method, target, content_length) = {
        let headers_text = std::str::from_utf8(&buffer[..header_end]).map_err(|_| {
            (
                "400 Bad Request",
                "Request headers must be valid UTF-8.".to_string(),
            )
        })?;
        let mut lines = headers_text.split("\r\n");
        let request_line = lines
            .next()
            .ok_or(("400 Bad Request", "Missing request line.".to_string()))?;
        let mut parts = request_line.split_whitespace();
        let method = parts.next().unwrap_or("");
        let target = parts.next().unwrap_or("");
        let version = parts.next().unwrap_or("");
        if method.is_empty()
            || target.is_empty()
            || !matches!(version, "HTTP/1.0" | "HTTP/1.1")
            || parts.next().is_some()
        {
            return Err((
                "400 Bad Request",
                "Malformed HTTP request line.".to_string(),
            ));
        }
        if !target.starts_with('/') {
            return Err((
                "400 Bad Request",
                "Only origin-form request targets are accepted.".to_string(),
            ));
        }
        if target.len() > MAX_TARGET_BYTES {
            return Err((
                "414 URI Too Long",
                "Request target is too long.".to_string(),
            ));
        }

        let mut content_length = None;
        for line in lines.filter(|line| !line.is_empty()) {
            let (name, value) = line
                .split_once(':')
                .ok_or(("400 Bad Request", "Malformed request header.".to_string()))?;
            if name.eq_ignore_ascii_case("content-length") {
                let parsed = value.trim().parse::<usize>().map_err(|_| {
                    (
                        "400 Bad Request",
                        "Invalid Content-Length header.".to_string(),
                    )
                })?;
                if content_length.replace(parsed).is_some() {
                    return Err((
                        "400 Bad Request",
                        "Duplicate Content-Length headers are not accepted.".to_string(),
                    ));
                }
            } else if name.eq_ignore_ascii_case("transfer-encoding")
                && !value.trim().is_empty()
                && !value.trim().eq_ignore_ascii_case("identity")
            {
                return Err((
                    "501 Not Implemented",
                    "Transfer-Encoding is not supported.".to_string(),
                ));
            }
        }

        (
            method.to_string(),
            target.to_string(),
            content_length.unwrap_or(0),
        )
    };

    if content_length > MAX_BODY_BYTES {
        return Err((
            "413 Payload Too Large",
            format!("Request bodies are limited to {MAX_BODY_BYTES} bytes."),
        ));
    }
    let request_end = header_end.checked_add(content_length).ok_or((
        "413 Payload Too Large",
        "Request size overflow.".to_string(),
    ))?;

    while buffer.len() < request_end {
        let remaining = request_end - buffer.len();
        let read_limit = chunk.len().min(remaining);
        let count = read_from_stream(stream, &mut chunk[..read_limit])?;
        if count == 0 {
            return Err((
                "400 Bad Request",
                "Connection closed before request body completed.".to_string(),
            ));
        }
        buffer.extend_from_slice(&chunk[..count]);
    }

    let body_text = std::str::from_utf8(&buffer[header_end..request_end]).map_err(|_| {
        (
            "400 Bad Request",
            "Request body must be valid UTF-8.".to_string(),
        )
    })?;
    let (path, query_text) = target.split_once('?').unwrap_or((target.as_str(), ""));
    Ok(HttpRequest {
        method,
        path: path.to_string(),
        query: parse_form(query_text),
        body: parse_form(body_text),
    })
}

fn read_from_stream(stream: &mut TcpStream, buffer: &mut [u8]) -> Result<usize, RouteError> {
    stream.read(buffer).map_err(|error| match error.kind() {
        std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock => {
            ("408 Request Timeout", "Request timed out.".to_string())
        }
        _ => ("400 Bad Request", "Could not read the request.".to_string()),
    })
}

fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

fn parse_form(value: &str) -> HashMap<String, String> {
    value
        .split('&')
        .filter(|pair| !pair.is_empty())
        .map(|pair| {
            let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
            (url_decode(key), url_decode(value))
        })
        .collect()
}

fn url_decode(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'+' => {
                output.push(b' ');
                index += 1;
            }
            b'%' if index + 2 < bytes.len() => {
                if let (Some(high), Some(low)) =
                    (hex_value(bytes[index + 1]), hex_value(bytes[index + 2]))
                {
                    output.push(high * 16 + low);
                    index += 3;
                } else {
                    output.push(bytes[index]);
                    index += 1;
                }
            }
            byte => {
                output.push(byte);
                index += 1;
            }
        }
    }
    String::from_utf8_lossy(&output).into_owned()
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn send_response(
    stream: &mut TcpStream,
    status: &str,
    content_type: &str,
    body: &str,
    extra_headers: &[(String, String)],
) -> std::io::Result<()> {
    let mut response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\nX-Content-Type-Options: nosniff\r\n{SECURITY_HEADERS}",
        body.len()
    );
    for (name, value) in extra_headers {
        response.push_str(&format!("{name}: {value}\r\n"));
    }
    response.push_str("\r\n");
    response.push_str(body);
    stream.write_all(response.as_bytes())
}

fn redirect_to_game(code: &str, token: &str) -> RouteResponse {
    redirect(format!(
        "/game?room={}&token={}",
        url_encode(code),
        url_encode(token)
    ))
}

fn redirect_to_local_game(code: &str, token: &str) -> RouteResponse {
    redirect(format!(
        "/local-game?room={}&token={}",
        url_encode(code),
        url_encode(token)
    ))
}

fn redirect(location: String) -> RouteResponse {
    (
        "303 See Other",
        "text/plain; charset=utf-8",
        "Redirecting…".to_string(),
        vec![("Location".to_string(), location)],
    )
}

fn url_encode(value: &str) -> String {
    value
        .bytes()
        .map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' => (byte as char).to_string(),
            _ => format!("%{byte:02X}"),
        })
        .collect()
}

fn clean_name(value: Option<&String>) -> Result<String, RouteError> {
    let name = value.map(|name| name.trim()).unwrap_or("");
    if name.is_empty() {
        return Err(("400 Bad Request", "Enter a racer name.".to_string()));
    }
    if name.chars().count() > 24 {
        return Err((
            "400 Bad Request",
            "Racer names are limited to 24 characters.".to_string(),
        ));
    }
    if name.chars().any(char::is_control) {
        return Err((
            "400 Bad Request",
            "Racer names cannot contain control characters.".to_string(),
        ));
    }
    Ok(name.to_string())
}

fn clean_room_code(value: &str) -> Result<String, RouteError> {
    const ALPHABET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
    let code = value.trim().to_ascii_uppercase();
    if code.len() != 6 || !code.bytes().all(|byte| ALPHABET.contains(&byte)) {
        return Err((
            "400 Bad Request",
            "Room codes contain exactly six supported letters or digits.".to_string(),
        ));
    }
    Ok(code)
}

fn required_query(request: &HttpRequest, key: &str) -> Result<String, RouteError> {
    request
        .query
        .get(key)
        .cloned()
        .filter(|value| !value.is_empty())
        .ok_or(("400 Bad Request", format!("Missing {key}.")))
}

fn required_body(request: &HttpRequest, key: &str) -> Result<String, RouteError> {
    request
        .body
        .get(key)
        .cloned()
        .filter(|value| !value.is_empty())
        .ok_or(("400 Bad Request", format!("Missing {key}.")))
}

fn internal_error() -> RouteError {
    (
        "500 Internal Server Error",
        "Game state is temporarily unavailable.".to_string(),
    )
}

fn env_usize(name: &str, default: usize, minimum: usize, maximum: usize) -> usize {
    env::var(name)
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .map(|value| value.clamp(minimum, maximum))
        .unwrap_or(default)
}

fn env_u64(name: &str, default: u64, minimum: u64, maximum: u64) -> u64 {
    env::var(name)
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .map(|value| value.clamp(minimum, maximum))
        .unwrap_or(default)
}

fn now_epoch() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn prune_expired_rooms(app: &mut AppState) {
    let now = now_epoch();
    let ttl = app.room_ttl_seconds;
    app.rooms
        .retain(|_, room| now.saturating_sub(room.last_activity) <= ttl);
}

fn ensure_room_capacity(app: &mut AppState) -> Result<(), RouteError> {
    prune_expired_rooms(app);
    if app.rooms.len() >= app.max_rooms {
        return Err((
            "503 Service Unavailable",
            "The server has reached its active-room limit. Try again later.".to_string(),
        ));
    }
    Ok(())
}

fn next_random(app: &mut AppState) -> u64 {
    app.entropy ^= app.entropy << 13;
    app.entropy ^= app.entropy >> 7;
    app.entropy ^= app.entropy << 17;
    app.entropy = app.entropy.wrapping_add(0x9E37_79B9_7F4A_7C15);
    app.entropy
}

fn unique_room_code(app: &mut AppState) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
    loop {
        let mut value = next_random(app);
        let code: String = (0..6)
            .map(|_| {
                let character = ALPHABET[(value as usize) % ALPHABET.len()] as char;
                value /= ALPHABET.len() as u64;
                character
            })
            .collect();
        if !app.rooms.contains_key(&code) {
            return code;
        }
    }
}

fn next_token(app: &mut AppState) -> String {
    let mut bytes = [0u8; 32];
    if fs::File::open("/dev/urandom")
        .and_then(|mut source| source.read_exact(&mut bytes))
        .is_err()
    {
        eprintln!("warning: /dev/urandom unavailable; using process entropy fallback");
        for chunk in bytes.chunks_mut(8) {
            chunk.copy_from_slice(&next_random(app).to_ne_bytes());
        }
    }

    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut token = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        token.push(HEX[(byte >> 4) as usize] as char);
        token.push(HEX[(byte & 0x0f) as usize] as char);
    }
    token
}
