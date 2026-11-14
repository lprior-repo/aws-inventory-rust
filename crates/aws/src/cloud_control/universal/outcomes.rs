use super::*;

use aws_sdk_cloudcontrol::error::SdkError;
use aws_sdk_cloudcontrol::operation::list_resources::ListResourcesError;

pub(super) fn list_error(error: SdkError<ListResourcesError>) -> ApiError {
    if let Some(ListResourcesError::TypeNotFoundException(service)) = error.as_service_error() {
        let detail = format!(
            "TypeNotFoundException: {}",
            service.message().unwrap_or("no service message")
        );
        return ApiError::UnsupportedType(detail);
    }
    sdk_error(error)
}

pub(crate) async fn collect(ctx: &Context, types: &[ResourceType]) -> Result<(), AwsScanError> {
    if ctx.executor.is_cancelled() {
        return Err(ApiError::Cancelled.into());
    }
    let client = Client::new(&ctx.config);
    let page_size = list_page_size(ctx);
    collect_bounded(types, TYPE_CONCURRENCY, |descriptor| {
        list_one(ctx, &client, descriptor, page_size)
    })
    .await?;
    Ok(())
}

async fn list_one(
    ctx: &Context,
    client: &Client,
    descriptor: &ResourceType,
    page_size: i32,
) -> Result<(), AwsScanError> {
    collect_type(ctx, descriptor, |token| {
        let type_name = descriptor.type_name.clone();
        async move {
            ctx.execute("cloudcontrol", "read", || async {
                client
                    .list_resources()
                    .type_name(&type_name)
                    .max_results(page_size)
                    .set_next_token(token.clone())
                    .send()
                    .await
                    .map_err(list_error)
            })
            .await
        }
    })
    .await
}

pub(super) async fn collect_type<L, LFut>(
    ctx: &Context,
    descriptor: &ResourceType,
    list: L,
) -> Result<(), AwsScanError>
where
    L: Fn(Option<String>) -> LFut + Send + Sync,
    LFut: Future<Output = Result<ListResourcesOutput, ApiError>> + Send,
{
    let ledger = Mutex::new(IdentityLedger::default());
    let scope = ctx.scope(
        "cloud_control_list",
        Some(&descriptor.type_name),
        Source::CloudControl,
    );
    let failure_scope = scope.clone();
    paginate(ctx, scope, |token, budget| {
        let page = PageContext::new(&ledger, &failure_scope, budget);
        collect_page(ctx, descriptor, &list, page, token)
    })
    .await
}

struct PageContext<'a> {
    ledger: &'a Mutex<IdentityLedger>,
    scope: &'a CollectionScope,
    budget: usize,
}

impl<'a> PageContext<'a> {
    fn new(ledger: &'a Mutex<IdentityLedger>, scope: &'a CollectionScope, budget: usize) -> Self {
        Self {
            ledger,
            scope,
            budget,
        }
    }
}

async fn collect_page<L, LFut>(
    ctx: &Context,
    descriptor: &ResourceType,
    list: &L,
    page: PageContext<'_>,
    token: Option<String>,
) -> Result<PageBody, AwsScanError>
where
    L: Fn(Option<String>) -> LFut + Send + Sync,
    LFut: Future<Output = Result<ListResourcesOutput, ApiError>> + Send,
{
    let output = match list_page(ctx, list, page.scope, token).await? {
        Some(output) => output,
        None => return Ok(PageBody::default()),
    };
    remember_list_page(descriptor, &output, page.ledger, page.budget)?;
    collect_descriptions(ctx, descriptor, output.resource_descriptions()).await?;
    Ok(PageBody {
        listed_count: output.resource_descriptions().len(),
        next_token: output.next_token().map(str::to_owned),
        ..PageBody::default()
    })
}

async fn list_page<L, LFut>(
    ctx: &Context,
    list: &L,
    failure_scope: &CollectionScope,
    token: Option<String>,
) -> Result<Option<ListResourcesOutput>, AwsScanError>
where
    L: Fn(Option<String>) -> LFut + Send + Sync,
    LFut: Future<Output = Result<ListResourcesOutput, ApiError>> + Send,
{
    if ctx.executor.is_cancelled() {
        return Err(ApiError::Cancelled.into());
    }
    match list(token).await {
        Ok(output) => Ok(Some(output)),
        Err(ApiError::UnsupportedType(message)) => {
            record_unsupported(ctx, failure_scope, message).await?;
            Ok(None)
        }
        Err(error) => Err(error.into()),
    }
}

async fn collect_descriptions(
    ctx: &Context,
    descriptor: &ResourceType,
    descriptions: &[ResourceDescription],
) -> Result<(), AwsScanError> {
    collect_bounded(descriptions, READ_CONCURRENCY, |description| {
        read_description(ctx, descriptor, description)
    })
    .await?;
    Ok(())
}

async fn read_description(
    ctx: &Context,
    descriptor: &ResourceType,
    description: &ResourceDescription,
) -> Result<(), AwsScanError> {
    let identifier = identifier(description)?;
    let parent = format!(
        "{}:{:x}",
        descriptor.type_name,
        base16ct::HexDisplay(&Sha256::digest(identifier.as_bytes()))
    );
    let scope = ctx.scope("cloud_control_get", Some(&parent), Source::CloudControl);
    let body = map_description(ctx, descriptor, identifier, description, false)?;
    emit_child(ctx, &scope, body).await
}
