use super::*;
use inventory_core::{catalog::runtime::ContextSource, record::FieldId};

pub(super) struct Fields<'a, 'ctx, 'catalog> {
    pub row: &'a Record,
    pub trusted: Option<&'ctx IdentityContext<'ctx>>,
    pub context: Option<&'a DiscoveryContext>,
    pub request: Option<&'a Record>,
    pub validation: &'ctx IdentityTextValidation<'ctx, 'catalog>,
}

impl<'a> Fields<'a, '_, '_> {
    pub(super) fn texts(&self, fields: &[IdentityFieldPlan]) -> Result<Cow<'a, str>, SweepError> {
        if fields.is_empty() || fields.len() > 64 {
            return Err(SweepError::Operation(
                "invalid correlated identity field count",
            ));
        }
        if let [field] = fields {
            return self.text(field);
        }
        fields
            .iter()
            .enumerate()
            .try_fold(String::new(), |text, (position, field)| {
                super::super::composite::append(text, position, &self.text(field)?)
            })
            .map(Cow::Owned)
    }

    pub(super) fn text(&self, field: &IdentityFieldPlan) -> Result<Cow<'a, str>, SweepError> {
        self.scalar(field)
            .or_else(|| self.integer(field))
            .or_else(|| self.string_list(field))
            .ok_or(SweepError::Operation(
                "unexpected contextual identity field",
            ))?
    }

    fn scalar(&self, field: &IdentityFieldPlan) -> Option<Result<Cow<'a, str>, SweepError>> {
        match field {
            IdentityFieldPlan::Selected(path) => Some(identity_text(self.row, path)),
            IdentityFieldPlan::Request(path) => Some(
                self.request()
                    .and_then(|record| identity_text(record, path)),
            ),
            IdentityFieldPlan::SelectedArn(path) | IdentityFieldPlan::RequestArn(path) => {
                Some(self.arn(field, path))
            }
            IdentityFieldPlan::SelectedDigest(path) => Some(digest(self.row, path).map(Cow::Owned)),
            IdentityFieldPlan::MapValue { path, key } => Some(map_text(self.row, path, key)),
            IdentityFieldPlan::Context {
                service,
                operation,
                source,
                path,
            } => Some(self.context_text(service, operation, *source, path)),
            _ => None,
        }
    }

    fn integer(&self, field: &IdentityFieldPlan) -> Option<Result<Cow<'a, str>, SweepError>> {
        let (record, path, minimum, maximum) = match field {
            IdentityFieldPlan::SelectedInteger {
                path,
                minimum,
                maximum,
            } => (Ok(self.row), path, *minimum, *maximum),
            IdentityFieldPlan::RequestInteger {
                path,
                minimum,
                maximum,
            } => (self.request(), path, *minimum, *maximum),
            _ => return None,
        };
        Some(record.and_then(|record| typed_operand::integer(record, path, minimum, maximum)))
    }

    fn string_list(&self, field: &IdentityFieldPlan) -> Option<Result<Cow<'a, str>, SweepError>> {
        let IdentityFieldPlan::SelectedStringList {
            path,
            max_items,
            max_item_bytes,
        } = field
        else {
            return None;
        };
        Some(
            self.validation
                .selected_operand(path)
                .and_then(|validation| {
                    typed_operand::string_list(
                        self.row,
                        path,
                        *max_items,
                        *max_item_bytes,
                        &validation,
                    )
                }),
        )
    }

    fn request(&self) -> Result<&'a Record, SweepError> {
        self.request.ok_or(SweepError::Operation(
            "identity requires actual request context",
        ))
    }

    fn arn(&self, field: &IdentityFieldPlan, path: &[FieldId]) -> Result<Cow<'a, str>, SweepError> {
        let record = if matches!(field, IdentityFieldPlan::SelectedArn(_)) {
            self.row
        } else {
            self.request()?
        };
        let value = path_value(record.root(), path)?
            .ok_or(SweepError::Operation("modeled identity path is absent"))?;
        let ValueRef::Text(text) = value else {
            return Err(SweepError::Operation(
                "modeled ARN identity operand must be text",
            ));
        };
        validate_arn(text, self.trusted)?;
        Ok(Cow::Borrowed(text))
    }

    fn context_text(
        &self,
        service: &str,
        operation: &str,
        source: ContextSource,
        path: &[FieldId],
    ) -> Result<Cow<'a, str>, SweepError> {
        self.context
            .ok_or(SweepError::Operation(
                "identity requires durable discovery context",
            ))?
            .value(service, operation, source, path)
            .map_err(|_| SweepError::Operation("invalid durable discovery context"))?
            .ok_or(SweepError::Operation(
                "required discovery ancestor identity is absent",
            ))
            .and_then(value_text)
    }
}

fn digest(row: &Record, path: &[FieldId]) -> Result<String, SweepError> {
    let value = path_value(row.root(), path)?
        .ok_or(SweepError::Operation("modeled identity path is absent"))?;
    let empty = match value {
        ValueRef::Null => true,
        ValueRef::Map(map) => map.count()? == 0,
        ValueRef::List(list) => list.count()? == 0,
        _ => false,
    };
    if empty {
        return Err(SweepError::Operation(
            "modeled identity digest value is null or empty",
        ));
    }
    digest_text(value)
}

fn digest_text(value: ValueRef<'_>) -> Result<String, SweepError> {
    inventory_core::record::selected_content_hash(value)?
        .as_bytes()
        .iter()
        .try_fold(String::from("blake3:"), |mut text, byte| {
            write!(text, "{byte:02x}")
                .map_err(|_| SweepError::Operation("identity digest formatting failed"))?;
            Ok(text)
        })
}
