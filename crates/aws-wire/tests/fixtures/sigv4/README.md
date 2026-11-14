# SigV4 header-signing fixtures

Copied byte-for-byte from `aws-sigv4` **1.6.5**, directory
`aws-signing-test-suite/v4/`, installed at
`/cache/cargo-shared/registry/src/index.crates.io-1949cf8c6b5b557f/aws-sigv4-1.6.5/`.
The suite originates in the AWS CRT `aws-c-auth` v0.9.0 signing suite:
https://github.com/awslabs/aws-c-auth/tree/v0.9.0/tests/aws-signing-test-suite
The two `double-*` cases were added by smithy-rs. Amazon's Apache-2.0
license is copied unchanged as `LICENSE`; these fixture bytes are not relicensed.

Only `request.txt`, `header-signed-request.txt`, `header-canonical-request.txt`,
and (where upstream supplies one) `context.json` are copied. Query-signing files
are not copied because this module deliberately supports only header signing.
`sign_vectors.rs` exercises every copied case through `aws_wire::sign::sign`,
comparing the entire Authorization value from `header-signed-request.txt`.
The canonical files and context files preserve reviewable upstream evidence;
they are not generated expectations. No fixture execution is claimed here.

## Fixed context

Ordinary cases use:

- Access key: `AKIDEXAMPLE`
- Secret: `wJalrXUtnFEMI/K7MDENG+bPxRfiCYEXAMPLEKEY`
- Region: `us-east-1`
- Signing service: `service`
- Time: `2015-08-30T12:36:00Z` (Unix second 1440938160)
- Session token: absent, except the two copied token cases, whose exact tokens
  are in their unchanged `context.json` files.

The historical `double-encode-path` case instead uses the AWS credential crate's
published test values `ANOTREAL` / `notrealrnrELgWzOk3IfjzDKtFBhDby` (as in
upstream `src/http_request/sign.rs::test_sign_url_escape`).
`double-url-encode` uses the ordinary key/secret, `us-east-2` / `lambda`, and
`2021-05-11T15:40:45Z` (Unix second 1620747645), as its signed request records.

The bounded test-only request parser splits at the first empty line, preserves
body bytes after that separator (after the same final-newline trimming used by
upstream), retains repeated headers in order, and combines Host and request target
into an absolute HTTPS URI. Published X-Amz-Date input headers are omitted from
the unsigned input because the signer always derives them from the explicit time.
It accepts at most 8192 fixture bytes and 64 header lines.

## Used: 37 cases

- `double-encode-path`, `double-url-encode`
- `get-header-key-duplicate`, `get-header-value-order`, `get-header-value-trim`
- `get-relative-normalized`, `get-relative-unnormalized`
- `get-relative-relative-normalized`, `get-relative-relative-unnormalized`
- `get-slash-dot-slash-normalized`, `get-slash-dot-slash-unnormalized`
- `get-slash-normalized`, `get-slash-unnormalized`
- `get-slash-pointless-dot-normalized`, `get-slash-pointless-dot-unnormalized`
- `get-slashes-normalized`, `get-slashes-unnormalized`
- `get-space-normalized`, `get-unreserved`, `get-utf8`
- `get-vanilla`, `get-vanilla-empty-query-key`, `get-vanilla-query`
- `get-vanilla-query-order-encoded`, `get-vanilla-query-order-key-case`
- `get-vanilla-query-unreserved`, `get-vanilla-utf8-query`
- `get-vanilla-with-session-token`
- `post-header-key-case`, `post-header-key-sort`, `post-header-value-case`
- `post-sts-header-before`, `post-vanilla`, `post-vanilla-empty-query-value`
- `post-vanilla-query`, `post-x-www-form-urlencoded`
- `post-x-www-form-urlencoded-parameters`

The unnormalized cases use S3 path handling. The normalized space case also
uses S3 handling because its published canonical path is single-encoded
`/example%20space/`; this path has no slash/dot normalization to perform.
Upstream skips it because its context lacks a single/double encoding control.
The two form cases request the checksum header as their contexts require.
All other copied cases use the default normalized, double-encoded path mode.

## Unused: 3 cases, explicitly excluded

- `get-header-value-multiline`: obsolete folded HTTP header syntax cannot be
  represented directly by `http::HeaderValue`; the small parser does not unfold
  obsolete headers. Upstream also skips this case for its parser's RFC7230 limit.
- `get-space-unnormalized`: its request target contains a literal unencoded
  space (`/example space/`), which is not a valid encoded HTTP URI. Encoding that
  target in the harness would alter the input under test. Upstream skips it too.
- `post-sts-header-after`: explicitly requires `omit_session_token: true`, contrary
  to this module's fixed rule that temporary credentials are part of the signature.

## Module contract

The caller supplies credentials, validated region/signing name, time, and unsigned
request parts. Standard signing normalizes and double-encodes paths; S3 preserves
the path with single encoding. Session tokens are always signed. Body bytes remain
borrowed; precomputed SHA-256 is supported. Signing adds two to four headers and
never performs I/O or reads the clock. For an expired signature the executor signs
the original unsigned request again with a new supplied time, replacing outbound
signing headers. Signing-owned input headers are rejected to avoid stale values.
`Request::payload_header` selects whether to emit and sign the checksum header;
it is request data, not a separate argument to the five-argument signing function.
Every header value's size is checked before decoding or comparing any header value,
including Host, so oversized headers consistently return the resource-bound error.
URI/method/header/body/credential sizes and supported dates are explicitly bounded
in the public API; malformed input returns typed errors without exposing secrets.

Emulator acceptance/rejection per serializer is intentionally outside M2.2a;
M2.1 serializers and M2.3 transport are prerequisites for that separate proof.
Benchmarks and any signing-key cache remain M2.8; no speed claim is made here.
