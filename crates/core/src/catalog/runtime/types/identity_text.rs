use super::ContextSource;
use crate::{
    catalog::{Shape, ValueConstraints, runtime::RuntimeCatalogError},
    record::{FieldId, ValueRef},
};
use regex::{Regex, RegexBuilder};
use std::{collections::HashMap, sync::Arc};

const MAX_PATTERN_BYTES: usize = 16_384;
const REGEX_SIZE_LIMIT: usize = 1_048_576;

/// Catalog-load-only interning; no runtime row mutates or recompiles a pattern.
#[derive(Default)]
pub(crate) struct IdentityPatternCache<'a> {
    patterns: HashMap<&'a str, Arc<Regex>>,
}

impl<'a> IdentityPatternCache<'a> {
    fn pattern(&mut self, pattern: &'a str) -> Result<Arc<Regex>, RuntimeCatalogError> {
        if let Some(compiled) = self.patterns.get(pattern) {
            return Ok(Arc::clone(compiled));
        }
        if pattern.len() > MAX_PATTERN_BYTES {
            return Err(RuntimeCatalogError::Bounds);
        }
        let compiled = Arc::new(
            RegexBuilder::new(pattern)
                .size_limit(REGEX_SIZE_LIMIT)
                .dfa_size_limit(REGEX_SIZE_LIMIT)
                .build()
                .map_err(|_| RuntimeCatalogError::Capability)?,
        );
        self.patterns.insert(pattern, Arc::clone(&compiled));
        Ok(compiled)
    }
}

/// A catalog-load-compiled text operand, including a modeled string-list element.
pub struct IdentityTextOperand<'a> {
    source: ContextSource,
    path: Box<[FieldId]>,
    shape: Option<Shape<'a>>,
    constraints: CompiledTextConstraints,
}

impl<'a> IdentityTextOperand<'a> {
    pub(crate) fn compile(
        source: ContextSource,
        path: &[FieldId],
        shape: Option<Shape<'a>>,
        member: Option<ValueConstraints<'a>>,
        cache: &mut IdentityPatternCache<'a>,
    ) -> Result<Self, RuntimeCatalogError> {
        if !matches!(source, ContextSource::SelectedItem | ContextSource::Request) {
            return Err(RuntimeCatalogError::Capability);
        }
        let target = match shape {
            Some(shape) if matches!(shape.kind()?, "string" | "enum") => {
                shape.value_constraints()?
            }
            Some(_) => return Err(RuntimeCatalogError::Capability),
            None => ValueConstraints::default(),
        };
        let constraints = CompiledTextConstraints::compile(target, member, cache)?;
        Ok(Self {
            source,
            path: path.into(),
            shape,
            constraints,
        })
    }

    /// Checks retained shape/member constraints without allocating or recompiling.
    ///
    /// # Errors
    /// Rejects malformed retained enum metadata.
    pub fn accepts(&self, text: &str) -> Result<bool, RuntimeCatalogError> {
        if !self.constraints.accepts(text) {
            return Ok(false);
        }
        match self.shape {
            Some(shape) => Ok(shape.accepts_enum_value(ValueRef::Text(text))?),
            None => Ok(true),
        }
    }
}

struct CompiledTextConstraints {
    minimum: Option<usize>,
    maximum: Option<usize>,
    patterns: Box<[Arc<Regex>]>,
}

impl CompiledTextConstraints {
    fn compile<'a>(
        target: ValueConstraints<'a>,
        member: Option<ValueConstraints<'a>>,
        cache: &mut IdentityPatternCache<'a>,
    ) -> Result<Self, RuntimeCatalogError> {
        let mut minimum = None;
        let mut maximum = None;
        let mut patterns: Vec<Arc<Regex>> = Vec::new();
        for constraint in std::iter::once(target).chain(member) {
            if let Some(bound) = constraint.min_length {
                minimum = Some(minimum.map_or(bound, |old: usize| old.max(bound)));
            }
            if let Some(bound) = constraint.max_length {
                maximum = Some(maximum.map_or(bound, |old: usize| old.min(bound)));
            }
            if let Some(pattern) = constraint.pattern
                && !patterns.iter().any(|compiled| compiled.as_str() == pattern)
            {
                patterns.push(cache.pattern(pattern)?);
            }
        }
        if matches!((minimum, maximum), (Some(min), Some(max)) if min > max) {
            return Err(RuntimeCatalogError::Capability);
        }
        Ok(Self {
            minimum,
            maximum,
            patterns: patterns.into_boxed_slice(),
        })
    }

    fn accepts(&self, text: &str) -> bool {
        if self.minimum.is_some() || self.maximum.is_some() {
            let length = text.chars().count();
            if self.minimum.is_some_and(|min| length < min)
                || self.maximum.is_some_and(|max| length > max)
            {
                return false;
            }
        }
        self.patterns.iter().all(|pattern| pattern.is_match(text))
    }
}

/// Validators for potential identity operands; only the chosen branch is checked.
#[derive(Default)]
pub struct IdentityTextPlan<'a> {
    operands: Box<[IdentityTextOperand<'a>]>,
}

impl<'a> IdentityTextPlan<'a> {
    pub(crate) fn new(mut operands: Vec<IdentityTextOperand<'a>>) -> Self {
        operands.sort_unstable_by(|left, right| {
            source_order(left.source)
                .cmp(&source_order(right.source))
                .then_with(|| left.path.cmp(&right.path))
        });
        operands.dedup_by(|left, right| left.source == right.source && left.path == right.path);
        Self {
            operands: operands.into_boxed_slice(),
        }
    }

    /// Checks one operand selected by the existing identity dispatch.
    ///
    /// # Errors
    /// Rejects missing compiled operand metadata or malformed retained enums.
    pub fn accepts(
        &self,
        source: ContextSource,
        path: &[FieldId],
        text: &str,
    ) -> Result<bool, RuntimeCatalogError> {
        self.operand(source, path)?.accepts(text)
    }

    /// Resolves a validator once for a scalar operand or an entire selected list.
    ///
    /// # Errors
    /// Rejects an operand not compiled from the emission's modeled schema.
    pub fn operand(
        &self,
        source: ContextSource,
        path: &[FieldId],
    ) -> Result<&IdentityTextOperand<'a>, RuntimeCatalogError> {
        let index = self
            .operands
            .binary_search_by(|operand| {
                source_order(operand.source)
                    .cmp(&source_order(source))
                    .then_with(|| operand.path.as_ref().cmp(path))
            })
            .map_err(|_| RuntimeCatalogError::Capability)?;
        self.operands
            .get(index)
            .ok_or(RuntimeCatalogError::Capability)
    }
}

fn source_order(source: ContextSource) -> u8 {
    match source {
        ContextSource::SelectedItem => 0,
        ContextSource::Request => 1,
        ContextSource::Response => 2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    type TestResult = Result<(), Box<dyn std::error::Error>>;

    #[test]
    fn given_unicode_text_and_model_bounds_when_validated_then_length_counts_characters()
    -> TestResult {
        let path = [FieldId::new(1)?];
        let mut cache = IdentityPatternCache::default();
        let operand = IdentityTextOperand::compile(
            ContextSource::SelectedItem,
            &path,
            None,
            Some(ValueConstraints {
                min_length: Some(2),
                max_length: Some(2),
                pattern: Some("^[éβ]+$"),
            }),
            &mut cache,
        )?;
        let plan = IdentityTextPlan::new(vec![operand]);
        assert!(plan.accepts(ContextSource::SelectedItem, &path, "éβ")?);
        assert!(!plan.accepts(ContextSource::SelectedItem, &path, "é")?);
        assert!(!plan.accepts(ContextSource::SelectedItem, &path, "éβé")?);
        assert!(!plan.accepts(ContextSource::SelectedItem, &path, "ab")?);
        assert!(matches!(
            plan.accepts(ContextSource::Request, &path, "éβ"),
            Err(RuntimeCatalogError::Capability)
        ));
        Ok(())
    }

    #[test]
    fn given_repeated_model_pattern_when_compiled_then_operands_share_one_bounded_regex()
    -> TestResult {
        let oversized = "x".repeat(MAX_PATTERN_BYTES + 1);
        let mut cache = IdentityPatternCache::default();
        let first = cache.pattern("^arn:aws:wisdom:[^:]+:[0-9]{12}:.*$")?;
        let second = cache.pattern("^arn:aws:wisdom:[^:]+:[0-9]{12}:.*$")?;
        assert!(Arc::ptr_eq(&first, &second));
        assert_eq!(cache.patterns.len(), 1);
        assert!(matches!(
            cache.pattern("["),
            Err(RuntimeCatalogError::Capability)
        ));
        assert!(matches!(
            cache.pattern(&oversized),
            Err(RuntimeCatalogError::Bounds)
        ));
        Ok(())
    }

    #[test]
    fn given_shape_and_member_constraints_when_compiled_then_both_patterns_and_tighter_member_bounds_apply()
    -> TestResult {
        // Given independent modeled shape and member constraints.
        let target = ValueConstraints {
            min_length: Some(2),
            max_length: Some(6),
            pattern: Some("^a"),
        };
        let member = ValueConstraints {
            min_length: Some(4),
            max_length: Some(5),
            pattern: Some("z$"),
        };
        let mut cache = IdentityPatternCache::default();
        // When the immutable intersection is compiled once.
        let rules = CompiledTextConstraints::compile(target, Some(member), &mut cache)?;
        // Then both inclusive bounds and both independent patterns apply.
        for (text, expected) in [
            ("abcz", true),
            ("abcdz", true),
            ("abz", false),
            ("abcdez", false),
            ("zzzz", false),
            ("aaaa", false),
            ("", false),
        ] {
            assert_eq!(rules.accepts(text), expected, "{text:?}");
        }
        assert_eq!(cache.patterns.len(), 2);
        Ok(())
    }

    #[test]
    fn given_looser_member_bounds_when_compiled_then_the_shape_bounds_are_not_overwritten()
    -> TestResult {
        let target = ValueConstraints {
            min_length: Some(4),
            max_length: Some(5),
            pattern: None,
        };
        let member = ValueConstraints {
            min_length: Some(2),
            max_length: Some(6),
            pattern: None,
        };
        let mut cache = IdentityPatternCache::default();
        let rules = CompiledTextConstraints::compile(target, Some(member), &mut cache)?;
        for (text, expected) in [
            ("aaa", false),
            ("aaaa", true),
            ("aaaaa", true),
            ("aaaaaa", false),
        ] {
            assert_eq!(rules.accepts(text), expected, "{text:?}");
        }
        Ok(())
    }

    #[test]
    fn given_disjoint_shape_and_member_lengths_when_compiled_then_catalog_load_rejects_the_recipe()
    {
        let target = ValueConstraints {
            min_length: Some(2),
            max_length: Some(3),
            pattern: None,
        };
        let member = ValueConstraints {
            min_length: Some(4),
            max_length: Some(5),
            pattern: None,
        };
        let mut cache = IdentityPatternCache::default();
        assert!(matches!(
            CompiledTextConstraints::compile(target, Some(member), &mut cache),
            Err(RuntimeCatalogError::Capability)
        ));
    }

    #[test]
    fn given_identical_shape_and_member_pattern_when_compiled_then_the_intersection_reuses_one_matcher()
    -> TestResult {
        let target = ValueConstraints {
            pattern: Some("^arn:"),
            ..ValueConstraints::default()
        };
        let mut cache = IdentityPatternCache::default();
        let rules = CompiledTextConstraints::compile(target, Some(target), &mut cache)?;
        assert_eq!(rules.patterns.len(), 1);
        assert_eq!(cache.patterns.len(), 1);
        assert_eq!(rules.accepts("arn:value"), true);
        assert_eq!(rules.accepts("value"), false);
        Ok(())
    }

    proptest::proptest! {
        #![proptest_config(proptest::test_runner::Config::with_cases(1024))]

        #[test]
        fn given_two_valid_length_ranges_when_intersected_then_acceptance_obeys_both_original_ranges(
            shape_min in 0usize..32,
            shape_width in 0usize..32,
            member_min in 0usize..32,
            member_width in 0usize..32,
            length in 0usize..64,
        ) {
            let shape_max = shape_min.saturating_add(shape_width);
            let member_max = member_min.saturating_add(member_width);
            let target = ValueConstraints { min_length: Some(shape_min), max_length: Some(shape_max), pattern: None };
            let member = ValueConstraints { min_length: Some(member_min), max_length: Some(member_max), pattern: None };
            let mut cache = IdentityPatternCache::default();
            let disjoint = shape_max < member_min || member_max < shape_min;
            match CompiledTextConstraints::compile(target, Some(member), &mut cache) {
                Err(error) => {
                    proptest::prop_assert!(matches!(error, RuntimeCatalogError::Capability));
                    proptest::prop_assert!(disjoint);
                },
                Ok(rules) => {
                    proptest::prop_assert!(!disjoint);
                    let expected = (shape_min..=shape_max).contains(&length) && (member_min..=member_max).contains(&length);
                    proptest::prop_assert_eq!(rules.accepts(&"x".repeat(length)), expected);
                },
            }
        }
    }

    #[test]
    fn given_modeled_namespace_element_constraints_when_a_list_validator_is_resolved_then_each_component_obeys_the_source()
    -> TestResult {
        // Pinned S3Tables NamespaceName: length 1..=255 and ^[0-9a-z_]*$.
        let path = [FieldId::new(1)?];
        let mut cache = IdentityPatternCache::default();
        let operand = IdentityTextOperand::compile(
            ContextSource::SelectedItem,
            &path,
            None,
            Some(ValueConstraints {
                min_length: Some(1),
                max_length: Some(255),
                pattern: Some("^[0-9a-z_]*$"),
            }),
            &mut cache,
        )?;
        let plan = IdentityTextPlan::new(vec![operand]);
        let element = plan.operand(ContextSource::SelectedItem, &path)?;
        for (component, expected) in [
            ("valid_name", true),
            ("0", true),
            ("", false),
            ("a:b", false),
            ("Upper", false),
            ("a-b", false),
            ("a b", false),
            ("é", false),
        ] {
            assert_eq!(element.accepts(component)?, expected, "{component:?}");
        }
        assert_eq!(element.accepts(&"a".repeat(255))?, true);
        assert_eq!(element.accepts(&"a".repeat(256))?, false);
        Ok(())
    }
}
