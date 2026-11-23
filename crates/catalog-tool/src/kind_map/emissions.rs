use super::validation::{row_error, valid_kind, valid_label, valid_operation, valid_service};
use super::{KindMapError, RuntimeOperation};
use std::collections::BTreeSet;

const MAX_ADDITIONAL_EMISSIONS: usize = 7;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdditionalEmissionMapping<'a> {
    pub service: &'a str,
    pub operation: &'a str,
    pub emitter_key: &'a str,
    pub kind: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewedEmission {
    pub service: String,
    pub operation: String,
    pub emitter_key: String,
    pub kind: String,
}

pub(super) fn compile(
    source: &[AdditionalEmissionMapping<'_>],
    operations: &[RuntimeOperation],
    known: &BTreeSet<&str>,
) -> Result<Vec<ReviewedEmission>, KindMapError> {
    let mut entries = source
        .iter()
        .enumerate()
        .map(|(index, row)| {
            validate(row, index, operations, known)?;
            Ok(ReviewedEmission {
                service: row.service.to_owned(),
                operation: row.operation.to_owned(),
                emitter_key: row.emitter_key.to_owned(),
                kind: row.kind.to_owned(),
            })
        })
        .collect::<Result<Vec<_>, KindMapError>>()?;
    entries.sort_by(|left, right| key(left).cmp(&key(right)));
    validate_group_bounds(&entries)?;
    Ok(entries)
}

fn validate(
    row: &AdditionalEmissionMapping<'_>,
    index: usize,
    operations: &[RuntimeOperation],
    known: &BTreeSet<&str>,
) -> Result<(), KindMapError> {
    if !valid_service(row.service)
        || !valid_operation(row.operation)
        || !valid_label(row.emitter_key)
        || !valid_kind(row.kind)
    {
        return Err(row_error(index, "invalid reviewed additional emission"));
    }
    let primary = operations
        .iter()
        .find(|operation| operation.service == row.service && operation.operation == row.operation)
        .ok_or_else(|| {
            row_error(
                index,
                "additional emission requires a reviewed primary operation",
            )
        })?;
    if !known.contains(row.kind) {
        return Err(KindMapError::UnknownKind {
            service: row.service.to_owned(),
            operation: row.operation.to_owned(),
            kind: row.kind.to_owned(),
        });
    }
    if primary.kind == row.kind {
        return Err(row_error(
            index,
            "additional emission repeats the primary kind",
        ));
    }
    Ok(())
}

fn validate_group_bounds(entries: &[ReviewedEmission]) -> Result<(), KindMapError> {
    let mut group_start = 0;
    for (index, row) in entries.iter().enumerate() {
        let previous = entries
            .get(group_start..index)
            .ok_or_else(|| row_error(index, "invalid additional emission group"))?;
        if previous
            .first()
            .is_some_and(|first| first.service != row.service || first.operation != row.operation)
        {
            group_start = index;
        }
        let siblings = entries
            .get(group_start..index)
            .ok_or_else(|| row_error(index, "invalid additional emission group"))?;
        if siblings.len() >= MAX_ADDITIONAL_EMISSIONS {
            return Err(row_error(
                index,
                "operation exceeds eight total resource emissions",
            ));
        }
        for sibling in siblings {
            if sibling.emitter_key == row.emitter_key {
                return Err(KindMapError::Duplicate(format!(
                    "{}.{}:{}",
                    row.service, row.operation, row.emitter_key
                )));
            }
            if sibling.kind == row.kind {
                return Err(row_error(
                    index,
                    "additional emissions repeat a sibling kind",
                ));
            }
        }
    }
    Ok(())
}

fn key(row: &ReviewedEmission) -> (&str, &str, &str) {
    (&row.service, &row.operation, &row.emitter_key)
}

pub(super) fn lookup<'a>(
    entries: &'a [ReviewedEmission],
    service: &str,
    operation: &str,
    emitter_key: &str,
) -> Option<&'a str> {
    let index = entries
        .binary_search_by(|row| key(row).cmp(&(service, operation, emitter_key)))
        .ok()?;
    entries.get(index).map(|row| row.kind.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kind_map::{ApiOnlyMapping, CfnMapping, MappingSource, OperationMapping};

    type TestResult = Result<(), Box<dyn std::error::Error>>;
    const PRIMARY: &str = "AWS::Route53::DNSSEC";
    const KEY: &str = "AWS::Route53::KeySigningKey";
    const OTHER: &str = "AWS::ApiOnly::OtherEmission";
    const ROW: AdditionalEmissionMapping<'static> = AdditionalEmissionMapping {
        service: "route-53",
        operation: "GetDNSSEC",
        emitter_key: "key_signing_keys",
        kind: KEY,
    };

    fn primary() -> Vec<RuntimeOperation> {
        vec![RuntimeOperation {
            service: ROW.service.to_owned(),
            operation: ROW.operation.to_owned(),
            kind: PRIMARY.to_owned(),
        }]
    }

    #[test]
    fn additional_emitter_lookup_is_exact_and_never_falls_back_to_primary() -> TestResult {
        // Given two reviewed resource emitters in reverse declaration order.
        let source = [
            ROW,
            AdditionalEmissionMapping {
                emitter_key: "a_other",
                kind: OTHER,
                ..ROW
            },
        ];
        let known = [PRIMARY, KEY, OTHER].into_iter().collect();
        // When compiling, key order is canonical and independent of the declaration order.
        let compiled = compile(&source, &primary(), &known)?;
        let [first, second] = source;
        let reversed = [second, first];
        assert_eq!(compiled, compile(&reversed, &primary(), &known)?);
        // Then only the exact operation and stable emitter key yields the reviewed kind.
        assert_eq!(
            lookup(&compiled, ROW.service, ROW.operation, ROW.emitter_key),
            Some(KEY)
        );
        assert_eq!(
            lookup(&compiled, ROW.service, ROW.operation, "a_other"),
            Some(OTHER)
        );
        assert_eq!(
            lookup(&compiled, ROW.service, ROW.operation, "missing"),
            None
        );
        assert_eq!(
            lookup(&compiled, ROW.service, "ListHostedZones", ROW.emitter_key),
            None
        );
        assert_eq!(
            lookup(&compiled, "kms", ROW.operation, ROW.emitter_key),
            None
        );
        Ok(())
    }

    #[test]
    fn additional_emitters_require_valid_keys_known_kinds_and_existing_primary() {
        let known = [PRIMARY, KEY].into_iter().collect();
        let oversized = "x".repeat(129);
        for emitter_key in ["", "has spaces", oversized.as_str()] {
            assert_invalid(
                compile(
                    &[AdditionalEmissionMapping { emitter_key, ..ROW }],
                    &primary(),
                    &known,
                ),
                0,
                "invalid reviewed additional emission",
            );
        }
        assert_invalid(
            compile(&[ROW], &[], &known),
            0,
            "additional emission requires a reviewed primary operation",
        );
        let unknown = AdditionalEmissionMapping { kind: OTHER, ..ROW };
        assert!(matches!(compile(&[unknown], &primary(), &known),
            Err(KindMapError::UnknownKind { service, operation, kind })
                if (service.as_str(), operation.as_str(), kind.as_str()) == (ROW.service, ROW.operation, OTHER)));
        assert_invalid(
            compile(
                &[AdditionalEmissionMapping {
                    kind: PRIMARY,
                    ..ROW
                }],
                &primary(),
                &known,
            ),
            0,
            "additional emission repeats the primary kind",
        );
    }

    fn assert_invalid(
        result: Result<Vec<ReviewedEmission>, KindMapError>,
        expected_line: usize,
        expected_reason: &'static str,
    ) {
        assert!(
            matches!(result, Err(KindMapError::InvalidRow { line, reason })
            if (line, reason) == (expected_line, expected_reason))
        );
    }

    #[test]
    fn additional_emitters_reject_duplicate_keys_and_overlapping_sibling_kinds() {
        let known = [PRIMARY, KEY, OTHER].into_iter().collect();
        assert!(
            matches!(compile(&[ROW, AdditionalEmissionMapping { kind: OTHER, ..ROW }], &primary(), &known),
            Err(KindMapError::Duplicate(key)) if key == "route-53.GetDNSSEC:key_signing_keys")
        );
        assert_invalid(
            compile(
                &[
                    ROW,
                    AdditionalEmissionMapping {
                        emitter_key: "another_key",
                        ..ROW
                    },
                ],
                &primary(),
                &known,
            ),
            1,
            "additional emissions repeat a sibling kind",
        );
    }

    #[test]
    fn eighth_total_emitter_is_allowed_but_ninth_is_rejected() -> TestResult {
        let kinds = [
            "AWS::Test::A",
            "AWS::Test::B",
            "AWS::Test::C",
            "AWS::Test::D",
            "AWS::Test::E",
            "AWS::Test::F",
            "AWS::Test::G",
            "AWS::Test::H",
        ];
        let known = kinds.into_iter().chain([PRIMARY]).collect();
        let source = kinds
            .into_iter()
            .zip(["a", "b", "c", "d", "e", "f", "g", "h"])
            .map(|(kind, emitter_key)| AdditionalEmissionMapping {
                kind,
                emitter_key,
                ..ROW
            })
            .collect::<Vec<_>>();
        let allowed = source.get(..7).ok_or("fixture bounds")?;
        assert_eq!(compile(allowed, &primary(), &known)?.len(), 7);
        assert_invalid(
            compile(&source, &primary(), &known),
            7,
            "operation exceeds eight total resource emissions",
        );
        Ok(())
    }

    fn assert_baseline_no_additional_emissions(source: &MappingSource) -> TestResult {
        use crate::names::Exceptions;
        let baseline = source.compile(&Exceptions::none())?;
        assert_eq!(baseline.additional_emissions(), &[]);
        assert_eq!(
            baseline.additional_emission_kind(ROW.service, ROW.operation, ROW.emitter_key),
            None
        );
        Ok(())
    }

    fn assert_adding_emitter_changes_provenance(
        source: &MappingSource,
        primary_operations: &[OperationMapping],
    ) -> TestResult {
        use crate::names::Exceptions;
        let baseline = source.compile(&Exceptions::none())?;
        let mapped = MappingSource {
            additional_emissions: &[ROW],
            operations: primary_operations,
            ..*source
        }
        .compile(&Exceptions::none())?;
        // When adding an emitter, its review changes provenance and exact lookup is available.
        assert_ne!(baseline.source_sha256(), mapped.source_sha256());
        assert_eq!(
            mapped.additional_emission_kind(ROW.service, ROW.operation, ROW.emitter_key),
            Some(KEY)
        );
        assert_eq!(mapped.additional_emissions().len(), 1);
        assert_eq!(
            mapped.additional_emission_kind(ROW.service, ROW.operation, "missing"),
            None
        );
        for row in [
            AdditionalEmissionMapping {
                emitter_key: "another",
                ..ROW
            },
            AdditionalEmissionMapping {
                operation: "GetOther",
                ..ROW
            },
            AdditionalEmissionMapping {
                service: "route-53-alt",
                ..ROW
            },
            AdditionalEmissionMapping { kind: OTHER, ..ROW },
        ] {
            let changed = MappingSource {
                additional_emissions: &[row],
                operations: primary_operations,
                ..*source
            }
            .compile(&Exceptions::none())?;
            assert_ne!(mapped.source_sha256(), changed.source_sha256());
        }
        Ok(())
    }

    fn assert_declaration_order_does_not_affect_fingerprint(
        source: &MappingSource,
        primary_operations: &[OperationMapping],
    ) -> TestResult {
        use crate::names::Exceptions;
        let other = AdditionalEmissionMapping {
            emitter_key: "a_other",
            kind: OTHER,
            ..ROW
        };
        let forward = MappingSource {
            additional_emissions: &[ROW, other],
            operations: primary_operations,
            ..*source
        }
        .compile(&Exceptions::none())?;
        let reverse = MappingSource {
            additional_emissions: &[other, ROW],
            operations: primary_operations,
            ..*source
        }
        .compile(&Exceptions::none())?;
        assert_eq!(forward, reverse);
        Ok(())
    }

    #[test]
    fn full_mapping_pipeline_binds_each_emitter_field_and_canonicalizes_declarations() -> TestResult
    {
        // Given two primary operations and separately reviewed additional resource kinds.
        let primary_operations = [
            OperationMapping {
                service: ROW.service,
                operation: ROW.operation,
                kind: PRIMARY,
            },
            OperationMapping {
                service: ROW.service,
                operation: "GetOther",
                kind: PRIMARY,
            },
            OperationMapping {
                service: "route-53-alt",
                operation: ROW.operation,
                kind: PRIMARY,
            },
        ];
        let source = MappingSource {
            cfn: &[CfnMapping::canonical(PRIMARY), CfnMapping::canonical(KEY)],
            api_only: &[ApiOnlyMapping {
                kind: OTHER,
                reason: "Distinct additional emission fixture resource",
            }],
            operations: &primary_operations,
            ..MappingSource::default()
        };
        assert_baseline_no_additional_emissions(&source)?;
        assert_adding_emitter_changes_provenance(&source, &primary_operations)?;
        // Then declaration order cannot alter either the map or its reviewed fingerprint.
        assert_declaration_order_does_not_affect_fingerprint(&source, &primary_operations)?;
        Ok(())
    }
}
