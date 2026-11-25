//! Coverage denominators computed from committed catalog artifacts.
//!
//! Each count is read from a committed artifact and checked against that
//! artifact's own partition totals. A value the catalog does not record is
//! rendered as `not_recorded`; it is never inferred or defaulted to zero.
//! Stages form an ordered funnel: every recorded stage is no larger than the
//! recorded stage before it.

use std::{
    collections::BTreeMap,
    fmt::{self, Write as _},
    fs, io,
    path::{Path, PathBuf},
};

use serde_json::Value;
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::report::{self, ReportError, ReportMode};

pub const REPORT_PATH: &str = "catalog/coverage-denominator.report";

const MODELS: &str = "catalog/aws-models.report";
const PLANS: &str = "catalog/aws-plans.report";
const CLASSES: &str = "catalog/aws-classes.report";
const OVERLAY: &str = "catalog/aws-overlay.report";
const OPERATIONS: &str = "catalog/aws-operations.toml";
const CAPABILITIES: &str = "catalog/cloudcontrol-capabilities.toml";
const RESOURCES: &str = "catalog/cloudcontrol-resources.toml";

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Report(#[from] ReportError),

    #[error("{file}: {source}")]
    Io { file: PathBuf, source: io::Error },

    #[error("{file}: {message}")]
    Invalid { file: &'static str, message: String },

    #[error("funnel {funnel}: stage {stage} exceeds the stage before it")]
    NotMonotone { funnel: &'static str, stage: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Count {
    Recorded(u64),
    NotRecorded,
}

impl fmt::Display for Count {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Recorded(count) => write!(f, "{count}"),
            Self::NotRecorded => f.write_str("not_recorded"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stage {
    pub name: String,
    pub count: Count,
}

impl Stage {
    fn recorded(name: impl Into<String>, count: u64) -> Self {
        Self {
            name: name.into(),
            count: Count::Recorded(count),
        }
    }

    fn not_recorded(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            count: Count::NotRecorded,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Funnel {
    pub name: &'static str,
    /// Ordered funnel; recorded counts must not increase down the list.
    pub stages: Vec<Stage>,
    /// Catalog partition of the population the funnel starts from.
    pub excluded: Vec<Stage>,
    /// Recorded counts outside the funnel order.
    pub notes: Vec<Stage>,
    /// Free-text basis for a recorded value the catalog states indirectly.
    pub basis: Vec<String>,
}

impl Funnel {
    /// Name of the first stage whose recorded count exceeds the stage before it.
    #[must_use]
    pub fn first_violation(&self) -> Option<&str> {
        let mut previous = u64::MAX;
        for stage in &self.stages {
            if let Count::Recorded(count) = stage.count {
                if count > previous {
                    return Some(&stage.name);
                }
                previous = count;
            }
        }
        None
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    pub digest: String,
    pub inputs: Vec<(&'static str, String)>,
    pub funnels: Vec<Funnel>,
}

impl Report {
    /// Renders the committed text form. Contains no timestamps or host paths.
    pub fn render(&self) -> Result<String, ReportError> {
        let mut text = String::new();
        writeln!(
            text,
            "# coverage denominator; counts are read from committed catalog artifacts"
        )?;
        writeln!(text, "catalog-digest sha256:{}", self.digest)?;
        for (path, sha256) in &self.inputs {
            writeln!(text, "input {path} sha256:{sha256}")?;
        }
        for funnel in &self.funnels {
            writeln!(text, "funnel {}", funnel.name)?;
            for stage in &funnel.stages {
                writeln!(text, "stage {} {}", stage.name, stage.count)?;
            }
            for stage in &funnel.excluded {
                writeln!(text, "excluded {} {}", stage.name, stage.count)?;
            }
            for stage in &funnel.notes {
                writeln!(text, "note {} {}", stage.name, stage.count)?;
            }
            for basis in &funnel.basis {
                writeln!(text, "basis {basis}")?;
            }
        }
        Ok(text)
    }
}

pub fn run(root: &Path, mode: ReportMode) -> Result<(), Error> {
    let text = build(root)?.render()?;
    report::publish(&root.join(REPORT_PATH), &text, mode)?;
    Ok(())
}

pub fn build(root: &Path) -> Result<Report, Error> {
    let models = Input::load(root, MODELS)?;
    let plans = Input::load(root, PLANS)?;
    let classes = Input::load(root, CLASSES)?;
    let overlay = Input::load(root, OVERLAY)?;
    let operations = Input::load(root, OPERATIONS)?;
    let capabilities = Input::load(root, CAPABILITIES)?;
    let resources = Input::load(root, RESOURCES)?;

    let inputs = [
        &models,
        &plans,
        &classes,
        &overlay,
        &operations,
        &capabilities,
        &resources,
    ];
    let manifest: String = inputs
        .iter()
        .map(|input| format!("{} {}\n", input.path, input.sha256))
        .collect();
    let digest = hex(&Sha256::digest(manifest.as_bytes()));

    let funnels = vec![
        operation_funnel(&models, &classes, &plans, &overlay)?,
        native_funnel(&operations)?,
        cloud_control_funnel(&capabilities, &resources)?,
        resource_kind_funnel(&models, &plans)?,
    ];
    for funnel in &funnels {
        if let Some(stage) = funnel.first_violation() {
            return Err(Error::NotMonotone {
                funnel: funnel.name,
                stage: stage.to_owned(),
            });
        }
    }

    Ok(Report {
        digest,
        inputs: inputs
            .iter()
            .map(|input| (input.path, input.sha256.clone()))
            .collect(),
        funnels,
    })
}

struct Input {
    path: &'static str,
    text: String,
    sha256: String,
}

impl Input {
    fn load(root: &Path, path: &'static str) -> Result<Self, Error> {
        let file = root.join(path);
        let bytes = fs::read(&file).map_err(|source| Error::Io {
            file: file.clone(),
            source,
        })?;
        let sha256 = hex(&Sha256::digest(&bytes));
        let text = String::from_utf8(bytes).map_err(|error| Error::Io {
            file,
            source: io::Error::new(io::ErrorKind::InvalidData, error),
        })?;
        Ok(Self { path, text, sha256 })
    }

    fn invalid(&self, message: impl Into<String>) -> Error {
        Error::Invalid {
            file: self.path,
            message: message.into(),
        }
    }

    fn json(&self) -> Result<Value, Error> {
        toml::from_str(&self.text).map_err(|source| self.invalid(source.to_string()))
    }

    fn toml(&self) -> Result<Value, Error> {
        toml::from_str(&self.text).map_err(|source| self.invalid(source.to_string()))
    }

    fn expect_equal(&self, what: &str, actual: u64, expected: u64) -> Result<(), Error> {
        if actual == expected {
            Ok(())
        } else {
            Err(self.invalid(format!("{what}: {actual} != {expected}")))
        }
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().fold(
        String::with_capacity(bytes.len().saturating_mul(2)),
        |mut text, byte| {
            let _ = write!(text, "{byte:02x}");
            text
        },
    )
}

fn sum(values: &[u64]) -> u64 {
    values
        .iter()
        .fold(0, |total, value| total.saturating_add(*value))
}

fn count_u64(count: usize) -> u64 {
    u64::try_from(count).unwrap_or(u64::MAX)
}

/// Categories the catalog never records directly; every funnel marks them explicitly.
fn unrecorded_categories() -> Vec<Stage> {
    vec![
        Stage::not_recorded("unsupported"),
        Stage::not_recorded("inaccessible"),
        Stage::not_recorded("skipped"),
        Stage::not_recorded("unobserved"),
    ]
}

/// Parses `key value` lines; prose and table rows have other shapes and are skipped.
fn key_values(input: &Input) -> Result<BTreeMap<String, u64>, Error> {
    let mut values = BTreeMap::new();
    for line in input.text.lines() {
        let fields: Vec<&str> = line.split_whitespace().collect();
        let [key, value] = fields.as_slice() else {
            continue;
        };
        let Ok(number) = value.parse::<u64>() else {
            continue;
        };
        if values.insert((*key).to_owned(), number).is_some() {
            return Err(input.invalid(format!("duplicate key {key}")));
        }
    }
    Ok(values)
}

fn key_value(input: &Input, values: &BTreeMap<String, u64>, key: &str) -> Result<u64, Error> {
    values
        .get(key)
        .copied()
        .ok_or_else(|| input.invalid(format!("missing {key}")))
}

struct ClassCounts {
    operations: u64,
    enumerate: u64,
    expand: u64,
    denied: u64,
    unclassified: u64,
}

fn class_counts(input: &Input) -> Result<ClassCounts, Error> {
    let values = key_values(input)?;
    let counts = ClassCounts {
        operations: key_value(input, &values, "operations")?,
        enumerate: key_value(input, &values, "enumerate")?,
        expand: key_value(input, &values, "expand")?,
        denied: key_value(input, &values, "denied")?,
        unclassified: key_value(input, &values, "unclassified")?,
    };
    input.expect_equal(
        "class partition",
        sum(&[
            counts.enumerate,
            counts.expand,
            counts.denied,
            counts.unclassified,
        ]),
        counts.operations,
    )?;
    Ok(counts)
}

struct OverlayCounts {
    enumerate: u64,
    expand: u64,
    denied_overlay: u64,
    denied_deny_list: u64,
    deferred: u64,
}

/// Counts one row per operation and checks each category against the overlay's own totals line.
fn overlay_counts(input: &Input, known: u64) -> Result<OverlayCounts, Error> {
    let lines = parse_overlay_lines(input)?;
    input.expect_equal("overlay rows", lines.rows, known)?;
    validate_overlay_totals(input, &lines.counts, &lines.totals)
}

struct OverlayLines<'a> {
    counts: BTreeMap<&'a str, u64>,
    totals: BTreeMap<&'a str, u64>,
    rows: u64,
}

fn parse_overlay_lines(input: &Input) -> Result<OverlayLines<'_>, Error> {
    let mut counts: BTreeMap<&str, u64> = BTreeMap::new();
    let mut totals: BTreeMap<&str, u64> = BTreeMap::new();
    let mut rows = 0_u64;
    for line in input.text.lines() {
        let columns: Vec<&str> = line.split('\t').collect();
        match columns.as_slice() {
            ["totals", fields @ ..] => {
                for (key, value) in fields.iter().filter_map(|field| field.split_once('=')) {
                    if let Ok(number) = value.parse::<u64>() {
                        totals.insert(key, number);
                    }
                }
            }
            ["operation", _service, _name, status, detail] => {
                rows = rows.saturating_add(1);
                let class = detail.split(',').next().unwrap_or_default();
                let key = match (*status, class) {
                    ("enumerate", "class=overlay") => "enumerate",
                    ("expand", "class=overlay") => "expand",
                    ("denied", "class=overlay") => "denied_overlay",
                    ("denied", "class=deny-list") => "denied_deny_list",
                    ("deferred", "class=overlay") => "deferred",
                    _ => {
                        return Err(
                            input.invalid(format!("unknown overlay row status {status} {class}"))
                        );
                    }
                };
                let entry = counts.entry(key).or_insert(0);
                *entry = entry.saturating_add(1);
            }
            _ => {}
        }
    }
    Ok(OverlayLines {
        counts,
        totals,
        rows,
    })
}

fn validate_overlay_totals(
    input: &Input,
    counts: &BTreeMap<&str, u64>,
    totals: &BTreeMap<&str, u64>,
) -> Result<OverlayCounts, Error> {
    let count = |key: &str| counts.get(key).copied().unwrap_or(0);
    let overlay = OverlayCounts {
        enumerate: count("enumerate"),
        expand: count("expand"),
        denied_overlay: count("denied_overlay"),
        denied_deny_list: count("denied_deny_list"),
        deferred: count("deferred"),
    };
    let total = |key: &str| {
        totals
            .get(key)
            .copied()
            .ok_or_else(|| input.invalid(format!("totals missing {key}")))
    };
    input.expect_equal(
        "totals callable_enumerate",
        total("callable_enumerate")?,
        overlay.enumerate,
    )?;
    input.expect_equal(
        "totals callable_expand",
        total("callable_expand")?,
        overlay.expand,
    )?;
    input.expect_equal(
        "totals denied_by_overlay",
        total("denied_by_overlay")?,
        overlay.denied_overlay,
    )?;
    input.expect_equal("totals deferred", total("deferred")?, overlay.deferred)?;
    Ok(overlay)
}

fn operation_funnel(
    models: &Input,
    classes: &Input,
    plans: &Input,
    overlay: &Input,
) -> Result<Funnel, Error> {
    let known = key_value(models, &key_values(models)?, "operations")?;
    let classes = class_counts(classes)?;
    models.expect_equal("classes operations", classes.operations, known)?;

    let plan_values = key_values(plans)?;
    plans.expect_equal(
        "plans operations",
        key_value(plans, &plan_values, "operations")?,
        known,
    )?;
    plans.expect_equal(
        "plans enumerate",
        key_value(plans, &plan_values, "enumerate")?,
        classes.enumerate,
    )?;
    plans.expect_equal(
        "plans expand",
        key_value(plans, &plan_values, "expand")?,
        classes.expand,
    )?;

    let overlay = overlay_counts(overlay, known)?;
    let callable = overlay.enumerate.saturating_add(overlay.expand);
    let excluded = sum(&[
        overlay.denied_overlay,
        overlay.denied_deny_list,
        overlay.deferred,
    ]);
    overlay_partition(callable, excluded, known)?;

    Ok(build_operation_funnel(known, classes, callable, &overlay))
}

fn build_operation_funnel(
    known: u64,
    classes: ClassCounts,
    callable: u64,
    overlay: &OverlayCounts,
) -> Funnel {
    let mut excluded_stages = vec![
        Stage::recorded("denied_by_overlay", overlay.denied_overlay),
        Stage::recorded("denied_by_deny_list", overlay.denied_deny_list),
        Stage::recorded("deferred_by_overlay", overlay.deferred),
    ];
    excluded_stages.extend(unrecorded_categories());
    Funnel {
        name: "operations",
        stages: vec![
            Stage::recorded("known_operations", known),
            Stage::recorded(
                "candidate_operations",
                classes.enumerate.saturating_add(classes.expand),
            ),
            Stage::recorded("callable_overlay", callable),
            Stage::not_recorded("fixture_tested"),
            Stage::not_recorded("emulator_tested"),
            Stage::not_recorded("real_aws_tested"),
        ],
        excluded: excluded_stages,
        notes: vec![
            Stage::recorded("class_denied", classes.denied),
            Stage::recorded("class_unclassified", classes.unclassified),
        ],
        basis: Vec::new(),
    }
}

fn overlay_partition(callable: u64, excluded: u64, known: u64) -> Result<(), Error> {
    if callable.saturating_add(excluded) == known {
        Ok(())
    } else {
        Err(Error::Invalid {
            file: OVERLAY,
            message: format!("callable {callable} + excluded {excluded} != {known}"),
        })
    }
}

fn native_funnel(input: &Input) -> Result<Funnel, Error> {
    let document = input.toml()?;
    let known = json_number(input, &document, "/counts/explicit_unique_operations")?;
    let operations = document
        .get("operations")
        .and_then(Value::as_array)
        .ok_or_else(|| input.invalid("operations is not an array"))?;
    input.expect_equal("operation array length", count_u64(operations.len()), known)?;

    let status = |name: &str| {
        count_where(operations, |operation| {
            operation
                .pointer("/emulator_coverage/status")
                .and_then(Value::as_str)
                == Some(name)
        })
    };
    let emulator = status("collector_success");
    let not_implemented = status("emulator_not_implemented");
    let decode_failure = status("sdk_decode_failure");
    let probe = status("explicit_sdk_probe_success");
    input.expect_equal(
        "native status partition",
        sum(&[emulator, not_implemented, decode_failure, probe]),
        known,
    )?;

    let mut excluded = vec![
        Stage::recorded("emulator_not_implemented", not_implemented),
        Stage::recorded("sdk_decode_failure", decode_failure),
        Stage::recorded("explicit_sdk_probe_success", probe),
    ];
    excluded.extend(unrecorded_categories());
    Ok(Funnel {
        name: "native_sdk_operations",
        stages: vec![
            Stage::recorded("known_operations", known),
            Stage::not_recorded("fixture_tested"),
            Stage::recorded("emulator_tested", emulator),
            Stage::not_recorded("real_aws_tested"),
        ],
        excluded,
        notes: Vec::new(),
        basis: Vec::new(),
    })
}

fn cloud_control_funnel(capabilities: &Input, resources: &Input) -> Result<Funnel, Error> {
    let document = capabilities.json()?;
    let known = json_number(capabilities, &document, "/summary/schema_count")?;
    let types = document
        .get("resource_types")
        .and_then(Value::as_array)
        .ok_or_else(|| capabilities.invalid("resource_types is not an array"))?;
    capabilities.expect_equal("resource type count", count_u64(types.len()), known)?;

    let (read_and_list, selected, excluded) = validate_type_counts(capabilities, &document, types)?;
    let (excluded_stages, excluded_total) =
        process_exclusion_reasons(capabilities, &document, types)?;
    capabilities.expect_equal("exclusion partition", excluded_total, excluded)?;

    let (emulator_read, emulator_list_error) =
        validate_emulator_outcomes(capabilities, &document, types, selected)?;

    let (real_aws, basis) = real_aws_stage(resources, selected)?;
    let mut all_excluded = excluded_stages;
    all_excluded.extend(unrecorded_categories());
    Ok(Funnel {
        name: "cloud_control_types",
        stages: vec![
            Stage::recorded("known_types", known),
            Stage::recorded("read_and_list", read_and_list),
            Stage::recorded("selected", selected),
            Stage::not_recorded("fixture_tested"),
            Stage::recorded("emulator_tested", emulator_read),
            real_aws,
        ],
        excluded: all_excluded,
        notes: vec![Stage::recorded(
            "selected_emulator_list_error",
            emulator_list_error,
        )],
        basis,
    })
}

fn real_aws_stage(resources: &Input, selected: u64) -> Result<(Stage, Vec<String>), Error> {
    let resource_document = resources.json()?;
    let live_validated = resource_document
        .pointer("/evidence/live_validated")
        .and_then(Value::as_bool)
        .ok_or_else(|| resources.invalid("evidence.live_validated is not a boolean"))?;
    let resource_types = resource_document
        .get("resource_types")
        .and_then(Value::as_array)
        .ok_or_else(|| resources.invalid("resource_types is not an array"))?;
    resources.expect_equal(
        "resource document selected count",
        count_u64(resource_types.len()),
        selected,
    )?;
    // The resource file records that no live validation ran, so the real-AWS count is
    // recorded as zero with that basis. Any other value means the count is unknown.
    if live_validated {
        Ok((Stage::not_recorded("real_aws_tested"), Vec::new()))
    } else {
        Ok((
            Stage::recorded("real_aws_tested", 0),
            vec!["real_aws_tested=0 from evidence.live_validated=false".to_owned()],
        ))
    }
}

fn validate_type_counts(
    capabilities: &Input,
    document: &Value,
    types: &[Value],
) -> Result<(u64, u64, u64), Error> {
    let read_and_list = count_where(types, |t| {
        flag(t, "read_supported") && flag(t, "list_supported")
    });
    let selected = count_where(types, |t| flag(t, "selected"));
    let excluded = count_where(types, |t| !flag(t, "selected"));
    capabilities.expect_equal(
        "read and list count",
        read_and_list,
        json_number(capabilities, document, "/summary/read_and_list_count")?,
    )?;
    capabilities.expect_equal(
        "selected count",
        selected,
        json_number(capabilities, document, "/summary/selected_count")?,
    )?;
    capabilities.expect_equal(
        "excluded count",
        excluded,
        json_number(capabilities, document, "/summary/excluded_count")?,
    )?;
    Ok((read_and_list, selected, excluded))
}

fn process_exclusion_reasons(
    capabilities: &Input,
    document: &Value,
    types: &[Value],
) -> Result<(Vec<Stage>, u64), Error> {
    let reasons = document
        .pointer("/summary/exclusions")
        .and_then(Value::as_object)
        .ok_or_else(|| capabilities.invalid("summary.exclusions is not an object"))?;
    let mut excluded_stages = Vec::new();
    let mut excluded_total = 0_u64;
    for (reason, claim) in reasons {
        let claim = claim
            .as_u64()
            .ok_or_else(|| capabilities.invalid(format!("exclusion {reason} is not a count")))?;
        let counted = count_where(types, |t| {
            !flag(t, "selected") && t.get("exclusion").and_then(Value::as_str) == Some(reason)
        });
        capabilities.expect_equal(&format!("exclusion {reason}"), counted, claim)?;
        excluded_total = excluded_total.saturating_add(counted);
        excluded_stages.push(Stage::recorded(reason.as_str(), counted));
    }
    Ok((excluded_stages, excluded_total))
}

fn validate_emulator_outcomes(
    capabilities: &Input,
    document: &Value,
    types: &[Value],
    selected: u64,
) -> Result<(u64, u64), Error> {
    let emulator_status = |status: &str| {
        count_where(types, |t| {
            flag(t, "selected")
                && t.pointer("/emulator_verification/status")
                    .and_then(Value::as_str)
                    == Some(status)
        })
    };
    let emulator_read = emulator_status("read_observed");
    let emulator_list_error = emulator_status("list_error");
    capabilities.expect_equal(
        "emulator read_observed",
        emulator_read,
        json_number(capabilities, document, "/summary/emulator_read_observed")?,
    )?;
    capabilities.expect_equal(
        "emulator list_error",
        emulator_list_error,
        json_number(
            capabilities,
            document,
            "/summary/emulator_outcomes/list_error",
        )?,
    )?;
    capabilities.expect_equal(
        "selected emulator outcomes",
        emulator_read.saturating_add(emulator_list_error),
        selected,
    )?;
    Ok((emulator_read, emulator_list_error))
}

fn resource_kind_funnel(models: &Input, plans: &Input) -> Result<Funnel, Error> {
    let known = key_value(models, &key_values(models)?, "resources")?;
    let plan_values = key_values(plans)?;
    plans.expect_equal(
        "plans resources",
        key_value(plans, &plan_values, "resources")?,
        known,
    )?;
    let read = key_value(plans, &plan_values, "resources_with_read")?;
    let list_and_read = key_value(plans, &plan_values, "resources_with_list_and_read")?;
    let list = key_value(plans, &plan_values, "resources_with_list")?;
    let arn = key_value(plans, &plan_values, "resources_with_arn")?;
    Ok(Funnel {
        name: "resource_kinds",
        stages: vec![
            Stage::recorded("known_resource_kinds", known),
            Stage::recorded("read_modeled", read),
            Stage::recorded("list_and_read_modeled", list_and_read),
            Stage::not_recorded("fixture_tested"),
            Stage::not_recorded("emulator_tested"),
            Stage::not_recorded("real_aws_tested"),
        ],
        excluded: unrecorded_categories(),
        notes: vec![
            Stage::recorded("list_modeled", list),
            Stage::recorded("arn_modeled", arn),
        ],
        basis: Vec::new(),
    })
}

fn flag(t: &Value, key: &str) -> bool {
    t.get(key).and_then(Value::as_bool).unwrap_or(false)
}

fn count_where(values: &[Value], predicate: impl Fn(&Value) -> bool) -> u64 {
    count_u64(values.iter().filter(|value| predicate(value)).count())
}

fn json_number(input: &Input, document: &Value, pointer: &str) -> Result<u64, Error> {
    document
        .pointer(pointer)
        .and_then(Value::as_u64)
        .ok_or_else(|| input.invalid(format!("{pointer} is not an unsigned integer")))
}
