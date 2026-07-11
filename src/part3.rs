fn read_request(stream: &mut TcpStream) -> Result<HttpRequest, String> {
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .map_err(|error| error.to_string())?;
    let mut buffer = Vec::new();
    let mut chunk = [0u8; 4096];
    let mut header_end = None;

    while buffer.len() < 65_536 {
        let count = stream.read(&mut chunk).map_err(|error| error.to_string())?;
        if count == 0 {
            break;
        }
        buffer.extend_from_slice(&chunk[..count]);
        if let Some(position) = find_bytes(&buffer, b"\r\n\r\n") {
            header_end = Some(position + 4);
            break;
        }
    }

    let header_end = header_end.ok_or_else(|| "Malformed HTTP request.".to_string())?;
    let (method, target, content_length) = {
        let headers_text = String::from_utf8_lossy(&buffer[..header_end]);
        let mut lines = headers_text.lines();
        let request_line = lines
            .next()
            .ok_or_else(|| "Missing request line.".to_string())?;
        let mut parts = request_line.split_whitespace();
        let method = parts.next().unwrap_or("").to_string();
        let target = parts.next().unwrap_or("/").to_string();
        let content_length = lines
            .find_map(|line| {
                let (name, value) = line.split_once(':')?;
                if name.eq_ignore_ascii_case("content-length") {
                    value.trim().parse::<usize>().ok()
                } else {
                    None
                }
            })
            .unwrap_or(0);
        (method, target, content_length)
    };

    while buffer.len() < header_end + content_length {
        let count = stream.read(&mut chunk).map_err(|error| error.to_string())?;
        if count == 0 {
            break;
        }
        buffer.extend_from_slice(&chunk[..count]);
    }

    let body_end = (header_end + content_length).min(buffer.len());
    let body_text = String::from_utf8_lossy(&buffer[header_end..body_end]);
    let (path, query_text) = target.split_once('?').unwrap_or((target.as_str(), ""));
    Ok(HttpRequest {
        method,
        path: path.to_string(),
        query: parse_form(query_text),
        body: parse_form(&body_text),
    })
}

fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|window| window == needle)
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
                if let (Some(high), Some(low)) = (hex_value(bytes[index + 1]), hex_value(bytes[index + 2])) {
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
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\nX-Content-Type-Options: nosniff\r\n",
        body.as_bytes().len()
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
    Ok(name.to_string())
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
    format!("{:016x}{:016x}", next_random(app), next_random(app))
}
