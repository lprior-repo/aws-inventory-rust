use super::cases::CaseCounts;
use super::report::Service;
use super::{DerivedEndpoint, Error, OverlayNeed, ServiceClass, SkipReason};
use std::fmt::Write as _;

pub(super) fn render(counts: &CaseCounts, services: &[Service]) -> Result<String, Error> {
    let mut text = String::new();
    count_lines(&mut text, counts)?;
    class_lines(&mut text, services)?;
    exception_counts(&mut text, services)?;
    services
        .iter()
        .try_for_each(|service| service_lines(&mut text, service))?;
    Ok(text)
}

fn skip_lines(text: &mut String, counts: &CaseCounts) -> Result<(), std::fmt::Error> {
    use SkipReason as S;
    [
        (S::OtherPartition, "other_partition"),
        (S::Fips, "fips"),
        (S::DualStack, "dual_stack"),
        (S::CustomEndpoint, "custom_endpoint"),
        (S::OtherParameters, "other_parameters"),
        (S::OperationInputs, "operation_inputs"),
        (S::RequestTimeFunction, "request_time_function"),
    ]
    .iter()
    .try_for_each(|(reason, label)| {
        let count = counts
            .skipped
            .get(reason)
            .copied()
            .into_iter()
            .sum::<usize>();
        writeln!(text, "skipped {label} {count}")
    })
}

fn classification(endpoint: &DerivedEndpoint) -> &'static str {
    match endpoint {
        DerivedEndpoint::Resolved {
            class: ServiceClass::Global,
            ..
        } => "global",
        DerivedEndpoint::Resolved { .. } => "regional",
        DerivedEndpoint::NeedsOverlay { .. } | DerivedEndpoint::NeedsOverlayRouting { .. } => {
            "needs_overlay"
        }
        DerivedEndpoint::Unsupported { .. } => "unsupported",
    }
}

fn service_line(text: &mut String, service: &Service) -> Result<(), std::fmt::Error> {
    let name = &service.name;
    let class = classification(&service.endpoint);
    match &service.endpoint {
        DerivedEndpoint::Resolved {
            url_template,
            signing_name,
            signing_region,
            ..
        } => writeln!(
            text,
            "service {name} {class} {url_template} {signing_name} {signing_region}"
        ),
        DerivedEndpoint::NeedsOverlayRouting { .. } => routing_line(text, service),
        other => unresolved_line(text, name, other),
    }
}

fn unresolved_line(
    text: &mut String,
    name: &str,
    endpoint: &DerivedEndpoint,
) -> Result<(), std::fmt::Error> {
    match endpoint {
        DerivedEndpoint::NeedsOverlay { need } => overlay_line(text, name, need),
        DerivedEndpoint::Unsupported { reason } => {
            writeln!(text, "service {name} unsupported - - - {reason:?}")
        }
        _ => Err(std::fmt::Error),
    }
}

fn overlay_line(text: &mut String, name: &str, need: &OverlayNeed) -> Result<(), std::fmt::Error> {
    match need {
        OverlayNeed::Parameter(parameter) => {
            writeln!(
                text,
                "service {name} needs-overlay - - - missing:{parameter}"
            )
        }
        OverlayNeed::UnsupportedFunction(function) => writeln!(
            text,
            "service {name} needs-overlay - - - unsupported-function:{function}"
        ),
    }
}

fn routing_line(text: &mut String, service: &Service) -> Result<(), std::fmt::Error> {
    let DerivedEndpoint::NeedsOverlayRouting {
        url_template,
        signing_name,
        signing_region,
        matches,
        ..
    } = &service.endpoint
    else {
        return Err(std::fmt::Error);
    };
    let reason = if *matches == 0 {
        "regional template not confirmed by any published case"
    } else {
        "regional template contradicted by more published cases than it matches"
    };
    let name = &service.name;
    writeln!(
        text,
        "service {name} needs-overlay {url_template} {signing_name} {signing_region} reason:{reason:?}"
    )
}

fn count_lines(text: &mut String, counts: &CaseCounts) -> Result<(), std::fmt::Error> {
    let skipped = counts
        .skipped
        .values()
        .fold(0usize, |a, b| a.saturating_add(*b));
    writeln!(text, "cases {}", counts.in_scope.saturating_add(skipped))?;
    writeln!(
        text,
        "in_scope {}\npassed {}\nfailed {}\nskipped {skipped}",
        counts.in_scope, counts.passed, counts.failed
    )?;
    skip_lines(text, counts)
}

fn class_lines(text: &mut String, services: &[Service]) -> Result<(), std::fmt::Error> {
    writeln!(text, "services {}", services.len())?;
    ["global", "regional", "needs_overlay", "unsupported"]
        .iter()
        .try_for_each(|class| {
            let count = services
                .iter()
                .filter(|s| classification(&s.endpoint) == *class)
                .count();
            writeln!(text, "{class} {count}")
        })
}

fn exception_counts(text: &mut String, services: &[Service]) -> Result<(), std::fmt::Error> {
    let affected = services.iter().filter(|s| !s.exceptions.is_empty()).count();
    let count = services
        .iter()
        .fold(0usize, |n, s| n.saturating_add(s.exceptions.len()));
    writeln!(
        text,
        "services_with_exceptions {affected}\nexceptions {count}"
    )
}

fn service_lines(text: &mut String, service: &Service) -> Result<(), std::fmt::Error> {
    service_line(text, service)?;
    service.unsupported.iter().try_for_each(|case| {
        writeln!(
            text,
            "unsupported_case {} {} {}",
            service.name, case.index, case.name
        )
    })?;
    service.failures.iter().try_for_each(|(index, reason)| {
        writeln!(text, "case_failure {} {index} {reason:?}", service.name)
    })?;
    exception_lines(text, service)
}

fn exception_lines(text: &mut String, service: &Service) -> Result<(), std::fmt::Error> {
    service.exceptions.iter().try_for_each(|exception| {
        writeln!(
            text,
            "exception {} {} {} {} {}",
            service.name,
            exception.region,
            exception.url,
            exception.signing_name,
            exception.signing_region
        )
    })
}
