# Cloud Control collection and architecture coverage

The executable Cloud Control catalog contains **418 reviewed regional resource types across 76 service families**: the frozen **397 root types** plus **21 parent-dependent types**. It uses the official Rust Cloud Control SDK's `ListResources` and `GetResource` operations through the shared executor. Parent discovery supplies an explicit `ResourceModel` for each reviewed child list. The same adapter handles each selected descriptor; adding a service SDK dependency alone does not add a collector.

The retained [capability matrix](../catalog/cloudcontrol-capabilities.json) audits **all 1,881 schemas** in the AWS registry archive and preserves the v0.5 root selection and executed evidence. The offline `coverage` command overlays the separate [parent catalog](../catalog/cloudcontrol-parent-resources.json), reports the current 418-type selection, and exposes its prerequisite plans without promoting historical results into new evidence. The [generated evidence rollup](CLOUDCONTROL_EVIDENCE.md) remains a source-bound record for the root catalog. Schema eligibility, implemented workflows and executed evidence are separate.

## Retained v0.5 evidence

The final v0.5 emulator run attempted all **397 selected type lists** and stored detailed observations for **386 types**. The Cloud Control scan contains **389 resource observations and 32 relationships**, with **775 successful scopes and 11 failed type-list scopes**. All **1,188 harness checks passed**, including the 111 independent architecture checks. The scan remains partial because the pinned emulator does not register the eleven failed namespaces. [Final collection summary](../artifacts/emulator-v05/summary.json) · [Exact scan receipt](../artifacts/emulator-v05/cloud-control.json)

The v0.5 coverage rows were bound only after the final health-supervised wrapper completed, two unmutated four-case snapshot repetitions passed, and the live first-party workspace passed all 503 tests. The four exports matched SQLite payloads and provenance; the real local FalkorDB checks compared resource identities, reference-only endpoints, relationship identities and provenance. Checkpointed replay sent no graph commands. These are retained v0.5 results, not a Rust or live-emulator pass claim for the new parent workflows. See [VERIFICATION.md](VERIFICATION.md) for current command outcomes. [Delivery verification](../artifacts/emulator-v05/delivery-verification.json) · [Detailed historical results](EMULATOR_TESTING.md#historical-v05-integration-results)

## Selection and exclusions

| Evidence category | Count | Meaning |
| --- | ---: | --- |
| Pinned schemas | 1,881 | Distinct type schemas in the retained official archive |
| Declared read and list handlers | 1,701 | Both handler declarations exist in that schema |
| Missing read or list | 180 | Excluded from this read/list adapter |
| Documented parent model still excluded | 32 | Required context has no admitted workflow |
| Additional schema parent model still excluded | 357 | The list handler rejects an empty object and no workflow is admitted |
| Schema-eligible but scope unreviewed | 894 | Not executable through this catalog |
| Reviewed regional root selection | 397 | Original 200 plus 197 reviewed additions; frozen root catalog |
| Reviewed parent-dependent selection | 21 | Five previously documented exclusions and sixteen schema exclusions now have explicit plans |
| Combined executable selection | 418 | Disjoint root and parent selections |
| Remaining exclusions | 1,463 | Missing handlers, missing parent workflows or unreviewed scope |

Every selected type has read/list handlers and a reviewed regional collection plan. The 397 root types accept an empty list model according to the pinned handler schema. The additional 21 types require an explicit model derived from a successfully read parent. This is not a claim that every account, partition, region, provider version or IAM policy permits collection. AWS documents that list output can contain incomplete properties; the adapter obtains `GetResource` details before deriving architecture edges. [AWS list-operation documentation](https://docs.aws.amazon.com/cloudcontrolapi/latest/userguide/resource-operations-list.html).

The selected additions include SageMaker, Bedrock, DAX, CloudFormation stacks, EMR, SES, Cognito, CodeDeploy and Image Builder, plus regional networking, database, observability and integration resources. The explicit [addition manifest](../catalog/cloudcontrol-regional-additions.json) names every admitted resource and the proposals rejected during review.

The new workflows enumerate children of EKS clusters, ELBv2 load balancers, API Gateway REST APIs, API Gateway V2 APIs, Logs log groups and AppConfig applications. A child-only selection adds its exact root prerequisite; selected siblings share one root enumeration. Root-only selection does not add children. See [PARENT_COLLECTIONS.md](PARENT_COLLECTIONS.md) for the exact 21 bindings, failure semantics and limits.

Global IAM, Route 53 hosted zones, CloudFront distributions and S3 buckets continue to use their native collectors. The regional Cloud Control list does not duplicate those global collection decisions. ELBv2 `Listener` is included; `ListenerRule` requires a second dependency level and remains excluded. Unreviewed policy, RDS proxy-target-group and other parent workflows also remain excluded. EC2 IPAM-family proposals remain scope-unreviewed: their ARN forms omit a region, so a generic regional identity assumption would be unsafe. [EC2 resource ARN reference](https://docs.aws.amazon.com/service-authorization/latest/reference/list_ec2.html).

The matrix's stable entry fields are `type_name`, `service`, `service_name`, `selected`, `exclusion`, `scope_review`, read/list capability flags, list-model evidence, schema file/hash, relationship-rule count, view name and an independent `emulator_verification` object. Inspect it without credentials:

```bash
aws-inventory coverage --service SageMaker --limit 100
aws-inventory coverage --type-name AWS::EKS::Addon --limit 1
aws-inventory coverage --type-name AWS::EC2::IPAM --limit 1
aws-inventory coverage --unselected --limit 2000
```

## Relationship extraction

The combined catalog declares **965 property relationship rules across 255 selected types**, plus **21 runtime `CHILD_OF` workflows**. There are **265 distinct types with at least one property rule or parent workflow**. The frozen root selection retains its 945 rules across 244 types. A property rule means that an eligible returned value can produce an edge; it does not guarantee that every resource has that property or that every rule has been exercised against AWS.

A parent edge requires a detailed child response whose model properties exactly match the requested parent context. Its target is the observed parent resource identity, with `owner_resolution=observed_parent_resource_identity`. The child observation retains `parent_resource_id`, `parent_type` and `list_resource_model`. A provider that ignores the filter cannot silently attach a contradictory response to the requested parent.

The compiler resolves local schema definitions and declared array items. It generates explicit JSON-pointer rules only for declared string properties with ARN semantics or reviewed single-identifier fields. It does not recursively search arbitrary provider JSON for strings resembling ARNs. Opaque maps, unresolved external schema references and cyclic definitions cannot justify invented dependency paths. CloudFormation schemas distinguish read-only properties, write-only properties and potentially compound primary identifiers. [AWS schema semantics](https://docs.aws.amazon.com/cloudformation-cli/latest/userguide/resource-type-schema.html).

| Reference form | Result |
| --- | --- |
| Exact returned role ARN | Preserve its partition/account/path; require IAM `role/` when the rule is typed |
| Exact returned KMS key ARN | Preserve its account and region; require KMS `key/` |
| Exact returned VPC Lattice target-group ARN | Require the `vpc-lattice` service and target-group resource prefix |
| Reviewed `subnet-…`, `sg-…`, `vpc-…` identifier | Form the reviewed ARN and record `owner_resolution=assumed_scanning_account` |
| EC2 image or snapshot identifier | Use the official ARN form with an empty account segment |
| Compound identifier, ARN in a short-ID property, path, wildcard selector or malformed typed ARN | Do not fabricate a target ARN |
| Unknown schema-declared ARN reference | Retain the exact ARN as a generic reference without inventing a resource kind |
| Undeclared ARN-looking property or redacted subtree | Do not derive an edge |

Typed references are checked at runtime before an edge is emitted. A name that happens to start with `sg-` is not enough: EC2 Instance `SecurityGroups` is explicitly excluded because AWS defines that property as names; `SecurityGroupIds` remains eligible. [EC2 property semantics](https://docs.aws.amazon.com/AWSCloudFormation/latest/TemplateReference/aws-resource-ec2-instance.html). ECS VPC Lattice target groups have explicit overrides because they use a different service from ELB target groups. [ECS VPC Lattice properties](https://docs.aws.amazon.com/AWSCloudFormation/latest/TemplateReference/aws-properties-ecs-service-vpclatticeconfiguration.html) and [target-group ARN grammar](https://docs.aws.amazon.com/vpc-lattice/latest/APIReference/API_GetTargetGroup.html).

Certificate references also require context. All 25 generated `CertificateArn`-family relationship sites were reviewed: ten documented ACM sites retain an ACM target kind; fourteen DMS endpoint/data-provider sites retain exact generic references; one MSK Replicator CA-certificate property is explicitly a Secrets Manager reference. The suffix alone no longer assigns an ACM kind. [DMS certificate example](https://docs.aws.amazon.com/cli/latest/userguide/cli_database-migration-service_code_examples.html) · [MSK OAuth CA secret example](https://docs.aws.amazon.com/msk/latest/developerguide/msk-replicator-external-api-examples.html) · [VPC Lattice ACM requirement](https://docs.aws.amazon.com/vpc-lattice/latest/ug/service-custom-domain-name.html).

Returned ARNs retain ownership information already present in the identifier. Short-ID rules explicitly expose the scanning-account assumption; shared-VPC or cross-account topology must not treat that assumption as authoritative ownership. ARN references describe observed configuration dependencies, not effective permissions, packet reachability, successful authentication or proven runtime data flow.

The snapshot self-identity template was corrected to `arn:{partition}:ec2:{region}::snapshot/{identifier}`. EBS snapshot and EC2 image ARNs omit the account segment in AWS's resource ARN table. Compound resource identifiers remain stable `awscc:` identities with a hash of the complete provider identifier instead of an invented ARN. [EC2 resource ARN reference](https://docs.aws.amazon.com/service-authorization/latest/reference/list_ec2.html).

Identical `(target, relation)` observations merge their property-path evidence. Relationships to the resource itself are omitted. List-only fallback records preserve the partial inventory but do not produce architecture edges.

## Redaction and bounds

Provider properties are sanitized before names, tags, SQL columns or edge evidence are derived. Schema write-only properties, shared sensitive-key rules and explicit per-type redaction all apply. The additions include specific handling for API-key values, private certificate material, credentials, environment values, model/pipeline definitions, prompts, template bodies, stack parameters/outputs and arbitrary query or script content. This is a metadata inventory; payload values are not needed for architecture topology.

The mapper bounds provider JSON at 262,144 bytes, JSON nesting at 32 levels, traversal at 16,384 nodes, relationship rules at 256 per descriptor, and output edges at 4,096 per resource. Each root group additionally admits at most 10,000 child-model list scopes and 100,000 distinct child identifiers across its selected families. Parent models have at most four bindings, values of at most 1,024 bytes and an 8,192-byte encoded limit. Duplicate child models from different parents and duplicate child identifiers across parents fail before replaying the scope. Existing pagination, cancellation and shared-executor retry bounds remain active.

The `cloud_control_parent_discovery` scope completes only after exhaustive successful prerequisite discovery. A successful empty root list proves an empty child discovery; a denied root list, traversal cap, invalid required property or failed required read leaves explicit incomplete evidence. Child list/read failures stay independent, and successful observations under other parents remain available. List-only fallback records do not acquire parent or property-derived edges.

The original 200 type order and SQL column definitions remain identical. Migration 003 retains those views; migration 005 adds the 197 v0.5 root views. **Schema version 6** adds **21 parent-type views** through `006_cloudcontrol_parents.sql`, bringing the total to **418 source-specific views and 4,273 provider columns**. Existing observations and unrelated database objects are preserved; a name collision fails the upgrade transaction.

## Verification

The retained schema suite checks the full 1,881-entry exclusion partition, every selected schema hash, every generated property path and string type, typed constraints, recursive-reference bounds and the frozen original view-column contract.

Rust mapping tests use 18 independently authored fixtures containing 34 exact expected edges, including cross-account IAM/KMS references, nested Firehose properties, duplicate-path evidence, compound volume attachments, accountless snapshots, invalid identifiers, unknown properties, distinct ECS ELB/VPC Lattice targets, DMS certificates and MSK CA secrets. Four additional cases check sensitive new-type redaction. Three fixed-seed property tests exercise typed ARN constraints, short-ID rejection and accepted accountless ARN forms.

The emulator seeder uses thirteen of the semantic fixtures for supported generic-provider types. The harness then checks 111 assertions on the actual Rust SDK-to-SQLite result: resource identities, all 30 expected edges, exact target kinds, ownership evidence, property paths, redaction and timestamps. Seven adversarial checker tests ensure missing/duplicate edges, wrong ownership, wrong paths, foreign scans/sources and redaction failures remain failures.

The parent fixture is independent of those historical root models. It defines twelve parents, 63 semantic child records covering all 21 types, and 98 additional EKS addons. EKS Addon therefore has 101 records, forcing the pinned emulator's default 100-item window and an empty nonterminal filtered page for the second parent. The new oracle requires 161 exact detailed child observations, 161 `CHILD_OF` edges, 24 separately authored architecture edges, complete prerequisite scopes and both addon pagination chains. It checks actual create-returned identifiers rather than assuming the emulator's identifier heuristic. These are test contracts; current Rust and live results require their own receipts.

Pure parent tests cover literal mappings, fixed-seed selection/model properties, denied and empty discovery, independent parent cursors, duplicate models, aggregate admission caps and cancellation. A separate bounded HTTP fixture exercises the actual Rust SDK serialization and executor retry path with two parents and an injected throttle. The emulator wrapper now prepares five snapshot cases and ten guarded integration cases. [Reproduction and fixture contract](EMULATOR_TESTING.md) · [Parent workflow design](PARENT_COLLECTIONS.md)

Most Cloud Control types in AWS Emulator 1.0.336 are handled by a generic property store. Only SQS Queue, Logs LogGroup and Logs Destination bridge to native service stores in the pinned source. A successful local generic-provider read proves the collection and mapping path, not live AWS resource-provider validation. The emulator's known failures remain separate outcomes, including selected namespaces it does not register. [Pinned Cloud Control emulator implementation](https://github.com/aws/aws-emulator/tree/f515581569b98a9dcbd2e686c12f912fdce51bba/services/service-com-amazonaws-cloudcontrol-cloudapiservice).

Config and Resource Explorer keep their own source contracts. Their lower-cased or sparse property shapes are not automatically treated as CloudFormation models, so these new relationship rules are not blindly applied to supplemental records.

## Reproduction and evidence refresh

The archive and its SHA-256 are retained under `catalog/upstream`; generation never reads live credentials or calls account APIs. Check the separate parent catalog and its literal schema/oracle tests without changing historical root evidence:

```bash
python3 catalog/upstream/build_parent_catalog.py --check
python3 -m unittest discover -s catalog/upstream -p test_parent_catalog.py -v
python3 -m unittest discover -s scripts/emulator -p test_assert_parents.py -v
cargo run --locked -p inventory-engine --example resource_views -- --check
```

The root generator `catalog/upstream/build_cloudcontrol_catalog.py` is for intentional root-catalog regeneration; it resets executed evidence. After a fresh asserted emulator run with the current sources and binary, bind root-provider request statuses to persisted typed reads:

```bash
python3 scripts/catalog/refresh_cloudcontrol_coverage.py \
  --emulator-summary artifacts/emulator-v06-local/summary.json
```

Before collection, the harness compares the executing CLI's embedded catalog with the merged 418 descriptors. It explicitly selects the frozen 397 roots for `cloud-control.json` and the 21 child types, with their automatically included prerequisites, for `parents.json`. The summary binds both catalog hashes, the actual CLI receipt, binary hash, source/manifest fingerprint, fixture hashes and exact command/scan identities. Root coverage refresh requires unchanged bindings, a matching root selected-type set, a terminal scan, matching schema hashes, passed assertions and read-only SQLite observations from that exact scan. Parent results remain separately inspectable in `parents.json`, `parent-assertions.json` and the seed receipt. The retained v0.5 summary cannot certify a changed v0.6 binary.

Python seeding is excluded. A transport 200 without a persisted detailed record becomes `response_without_persisted_read`, not a successful read. Regenerating the catalog intentionally resets executed evidence; rerun this explicit binding step afterward. Rebuild the CLI to embed any refreshed capability matrix in its offline `coverage` output.
