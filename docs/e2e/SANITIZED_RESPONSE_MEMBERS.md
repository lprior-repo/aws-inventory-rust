# Sanitized response members

## Contract

`OperationOverlay.exclude_response_members` is a reviewed, per-operation list of
at most eight output-root Smithy member paths. Lists are transparent when walking
these paths: `InstanceProfiles.Roles.AssumeRolePolicyDocument` names a member in
every embedded role, not a particular array index. Paths must resolve against the
pinned output model. Arbitrary document/map keys are not modeled member paths.
Exclude their containing member instead.

The decoder omits the **terminal member name everywhere in that operation's
response**. This conservative name-global rule is intentional: output-root paths
prove that a reviewed member exists, but do not restrict omission to one nesting
site. Review all uses of that name before enabling a policy. Never exclude an
identity, pagination cursor, discriminator, binding source, or relationship target
needed by the remaining metadata contract. This is omission, not masking: the
member is absent, not null, hashed, replaced, or copied into another field.

Compilation writes validated numeric paths into callable-plan field 24. The
runtime consumer independently validates the output shape, member IDs, path and
collection bounds, rejects traversal through arbitrary maps, and constructs a
fixed eight-slot `MemberExclusions` value. Empty policies omit field 24. Existing
catalogs remain readable with empty exclusions; older readers reject a catalog
containing the new field rather than silently retaining secrets. Recompile and
deploy the catalog together with its collector.

The collector passes this policy to the public response decoder. JSON, CBOR,
AWS Query, EC2 Query and REST XML skip excluded values before constructing their
modeled record values. HTTP payload/header bindings and SDK-default insertion
honor the same exclusions. Existing depth/node/body bounds still apply to skipped
content. Sensitive response bytes necessarily exist transiently in the bounded
HTTP input buffer; this mechanism does not promise zeroization, prevent the
remote service returning them, or protect an independently enabled raw-HTTP
capture. It prevents their retention as decoded inventory members. No raw-response
logging is added.

`exclude_from_hash`, hash map-key exclusions, queryability and resource projections
are not substitutes: their input record has already been decoded, and hash
exclusion does not remove stored data. Existing IAM trust-summary projections
remain unchanged; the new profile plans omit embedded trust policies outright.

## Enabled operations

| Service | Operations | Omitted output-root members |
|---|---|---|
| IAM | `ListInstanceProfiles`, `ListInstanceProfilesForRole` | `InstanceProfiles.Roles.AssumeRolePolicyDocument` |
| IAM | `GetInstanceProfile` | `InstanceProfile.Roles.AssumeRolePolicyDocument` |
| Lambda | `ListFunctions` | `Functions.Environment`, `Functions.ImageConfigResponse` |
| Lambda | `GetFunctionConfiguration` | `Environment`, `ImageConfigResponse` |
| SNS | `ListPlatformApplications` | `PlatformApplications.Attributes` |

Six directly sanitized operations plus dependent Lambda `ListAliases` are newly
callable: **seven operations**. Environment maps and image command/configuration
content are dropped as whole modeled members, not scanned for selected secret
keys. SNS keeps the application ARN and drops its entire open attributes map;
there is no assumption that a credential key is absent or that unknown keys are
safe. Its kind is explicitly `AWS::ApiOnly::SNSPlatformApplication`.

IAM profile discovery and detail share canonical `AWS::IAM::InstanceProfile` ARN
identity and account scope. Profile names bind detail calls. Role-scoped expansion
binds `RoleName` from complete `ListRoles` discovery and emits `HAS_ROLE` edges
using request role-name context resolved against `AWS::IAM::Role.RoleName`.
Nested `Roles[].Arn` is retained, but is not falsely represented as an executable
scalar relation path. A selector containing only `ListInstanceProfiles` does not
run the separate role-scoped expansion; select `ListInstanceProfilesForRole` (or
run the full catalog) when collecting membership edges.

Lambda root/detail plans share `AWS::Lambda::Function` ARN identity. Aliases have
their own canonical `AWS::Lambda::Alias` ARN identity and are discovered from the
complete sanitized function list, not a code-signing or capacity-provider subset.
Qualified version/configuration APIs are not silently collapsed into function
records.

## Regression scenarios

- Given modeled secret strings/blobs in root and nested members, decoding every
  supported response protocol preserves the metadata and omits the secret.
- Given invalid base64 inside an excluded blob, omission bypasses blob decoding;
  the same unexcluded JSON response fails without echoing its contents.
- Given an absent excluded member with an SDK default, no replacement is inserted.
- Given a decoded sanitized record, encoding and persisting its actual bytes and
  formatting its diagnostic representation do not reintroduce the test marker.
- Given absent policy metadata, prior decoding behavior remains unchanged.
- Given an unmodeled/non-root path, arbitrary map-key traversal, or more than eight
  configured paths, the overlay checker rejects publication.
- Given honestly resealed malformed plan metadata, the runtime consumer still
  rejects invalid exclusion paths.
- Given the six built-in sanitizing plans, their required omissions and callable
  dispositions remain pinned by a regression test.

Focused scenarios live in `crates/{core,aws-wire}/tests/member_exclusions.rs` and
`crates/catalog-tool/tests/member_exclusion_runtime.rs`. Fresh-emulator evidence
must separately inspect the actual database and collector logs; protocol fixtures
alone do not establish AWS or emulator service behavior.

## Remaining boundaries

This mechanism does not authorize credential-fetching APIs, workload payloads,
mutation, arbitrary runtime policy/document parsing, secret-derived identities,
or incomplete resource enumeration. Examples that remain blocked include:

- IAM policy versions/SAML metadata requiring parent identity; inline policy
  discovery requiring correlated owner/policy-name bindings and graph semantics;
  account authorization details requiring multiple independently typed item lists.
- Lambda published versions requiring qualified kind/identity review; concurrency
  and recursion singleton responses requiring parent-identity enrichment. Their
  parent discovery is now available and their defer reasons say so.
- SNS platform-application detail, whose only output is the unsafe open attributes
  map: dropping it leaves no detail metadata, while key allowlisting and
  request-bound enrichment are separate capabilities.
- ELB listener/rule OIDC secrets, CloudFront origin headers, EC2 launch user data,
  RDS parameter/option values, container environment/commands and other mixed
  output families still requiring complete per-operation safety and graph review.
  A `Secret` disposition alone is not proof that identity, request coverage,
  pagination and all other output members are safe.
- Phone/token/accessor/credential APIs where the identity or entire purpose is
  sensitive. No secret identity is enabled by dropping a sibling member.

## Emulator reproduction

Use a fresh pinned emulator with 1,000 IAM profiles, their roles and attachments,
a Lambda function/alias, and an SNS platform application. Put distinct synthetic
markers in the embedded trust document, function environment and platform
credential. The harness seed language does not support arbitrary open-map keys;
seed the Lambda environment and SNS attributes using native signed HTTP requests,
not dotted pseudo-member paths.

Configure the emulator's credential files and CA as in `RUN_HERE.md`. Snapshot the
compiled catalog before opening the database: an existing database deliberately
rejects a different catalog digest.

```sh
cp catalog/aws-api.catalog /tmp/sanitized.catalog
aws-inventory sweep \
  --database .local/e2e/sanitized.sqlite --catalog /tmp/sanitized.catalog \
  --profile default --region us-east-1 \
  --endpoint "$EMULATOR_ENDPOINT" --ca-bundle "$EMULATOR_CA" \
  --service iam:ListInstanceProfiles \
  --service iam:GetInstanceProfile \
  --service iam:ListInstanceProfilesForRole \
  --service lambda:ListFunctions \
  --service lambda:GetFunctionConfiguration \
  --service lambda:ListAliases \
  --service sns:ListPlatformApplications
```

Inspect `resources`, `resource_facets`, `edges` and the actual encoded SQLite
bytes, including present WAL/SHM files. Expect one profile per ARN, root/detail/
role-scoped facets converging on those identities, and one resolved `HAS_ROLE`
edge for each attachment. Check collector logs for the same synthetic markers.
Replay with the **same catalog snapshot** and compare identities/content hashes;
timestamps are not an idempotence assertion.

### Observed acceptance

Against emulator 1.0.348 (`7461ab1bbe540afe8063ce03ee5477ebb649df48`) and pinned
models `46135f8468cd5cb60d983b97290c9f6661e359c6`, the seven-operation selection
completed all 2,006 scopes with no partial, failed or unobserved scopes. SQLite
held 1,000 profiles, each with all three operation facets, 1,000 roles, 1,000
resolved profile→role edges, one Lambda function/alias pair and one SNS platform
application. None of the three synthetic secret markers occurred in stored
SQLite bytes or collector logs; SQLite integrity checking returned `ok`.

A replay selecting all historical roots plus Lambda detail/aliases completed
six scopes with zero resource writes. All 2,003 resource identities and content
hashes were unchanged. Include `iam:ListRoles` in a narrower root-only replay:
otherwise its intentionally omitted historical root is reported as unobserved,
and the collector correctly exits nonzero even though records remain unchanged.
The full seven-operation selection includes that dependency automatically.

Touched-crate gates passed: strict source clippy, catalog compilation, formatting
check, and 2,073 tests across 112 result groups (11 ignored). These observations
do not claim real-AWS behavior, graph delivery, all-feature coverage, secret
zeroization or a formal proof.
