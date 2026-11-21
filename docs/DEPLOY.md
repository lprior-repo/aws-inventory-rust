# Deploy the read-only role

`deploy/read-only-role.yaml` creates the cross-account role that aws-inventory
assumes. Deploy it in every account to be scanned.

## What the template creates

- `AWS::IAM::ManagedPolicy`: the 79 read-only actions from
  `catalog/iam-read-only-policy.json`, copied verbatim. That file is generated
  from the reviewed operation catalog by `catalog-tool iam-policy`. The policy is
  2,215 non-whitespace characters, under the 6,144-character managed-policy limit
  and the 10,240-byte inline limit, so it is not split. Resources are `"*"`
  because the catalog carries no resource ARNs.
- `AWS::IAM::Role` named `RoleName` (default `aws-inventory-read-only`). Its trust
  policy allows only `TrustedPrincipalArn`, and only when `sts:ExternalId`
  equals `ExternalId`.
- Output `RoleArn`.

A test in `crates/catalog-tool/tests/iam_policy.rs` fails when the template's
actions differ from the catalog policy. After the catalog changes, regenerate the
policy and update the template.

## Deploy

`TrustedPrincipalArn` must be an IAM role or user ARN, not an assumed-role session
ARN. That principal also needs `sts:AssumeRole` on the role in its own IAM policy.
`ExternalId` is a `NoEcho` parameter; keep it secret.

```bash
export INVENTORY_EXTERNAL_ID='...'   # choose a random value, 2-1224 chars

aws cloudformation deploy \
  --template-file deploy/read-only-role.yaml \
  --stack-name aws-inventory-read-only \
  --capabilities CAPABILITY_NAMED_IAM \
  --parameter-overrides \
    TrustedPrincipalArn=arn:aws:iam::111111111111:role/InventoryRunner \
    ExternalId="$INVENTORY_EXTERNAL_ID"

aws cloudformation describe-stacks --stack-name aws-inventory-read-only \
  --query 'Stacks[0].Outputs[?OutputKey==`RoleArn`].OutputValue' --output text
```

## Point the tool at the role

Export the same external ID value in the variable named by `--external-id-env`.
The tool's debug output reports only whether an external ID is present, never its
value.

One account, using the role ARN from the stack output:

```bash
export INVENTORY_EXTERNAL_ID='...'
aws-inventory scan --db inventory.sqlite \
  --role-arn arn:aws:iam::222222222222:role/aws-inventory-read-only \
  --external-id-env INVENTORY_EXTERNAL_ID --region us-east-1 --service all
```

An AWS Organization: deploy the template to each member account you want scanned
(for example with a StackSet), using the same `TrustedPrincipalArn`, `ExternalId`
and `RoleName`. Then scan through the management or delegated-admin profile:

```bash
aws-inventory scan --db inventory.sqlite --profile management \
  --organization-role aws-inventory-read-only \
  --external-id-env INVENTORY_EXTERNAL_ID --region us-east-1 --service all
```

Add `--account-id` to limit the scan to specific accounts.

Service control policies and permissions boundaries can still deny an action even
when the role allows it. Review the scan result with `aws-inventory report --db
inventory.sqlite` to see denied or partial collection.
