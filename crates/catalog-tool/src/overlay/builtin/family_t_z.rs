#[cfg(test)]
mod tests;

mod completion;
mod context_plans;
mod endpoint_plans;
mod extra_plans;
mod tag_plans;
mod taxsettings;
mod textract;
mod textract_plans_1;
mod timestream_influxdb;
mod timestream_influxdb_plans_1;
mod timestream_influxdb_plans_2;
mod timestream_influxdb_plans_3;
mod timestream_query;
mod timestream_write;
mod transcribe;
mod transcribe_plans_1;
mod transcribe_streaming;
mod transfer;
mod transfer_plans_1;
mod transfer_plans_2;
mod translate;
mod translate_plans_1;
mod trustedadvisor;
mod uxc;
mod verifiedpermissions;
mod verifiedpermissions_plans_1;
mod voice_id;
mod voice_id_plans_1;
mod vpc_lattice;
mod vpc_lattice_plans_1;
mod vpc_lattice_plans_2;
mod vpc_lattice_plans_3;
mod waf;
mod waf_plans_1;
mod waf_plans_2;
mod waf_regional;
mod waf_regional_plans_1;
mod waf_regional_plans_2;
mod wafv2;
mod wellarchitected;
mod wellarchitected_plans_1;
mod wickr;
mod wickr_plans_1;
mod wisdom;
mod wisdom_plans_1;
mod workdocs;
mod workmail;
mod workmail_plans_1;
mod workmailmessageflow;
mod workspaces;
mod workspaces_instances;
mod workspaces_instances_plans_1;
mod workspaces_plans_1;
mod workspaces_thin_client;
mod workspaces_thin_client_plans_1;
mod workspaces_web;
mod workspaces_web_plans_1;
mod workspaces_web_plans_2;
mod xray;
mod xray_plans_1;

use super::{empty, name, path};
use crate::{
    classes::DenyReason,
    overlay::{
        Binding, Identity, ItemSelector, MemberPath, NameError, OperationOverlay, OverlayClass,
        Paginator, Scope, ServiceOverlay, TokenPair,
    },
};

pub(super) fn overlays() -> Result<Vec<ServiceOverlay>, NameError> {
    completion::apply(tag_plans::apply(context_plans::apply(
        endpoint_plans::apply(vec![
            taxsettings::overlay()?,
            textract::overlay()?,
            timestream_influxdb::overlay()?,
            timestream_query::overlay()?,
            timestream_write::overlay()?,
            transcribe::overlay()?,
            transcribe_streaming::overlay()?,
            transfer::overlay()?,
            translate::overlay()?,
            trustedadvisor::overlay()?,
            uxc::overlay()?,
            verifiedpermissions::overlay()?,
            voice_id::overlay()?,
            vpc_lattice::overlay()?,
            waf::overlay()?,
            waf_regional::overlay()?,
            wafv2::overlay()?,
            wellarchitected::overlay()?,
            wickr::overlay()?,
            wisdom::overlay()?,
            workdocs::overlay()?,
            workmail::overlay()?,
            workmailmessageflow::overlay()?,
            workspaces::overlay()?,
            workspaces_instances::overlay()?,
            workspaces_thin_client::overlay()?,
            workspaces_web::overlay()?,
            xray::overlay()?,
        ])?,
    )?)?)
}

fn deny(reason: DenyReason) -> OperationOverlay {
    OperationOverlay {
        class: Some(OverlayClass::Deny(reason)),
        ..empty()
    }
}

fn defer(reason: &str) -> OperationOverlay {
    OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some(reason.to_owned()),
        ..empty()
    }
}

fn paths(values: &[&str]) -> Result<Vec<MemberPath>, NameError> {
    values.iter().map(|value| path(value)).collect()
}

fn token(input: &str, output: &str) -> Result<Paginator, NameError> {
    Ok(Paginator::Token(TokenPair {
        input: path(input)?,
        output: path(output)?,
        optional: false,
    }))
}

fn inventory(
    items: &str,
    identity: Identity,
    paginator: Paginator,
    scope: Scope,
) -> Result<OperationOverlay, NameError> {
    Ok(OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(scope),
        items: Some(ItemSelector::parse(items)?),
        identity: Some(identity),
        paginator: Some(paginator),
        ..empty()
    })
}

fn bind(
    operation: &mut OperationOverlay,
    input: &str,
    parent: &str,
    field: &str,
) -> Result<(), NameError> {
    operation.class = Some(OverlayClass::Expand);
    operation.bind.insert(
        path(input)?,
        Binding::From {
            operation: name(parent)?,
            path: path(field)?,
        },
    );
    Ok(())
}
