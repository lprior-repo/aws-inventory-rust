# Cloud Control coverage evidence

Generated from the pinned capability matrix. Schema support, reviewed collector selection,
relationship rules, and observed emulator reads are separate evidence dimensions.

| Measure | Count |
| --- | ---: |
| Retained provider schemas | 1,881 |
| Schemas declaring read and list handlers | 1,701 |
| Selected executable regional descriptors | 397 |
| Declared relationship rules | 945 |
| Selected types with relationship rules | 244 |
| Types with observed Rust ListResources requests | 397 |
| Types with matching Rust GetResource and persisted detailed observations | 386 |

## Exclusions

| Reason | Count |
| --- | ---: |
| documented_parent_model_required | 37 |
| missing_read_or_list_handler | 180 |
| schema_parent_model_required | 373 |
| scope_unreviewed | 894 |

## Executed evidence

- Summary: `artifacts/emulator-v05/summary.json`.
- Summary SHA-256: `c13e384c51e1318e79d46ffa63be189afa8ad3630def523cd45a23718289fba8`.
- Cloud Control scan receipt: `artifacts/emulator-v05/cloud-control.json`.
- Database: `artifacts/emulator-v05/inventory.sqlite`.
- Python fixture creation is excluded from operation coverage. A successful HTTP response
  alone does not establish a successful typed, stored read.

| Observed per-type outcome | Count |
| --- | ---: |
| list_error | 11 |
| read_observed | 386 |

Selected types without an observed detailed read:

- `AWS::Bedrock::Agent`.
- `AWS::Bedrock::ApplicationInferenceProfile`.
- `AWS::Bedrock::AutomatedReasoningPolicy`.
- `AWS::Bedrock::DataAutomationProject`.
- `AWS::Bedrock::Flow`.
- `AWS::Bedrock::Guardrail`.
- `AWS::Bedrock::KnowledgeBase`.
- `AWS::Bedrock::Prompt`.
- `AWS::S3Express::DirectoryBucket`.
- `AWS::S3Tables::TableBucket`.
- `AWS::Scheduler::ScheduleGroup`.

## Full matrix and interpretation

- [All provider capability entries](../catalog/cloudcontrol-capabilities.toml).
- [Executable descriptors and property mappings](../catalog/cloudcontrol-resources.toml).
- [Collection and relationship design](CLOUDCONTROL_COVERAGE.md).

The pinned emulator uses a generic property store for most types. These observations
exercise SDK decoding, collection, normalization and storage; they do not certify the
production AWS provider's behavior, IAM permissions, regional availability or quotas.
