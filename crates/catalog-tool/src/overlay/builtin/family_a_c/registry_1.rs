use super::support::{NameError, ServiceOverlay, service};

pub(super) fn overlays() -> Result<Vec<ServiceOverlay>, NameError> {
    Ok(vec![
        service("amplify", [
            super::amplify_0::part_0()?,
        ].into_iter().flatten().collect())?,
        service("amplifybackend", [
            super::amplifybackend_0::part_0()?,
        ].into_iter().flatten().collect())?,
        service("amplifyuibuilder", [
            super::amplifyuibuilder_0::part_0()?,
        ].into_iter().flatten().collect())?,
        service("api-gateway", [
            super::api_gateway_0::part_0()?,
            super::api_gateway_0::part_1()?,
            super::api_gateway_0::part_2()?,
            super::api_gateway_0::part_3()?,
            super::api_gateway_0::part_4()?,
        ].into_iter().flatten().collect())?,
        service("apigatewaymanagementapi", [
            super::apigatewaymanagementapi_0::part_0()?,
        ].into_iter().flatten().collect())?,
        service("apigatewayv2", [
            super::apigatewayv2_0::part_0()?,
            super::apigatewayv2_0::part_1()?,
            super::apigatewayv2_0::part_2()?,
            super::apigatewayv2_0::part_3()?,
        ].into_iter().flatten().collect())?,
        service("app-mesh", [
            super::app_mesh_0::part_0()?,
            super::app_mesh_0::part_1()?,
        ].into_iter().flatten().collect())?,
        service("appconfig", [
            super::appconfig_0::part_0()?,
            super::appconfig_0::part_1()?,
            super::appconfig_0::part_2()?,
        ].into_iter().flatten().collect())?,
        service("appconfigdata", [
            super::appconfigdata_0::part_0()?,
        ].into_iter().flatten().collect())?,
        service("appfabric", [
            super::appfabric_0::part_0()?,
            super::appfabric_0::part_1()?,
        ].into_iter().flatten().collect())?,
    ])
}
