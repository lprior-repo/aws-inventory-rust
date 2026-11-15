use super::*;

pub(super) fn request_line(line: &str) -> Result<(Method, String)> {
    let mut parts = line.split_ascii_whitespace();
    let method = Method::from_bytes(
        parts
            .next()
            .context("proxy request method is missing")?
            .as_bytes(),
    )?;
    let target = parts.next().context("proxy request target is missing")?;
    ensure!(
        target.starts_with('/')
            && !target.starts_with("//")
            && parts.next() == Some("HTTP/1.1")
            && parts.next().is_none(),
        "proxy accepts only origin-form HTTP/1.1 requests"
    );
    Ok((method, target.to_owned()))
}

pub(super) fn request_headers<'a>(lines: impl Iterator<Item = &'a str>) -> Result<HeaderMap> {
    let mut headers = HeaderMap::new();
    for line in lines {
        let Some((name, value)) = line.split_once(':') else {
            anyhow::bail!("proxy request header is malformed");
        };
        let name = HeaderName::from_bytes(name.trim().as_bytes())?;
        if hop_by_hop(&name) {
            continue;
        }
        let value = HeaderValue::from_bytes(value.trim().as_bytes())?;
        headers.append(name, value);
    }
    Ok(headers)
}

pub(super) fn hop_by_hop(name: &HeaderName) -> bool {
    [
        header::CONNECTION,
        header::TRANSFER_ENCODING,
        header::UPGRADE,
    ]
    .iter()
    .any(|value| name == value)
        || name.as_str().eq_ignore_ascii_case("keep-alive")
        || name.as_str().eq_ignore_ascii_case("proxy-connection")
}

pub(super) fn content_length(headers: &HeaderMap) -> Result<usize> {
    headers
        .get(header::CONTENT_LENGTH)
        .context("AWS request omitted Content-Length")?
        .to_str()?
        .parse()
        .context("AWS request Content-Length is invalid")
}

pub(super) fn continuation_checkpoint(body: &[u8]) -> Option<ContinuationCheckpoint> {
    let action = if contains(body, b"Action=DescribeVpcs") {
        "DescribeVpcs"
    } else if contains(body, b"Action=DescribeSubnets") {
        "DescribeSubnets"
    } else {
        return None;
    };
    let token = body
        .split(|byte| *byte == b'&')
        .find_map(|field| field.strip_prefix(b"NextToken="))?;
    (!token.is_empty()).then(|| ContinuationCheckpoint {
        action,
        token: token.to_vec(),
    })
}

pub(super) fn contains(bytes: &[u8], needle: &[u8]) -> bool {
    bytes.windows(needle.len()).any(|window| window == needle)
}

pub(super) async fn write_response(
    stream: &mut tokio_rustls::server::TlsStream<TcpStream>,
    response: &aws_wire::transport::Response,
) -> Result<()> {
    let mut head = format!(
        "HTTP/1.1 {} {}\r\n",
        response.status.as_u16(),
        response
            .status
            .canonical_reason()
            .context("AWS response status has no reason")?
    );
    for name in [
        "content-type",
        "content-encoding",
        "x-amzn-requestid",
        "x-amz-request-id",
        "x-amzn-errortype",
    ] {
        if let Some(value) = response.headers.get(name) {
            head.push_str(&format!("{name}: {}\r\n", value.to_str()?));
        }
    }
    head.push_str(&format!(
        "content-length: {}\r\nconnection: close\r\n\r\n",
        response.body.len()
    ));
    tokio::time::timeout(Duration::from_secs(5), stream.write_all(head.as_bytes())).await??;
    tokio::time::timeout(Duration::from_secs(5), stream.write_all(&response.body)).await??;
    Ok(())
}
