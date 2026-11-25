use super::{EncodeError, RequestContext};
use inventory_core::{catalog::Target, record::ValueRef};

pub(super) fn suppress_prefix(context: RequestContext<'_>) -> Result<bool, EncodeError> {
    let uri = context
        .endpoint
        .parse::<http::Uri>()
        .map_err(|_| EncodeError::InvalidEndpoint)?;
    if uri.host().is_some_and(loopback) {
        return Ok(true);
    }
    if context.service.endpoint_prefix()? != Some("s3-control") {
        return Ok(false);
    }
    let Target::Shape(index) = context.operation.input()? else {
        return Ok(false);
    };
    let input = context
        .service
        .shape(index)?
        .ok_or(EncodeError::MissingInput)?;
    input.members()?.try_fold(false, |found, entry| {
        let member = entry?;
        if member.name()? != "OutpostId" {
            return Ok(found);
        }
        match context.input.get_field(member.id)? {
            None | Some(ValueRef::Null) => Ok(found),
            Some(ValueRef::Text(value)) if valid_outpost(value) => Ok(true),
            _ => Err(EncodeError::InvalidValue {
                field: Some(member.id),
                reason: "invalid OutpostId",
            }),
        }
    })
}

fn valid_outpost(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
}

fn loopback(host: &str) -> bool {
    host.eq_ignore_ascii_case("localhost")
        || host
            .trim_matches(['[', ']'])
            .parse::<std::net::IpAddr>()
            .is_ok_and(|ip| ip.is_loopback())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn emulator_prefix_suppression_uses_the_same_loopback_forms_as_configuration() {
        for host in [
            "localhost",
            "LOCALHOST",
            "127.0.0.1",
            "127.1.2.3",
            "[::1]",
            "::1",
        ] {
            assert!(loopback(host), "{host}");
        }
        for host in [
            "localhost.evil.example",
            "127.0.0.1.evil.example",
            "192.168.0.1",
            "[2001:db8::1]",
        ] {
            assert!(!loopback(host), "{host}");
        }
    }
}
