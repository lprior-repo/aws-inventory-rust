mod types;
use aws_sigv4::{http_request as aws, sign::v4};
use http::{HeaderMap, HeaderName, HeaderValue, Uri};
use std::time::SystemTime;
pub use types::{Body, Credentials, PathMode, PayloadHeader, Region, Request, Scope, SigningName};

#[derive(Debug, thiserror::Error, Eq, PartialEq)]
#[non_exhaustive]
pub enum Error {
    #[error("invalid signing credentials")]
    Credentials,

    #[error("invalid signing region")]
    Region,

    #[error("invalid signing name")]
    SigningName,

    #[error("invalid signing URI")]
    Uri,

    #[error("invalid signing headers")]
    Headers,

    #[error("signing input exceeds its bound")]
    Bounds,

    #[error("invalid precomputed SHA-256")]
    Digest,

    #[error("signing time outside 1970..=9999")]
    Time,

    #[error("SigV4 signing failed")]
    Signing,
}

pub fn sign(
    credentials: &Credentials,
    scope: &Scope<'_>,
    time: SystemTime,
    request: Request<'_>,
    path: PathMode,
) -> Result<HeaderMap, Error> {
    validate_time(time)?;
    let checksum = request.payload_header;
    let request = signable_request(request)?;
    let params = parameters(credentials, scope, time, path, checksum)?;
    let (instructions, _) = aws::sign(request, &params)
        .map_err(|_| Error::Signing)?
        .into_parts();
    instructions
        .headers()
        .try_fold(HeaderMap::with_capacity(4), output_header)
}

fn validate_time(time: SystemTime) -> Result<(), Error> {
    let seconds = time
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_err(|_| Error::Time)?;
    if seconds.as_secs() > 253_402_300_799 {
        return Err(Error::Time);
    }
    Ok(())
}

fn signable_request(request: Request<'_>) -> Result<aws::SignableRequest<'_>, Error> {
    let uri = validate_uri(request.uri)?;
    if request.method.as_str().len() > 64 {
        return Err(Error::Bounds);
    }
    let headers = request_headers(request.headers, &uri)?;
    aws::SignableRequest::new(
        request.method.as_str(),
        request.uri,
        headers.into_iter(),
        signable_body(request.body)?,
    )
    .map_err(|_| Error::Signing)
}

fn parameters<'a>(
    credentials: &'a Credentials,
    scope: &Scope<'a>,
    time: SystemTime,
    path: PathMode,
    checksum: PayloadHeader,
) -> Result<aws::SigningParams<'a>, Error> {
    v4::SigningParams::builder()
        .identity(&credentials.0)
        .region(scope.region.0)
        .name(scope.name.0)
        .time(time)
        .settings(settings(path, checksum))
        .build()
        .map(Into::into)
        .map_err(|_| Error::Signing)
}

fn output_header(mut out: HeaderMap, (key, value): (&str, &str)) -> Result<HeaderMap, Error> {
    let mut header = HeaderValue::from_str(value).map_err(|_| Error::Signing)?;
    header.set_sensitive(key == "authorization" || key == "x-amz-security-token");
    out.insert(
        HeaderName::from_bytes(key.as_bytes()).map_err(|_| Error::Signing)?,
        header,
    );
    Ok(out)
}

fn settings(path: PathMode, checksum: PayloadHeader) -> aws::SigningSettings {
    let mut settings = aws::SigningSettings::default();
    if path == PathMode::S3 {
        settings.percent_encoding_mode = aws::PercentEncodingMode::Single;
        settings.uri_path_normalization_mode = aws::UriPathNormalizationMode::Disabled;
    }
    settings.payload_checksum_kind = match checksum {
        PayloadHeader::Omit => aws::PayloadChecksumKind::NoHeader,
        PayloadHeader::Include => aws::PayloadChecksumKind::XAmzSha256,
    };
    settings
}

fn validate_uri(value: &str) -> Result<Uri, Error> {
    if value.len() > 16_384 {
        return Err(Error::Bounds);
    }
    let uri: Uri = value.parse().map_err(|_| Error::Uri)?;
    if !matches!(uri.scheme_str(), Some("http" | "https"))
        || !uri.host().is_some_and(|h| !h.is_empty())
        || uri.authority().is_some_and(|a| a.as_str().contains('@'))
        || value.contains('#')
    {
        return Err(Error::Uri);
    }
    Ok(uri)
}

type Header<'a> = (&'a str, &'a str);

fn request_headers<'a>(headers: &'a HeaderMap, uri: &Uri) -> Result<Vec<Header<'a>>, Error> {
    if headers.len() > 128 || headers.values().any(|value| value.len() > 8192) {
        return Err(Error::Bounds);
    }
    if let Some(host) = headers.get("host") {
        let text = host.to_str().map_err(|_| Error::Headers)?;
        if headers.get_all("host").iter().count() != 1
            || !uri
                .authority()
                .is_some_and(|a| a.as_str().eq_ignore_ascii_case(text))
        {
            return Err(Error::Headers);
        }
    }
    headers.iter().map(request_header).collect()
}

fn request_header<'a>(
    (name, value): (&'a HeaderName, &'a HeaderValue),
) -> Result<Header<'a>, Error> {
    let text = value.to_str().map_err(|_| Error::Headers)?;
    if matches!(
        name.as_str(),
        "authorization" | "x-amz-date" | "x-amz-content-sha256" | "x-amz-security-token"
    ) {
        return Err(Error::Headers);
    }
    Ok((name.as_str(), text))
}

fn signable_body(body: Body<'_>) -> Result<aws::SignableBody<'_>, Error> {
    match body {
        Body::Bytes(bytes) if bytes.len() <= 16_777_216 => Ok(aws::SignableBody::Bytes(bytes)),
        Body::Bytes(_) => Err(Error::Bounds),
        Body::Sha256(hex) => {
            if hex.len() != 64
                || !hex
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            {
                return Err(Error::Digest);
            }
            Ok(aws::SignableBody::Precomputed(hex.to_owned()))
        }
    }
}
