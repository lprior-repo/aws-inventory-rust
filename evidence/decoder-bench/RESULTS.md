# Response decoder tokenizer evidence — 2026-11-13

## Decision

**JSON: `jiter = 0.19.0`, without default features**, if a bounded, contiguous response buffer is acceptable. **XML: `quick-xml = 0.42.0`.** Both consume tokens/events without a document tree. Neither recommendation requires `serde_json` for tokenizing. These are evidence recommendations, not production decoders or changes to VISION/ROADMAP.

JSON runner-up by throughput is the **Deserializer/DeserializeSeed/Visitor use of `serde_json`**, never `serde_json::Value`. Choosing it would require changing **M3.5's dependency prohibition**, even though it builds no JSON tree. If resumable socket chunks rather than a buffered page are mandatory, choose **`json-event-parser = 0.2.3`** instead: it is the strongest eligible incremental alternative measured here, not a claim that Jiter can resume on incomplete input. XML runner-up is **`xmlparser = 0.13.6`**: slightly faster on the smallest page, slower on medium/large pages, and requires more framing/entity work outside its tokenizer.

Confidence: **moderate**. Jiter wins JSON records at every size; quick-xml wins XML records at medium/large sizes. Confidence concerns the measured adapters/workload, not arbitrary AWS models, third-party soundness, or optimal possible adapters. Re-evaluate with M2.4 protocol fixtures, a public streaming record encoder, different string/escape distributions, a no-buffer socket requirement, another CPU, or an applicable security advisory. No library was rejected for being unsafe-heavy or unmaintained: unsafe counts are evidence, not a safety proof or maintenance verdict. The extra candidates are json-event-parser (explicit events plus reader/low-level incremental APIs) and `xml` 1.4.0 (the maintained package to which xml-rs 1.0.0 points).

### Headline: full tokenize + shape + item tree + CBOR record, largest page

| Candidate | MiB/s median (min–max) | Items/s | ms/page | Allocs/item | Allocated B/item | Incremental peak live heap B |
|---|---:|---:|---:|---:|---:|---:|
| **jiter** | **1276.88 (1264.46–1286.88)** | **386627** | **3.880** | **27.01** | **17577.99** | 9481693 |
| serde_json seed/visitor | 1066.68 (1064.78–1071.60) | 322979 | 4.644 | 56.01 | 17795.65 | 9476540 |
| aws-smithy-json | 651.12 (647.87–655.29) | 197153 | 7.608 | 28.01 | 20628.53 | 9473501 |
| json-event-parser | 648.43 (637.09–650.08) | 196337 | 7.640 | 32.01 | 20734.53 | 9473501 |
| minimal handwritten | 531.65 (526.80–533.22) | 160977 | 9.318 | 28.01 | 20628.54 | 9473509 |
| struson | 490.04 (486.47–494.30) | 148380 | 10.109 | 49.02 | 25753.97 | 9474037 |
| **quick-xml** | **751.96 (746.10–757.21)** | **213266** | **7.033** | **83.02** | **21090.93** | 9481396 |
| xmlparser | 640.75 (593.68–645.54) | 181726 | 8.254 | 78.01 | 24065.71 | 9481232 |
| aws-smithy-xml — disqualified | 418.58 (414.11–422.37) | 118715 | 12.635 | 75.02 | 24039.86 | 9481232 |
| xml | 74.12 (70.88–76.21) | 21023 | 71.352 | 565.13 | 81082.86 | 9484900 |

Jiter is about 2.40× the handwritten baseline for full large-page decoding, with a 37-line adapter instead of the baseline's **126 lines** (40 parser + 86 lexer/unescaper). quick-xml is about 1.17× xmlparser on that workload; xmlparser is about 1.08× quick-xml on small pages. Prefer library code over maintaining the baseline.

## Record API findings: M2.4a/b prerequisites

The public API is `Record::encode(&Value)`, requires `Value::Map`, and accepts owned `Value::List(Vec<Value>)` / `Value::Map(BTreeMap<Key, Value>)`. There is **no public event/stream writer**, borrowed-value encoder, container begin/end API, ordered encoded-field staging API, or definite-length container count backpatching API. `Record::from_bytes(Vec<u8>)` validates already encoded bytes; it does not construct records from wire events. `minicbor` is internal to the implementation, not exposed as a record builder.

Therefore this benchmark takes the contract's explicit fallback: **one transient owned record Value tree per item**, encoded immediately through the real public API, never a page/document tree. Page output holds only the completed record byte vectors. Unknown members are traversed/validated without constructing their values. Both token-only and token-plus-record results are measured; token-only still includes shared shape/framing traversal and string unescaping, and XML still stages scalar text. It is not a bare-lexer microbenchmark. JSON/XML records are byte-identical for all three sizes before timing, including sorted field IDs, sorted tag-name keys, exact i64 longs, ISO/epoch timestamp normalization and decoded blobs. Independent public record readers also check selected values, escapes and types.

**M3.5 has an additional existing conflict**: inventory-core itself directly depends on serde_json in this worktree. Merely depending on inventory-core from aws-wire would violate the transitive `cargo tree -i serde_json` criterion, independently of which tokenizer is chosen. This evidence crate intentionally does not change product dependencies or M3.5.

A retained JSON `Document` member with 64 nested arrays tokenizes with every JSON candidate, but every adapter then returns the public record encoder's typed `RecordError::DepthLimit` (record maximum is 32 containers). The Serde adapter preserves that original error rather than losing it in `de::Error::custom`. The depth-64 *unknown/skipped* document succeeds for all candidates. Supporting retained depth 64 is a record-format/API decision, not a tokenizer repair.

## Machine, workload and measurement

- AMD Ryzen 9 9950X3D, 16 physical cores / 32 logical CPUs; benchmark pinned to logical CPU 7 with `taskset`. CPU governor `performance`; boost enabled, maximum reported 5756.4521 MHz. Other workers were active; machine was not reserved or thermally stabilized.
- RAM: 132498264064 bytes (about 123.40 GiB). Linux `7.2.11-arch1-1`, x86_64, `#1 SMP PREEMPT_DYNAMIC Fri, 16 Oct 2026 00:28:58 +0000`.
- Root toolchain copied unchanged: `nightly-2026-11-11`; `rustc 1.102.0-nightly (9c5648833 2026-11-10)`.
- Root release settings copied unchanged: opt-level 3, thin LTO, codegen-units 1, strip symbols; ordinary unwinding, no native-CPU flags or unsafe first-party code.
- Single-threaded, deterministic Rust generator, LCG seed `0x5eed` plus item index; generation and oracle work are outside timed regions. No megabyte fixtures committed.
- Synthetic enriched EC2 DescribeInstances-shaped ec2Query XML: response/reservation/instances wrappers, instance state, group structures/lists, tag key/value entries converted to a map. Equivalent awsJson1.1-style `Items` page. Extra modeled fields deliberately exercise floats, booleans, nulls and blobs that a literal EC2 instance does not expose in these exact forms. This is not an assertion that the synthetic operation is a real AWS model.
- Every item has a varying deterministic description, escaped quote/backslash/newline, Unicode escapes including a surrogate pair, literal Zoë/東京, nested structures, lists, maps, integer 8, float 0.125, long 9223372036854775806, true, null, timestamp 1735689600.125 / 2025-01-01T00:00:00.125Z, and base64 `AAEC/f7/`. Unknown members occur inside items and after the page list. Field wire order is deliberately not field-ID order.

| Size | Items/page | Padding/item | JSON bytes | XML bytes |
|---|---:|---:|---:|---:|
| small | 2 | 450 | 1896 | 2610 |
| medium | 100 | 1550 | 201370 | 225016 |
| large | 1500 | 3000 | 5194570 | 5545816 |

Each candidate/mode/size has three untimed warmups then **15 measured samples**, each batching calls for at least 10 ms or until 4096 calls. Tables show median and the full minimum–maximum spread of sample throughput, not a confidence interval or per-request latency percentile. `std::hint::black_box` covers input/output; output destruction is included. A separate measured decode reports allocations/allocated bytes and peak simultaneously live heap through the existing `allocation-counter` allocator: no handwritten GlobalAlloc/unsafe. The allocator wrapper is linked into all candidates, so absolute timings include its disabled-region bookkeeping. Allocations count allocator calls, including Vec growth; bytes allocated are cumulative, not retained bytes.

Peak values exclude the already allocated input and generation/oracle buffers. For the recommended large-page decoders, **input + incremental peak live heap** is 14676263 B (Jiter) and 15027212 B (quick-xml). This is heap high-water, **not RSS**; process RSS, allocator arena retention, stack, shared libraries, and the generator retaining both wire forms are not measured. Returning a page of record Vecs rather than emitting each record to a consumer contributes to peak memory and spare output capacity. No allocator/storage alternative was evaluated, so no claim that this owned-tree storage layout is optimal.

Selection gate: exact byte parity and the adversarial framing/panic gate precede speed. For future regression triage on this machine, a repeated large-page median below 1085 MiB/s for Jiter or 639 MiB/s for quick-xml (>15% below this run) merits investigation, not changing correctness tests. These are documented comparison thresholds, not machine-independent CI promises.

## Complete measurements

`T` = token/event + shape traversal, no record Values; `R` = traversal + per-item Value + public record encoder. Peak is incremental live heap excluding input.

| Candidate | Size | Mode | MiB/s median | MiB/s min–max | Items/s | µs/page | Allocs/item | Allocated B/item | Peak B |
|---|---|---|---:|---:|---:|---:|---:|---:|---:|
| aws-smithy-json | small | T | 454.47 | 384.14–466.75 | 502688 | 4 | 4.50 | 4742.50 | 8962 |
| aws-smithy-json | small | R | 342.99 | 294.90–354.12 | 379374 | 5 | 30.50 | 12399.50 | 16685 |
| jiter | small | T | 788.48 | 780.18–809.49 | 872127 | 2 | 4.00 | 5060.50 | 10109 |
| jiter | small | R | 492.40 | 485.02–508.07 | 544641 | 4 | 30.00 | 12717.50 | 18333 |
| struson | small | T | 314.05 | 311.81–324.84 | 347370 | 6 | 27.50 | 5580.50 | 9648 |
| struson | small | R | 246.14 | 244.24–253.50 | 272257 | 7 | 53.50 | 13237.50 | 17221 |
| serde_json | small | T | 596.78 | 588.61–614.72 | 660090 | 3 | 36.50 | 4773.00 | 9020 |
| serde_json | small | R | 408.63 | 402.70–420.48 | 451980 | 4 | 62.50 | 12430.00 | 17174 |
| minimal | small | T | 491.85 | 474.79–507.35 | 544031 | 4 | 5.00 | 4751.00 | 8970 |
| minimal | small | R | 349.87 | 341.79–361.96 | 386984 | 5 | 31.00 | 12408.00 | 16693 |
| json-event-parser | small | T | 302.76 | 295.67–307.63 | 334883 | 6 | 8.00 | 4848.00 | 9012 |
| json-event-parser | small | R | 252.71 | 251.23–256.00 | 279518 | 7 | 34.00 | 12505.00 | 16685 |
| quick-xml | small | T | 376.06 | 372.25–378.85 | 302169 | 7 | 66.50 | 9252.00 | 16890 |
| quick-xml | small | R | 304.57 | 301.42–308.60 | 244724 | 8 | 92.50 | 16909.00 | 24580 |
| xmlparser | small | T | 394.09 | 391.14–407.85 | 316657 | 6 | 58.50 | 9509.50 | 17182 |
| xmlparser | small | R | 327.85 | 326.68–331.09 | 263430 | 8 | 84.50 | 17166.50 | 24416 |
| aws-smithy-xml | small | T | 200.85 | 198.84–203.20 | 161383 | 12 | 56.00 | 9595.50 | 17191 |
| aws-smithy-xml | small | R | 180.74 | 180.04–182.79 | 145223 | 14 | 82.00 | 17252.50 | 24416 |
| xml | small | T | 39.08 | 38.70–39.39 | 31404 | 64 | 628.50 | 69966.50 | 21619 |
| xml | small | R | 37.95 | 37.64–38.26 | 30495 | 66 | 654.50 | 77623.50 | 28084 |
| aws-smithy-json | medium | T | 744.21 | 446.53–759.46 | 387525 | 258 | 3.03 | 1698.57 | 10062 |
| aws-smithy-json | medium | R | 538.98 | 365.38–544.03 | 280658 | 356 | 28.10 | 13458.37 | 356521 |
| jiter | medium | T | 1807.25 | 1801.48–1828.90 | 941075 | 106 | 2.04 | 174.49 | 16653 |
| jiter | medium | R | 941.46 | 934.75–949.65 | 490239 | 204 | 27.11 | 11934.29 | 364713 |
| struson | medium | T | 525.73 | 381.90–531.91 | 273760 | 365 | 23.13 | 4184.19 | 11952 |
| struson | medium | R | 403.92 | 402.86–407.60 | 210329 | 475 | 48.20 | 15943.99 | 357057 |
| serde_json | medium | T | 1306.06 | 1300.20–1315.83 | 680095 | 147 | 31.11 | 330.88 | 10120 |
| serde_json | medium | R | 782.75 | 775.50–792.74 | 407595 | 245 | 56.18 | 12090.68 | 358110 |
| minimal | medium | T | 633.11 | 614.35–641.97 | 329676 | 303 | 3.04 | 1698.74 | 10070 |
| minimal | medium | R | 467.67 | 462.83–475.04 | 243525 | 411 | 28.11 | 13458.54 | 356529 |
| json-event-parser | medium | T | 612.18 | 610.26–617.57 | 318773 | 314 | 7.02 | 1804.56 | 10112 |
| json-event-parser | medium | R | 471.31 | 467.59–477.39 | 245421 | 407 | 32.09 | 13564.36 | 356521 |
| quick-xml | medium | T | 678.08 | 672.31–693.72 | 315986 | 316 | 58.17 | 2236.62 | 17990 |
| quick-xml | medium | R | 512.19 | 509.18–516.38 | 238683 | 419 | 83.24 | 13996.42 | 364416 |
| xmlparser | medium | T | 649.84 | 642.01–654.43 | 302824 | 330 | 53.11 | 3758.27 | 19382 |
| xmlparser | medium | R | 498.26 | 496.38–501.31 | 232189 | 431 | 78.18 | 15518.07 | 364252 |
| aws-smithy-xml | medium | T | 346.64 | 342.23–349.55 | 161533 | 619 | 50.12 | 3734.51 | 19391 |
| aws-smithy-xml | medium | R | 296.89 | 294.38–299.90 | 138350 | 723 | 75.19 | 15494.31 | 364252 |
| xml | medium | T | 61.18 | 58.67–62.01 | 28508 | 3508 | 540.83 | 57289.53 | 24069 |
| xml | medium | R | 58.49 | 58.01–59.04 | 27258 | 3669 | 565.90 | 69049.33 | 367920 |
| aws-smithy-json | large | T | 892.34 | 887.52–900.20 | 270192 | 5552 | 3.00 | 3069.64 | 11512 |
| aws-smithy-json | large | R | 651.12 | 647.87–655.29 | 197153 | 7608 | 28.01 | 20628.53 | 9473501 |
| jiter | large | T | 2866.63 | 2841.16–2895.77 | 867987 | 1728 | 2.00 | 19.10 | 16653 |
| jiter | large | R | 1276.88 | 1264.46–1286.88 | 386627 | 3880 | 27.01 | 17577.99 | 9481693 |
| struson | large | T | 644.77 | 640.82–647.70 | 195231 | 7683 | 24.01 | 8195.08 | 15024 |
| struson | large | R | 490.04 | 486.47–494.30 | 148380 | 10109 | 49.02 | 25753.97 | 9474037 |
| serde_json | large | T | 2049.98 | 2043.08–2062.17 | 620714 | 2417 | 31.01 | 236.76 | 11570 |
| serde_json | large | R | 1066.68 | 1064.78–1071.60 | 322979 | 4644 | 56.01 | 17795.65 | 9476540 |
| minimal | large | T | 691.32 | 680.34–699.29 | 209323 | 7166 | 3.00 | 3069.65 | 11520 |
| minimal | large | R | 531.65 | 526.80–533.22 | 160977 | 9318 | 28.01 | 20628.54 | 9473509 |
| json-event-parser | large | T | 889.75 | 886.97–914.89 | 269407 | 5568 | 7.00 | 3175.64 | 11562 |
| json-event-parser | large | R | 648.43 | 637.09–650.08 | 196337 | 7640 | 32.01 | 20734.53 | 9473501 |
| quick-xml | large | T | 1100.09 | 1092.65–1104.90 | 312000 | 4808 | 58.01 | 3532.04 | 19440 |
| quick-xml | large | R | 751.96 | 746.10–757.21 | 213266 | 7033 | 83.02 | 21090.93 | 9481396 |
| xmlparser | large | T | 876.39 | 871.39–881.98 | 248555 | 6035 | 53.01 | 6506.82 | 22282 |
| xmlparser | large | R | 640.75 | 593.68–645.54 | 181726 | 8254 | 78.01 | 24065.71 | 9481232 |
| aws-smithy-xml | large | T | 508.55 | 503.75–515.78 | 144231 | 10400 | 50.01 | 6480.97 | 22291 |
| aws-smithy-xml | large | R | 418.58 | 414.11–422.37 | 118715 | 12635 | 75.02 | 24039.86 | 9481232 |
| xml | large | T | 78.75 | 76.97–80.14 | 22335 | 67158 | 540.12 | 63523.97 | 28079 |
| xml | large | R | 74.12 | 70.88–76.21 | 21023 | 71352 | 565.13 | 81082.86 | 9484900 |

## Hostile input behavior

A = accepted, E = typed error. Every row observed **zero panics and zero cases taking ≥2 seconds**. All candidates except aws-smithy-xml rejected every byte prefix of their small input (including the empty prefix). Recommended candidates' expectations are enforced as errors from the benchmark, not merely printed. `catch_unwind` is measurement containment, not a decoder fallback. The command-level 300-second tool deadline bounds an entire run; ≥2s is a post-return observation, not a preemptive per-case watchdog, so this is not a general no-hang proof.

| Candidate | Truncated prefixes: accepted / errors | Invalid UTF-8 | Unterminated string/tag | 1 MiB string | Duplicate known key | Unknown members | Skipped depth 64 | Mismatched/trailing syntax |
|---|---:|---|---|---|---|---|---|---|
| aws-smithy-json | 0 / 1896 | E | E | A | E | A | A | E |
| jiter | 0 / 1896 | E | E | A | E | A | A | E |
| struson | 0 / 1896 | E | E | A | E | A | A | E |
| serde_json | 0 / 1896 | E | E | A | E | A | A | E |
| minimal | 0 / 1896 | E | E | A | E | A | A | E |
| json-event-parser | 0 / 1896 | E | E | A | E | A | A | E |
| quick-xml | 0 / 2610 | E | E | A | E | A | A | E |
| xmlparser | 0 / 2610 | E | E | A | E | A | A | E |
| **aws-smithy-xml** | **2418 / 192** | E | E | A | E | A | A | **A** |
| xml | 0 / 2610 | E | E | A | E | A | A | E |

**aws-smithy-xml is disqualified through the public shape traversal used here.** It can be driven without generated code (`Document`, depth-first `next_start_element`, scoped `try_data`), but the public `XmlToken` hides the underlying token. `next_start_element`/scope traversal consume termination/error information. This adapter synthesizes closes from depth; the resulting silent truncation/mismatch acceptance is shown, not hidden. Published 0.64.1 contains `check_well_formedness` and `finish_well_formed`, but both are **pub(crate)**, not a documented public option we can enable. Its raw Document iterator can expose Result failures, but cannot supply public element/end/text events sufficient for this nonrecursive adapter without a second tokenizer pass, generated-code-style scoped traversal, or relying on Debug formatting. None is smuggled into timings. This disqualifies this public-API route, not every possible use of the crate. The coordinator checked the cached archive's SHA-256 against Cargo.lock (`590fdc1a70546b2f7f2061c42eaf173fb6f26d935330d069edc04dbd8f01d924`) and compared the extracted source: these internal methods are published code, not a local patch.

The shared layer rejects duplicate **known** members and duplicate retained map keys; unknown members are skipped and duplicate unknown names have no retained semantic effect. XML wrappers/entity framing are fixture-focused, not a complete XML Namespaces/DTD implementation. DTDs are disabled in the quick-xml/xmlparser adapters. Prefix resolution, all XML attribute constraints, mixed content, unusual timestamp formats, arbitrary document-number ranges, header/status bindings, and the seven-protocol AWS suite are not established by this benchmark. The baseline is a cost baseline, not a production JSON implementation recommendation.

## Chunk feeding, memory bound, dependencies and code liability

| Candidate | Incremental input capability | What was exercised |
|---|---|---|
| aws-smithy-json | Whole `&[u8]` token iterator; no resumable chunk API used | contiguous page |
| jiter | Whole `&[u8]`; incomplete-input modes are not a resumable socket stream | contiguous page; strict, no partial-string/NaN options |
| struson | `JsonStreamReader<R: Read>` can read incrementally | slice reader in timings; reader API inspected |
| serde_json | `Deserializer::from_reader` supports reader input | slice Deserializer + Seed/Visitor in timings |
| minimal | Whole slice; custom nonrecursive task stack | contiguous page |
| json-event-parser | Reader plus low-level `parse_next(buffer, is_ending)` | slice timings; **1-, 7-, 4096-byte Read chunks** checked against Jiter records |
| quick-xml | `Reader<R: BufRead>::read_event_into` | borrowed slice timings; **1-, 7-, 4096-byte reads** checked against Jiter records |
| xmlparser | `Tokenizer::from(&str)` needs contiguous valid UTF-8 | UTF-8 prevalidation included in timings |
| aws-smithy-xml | `Document::try_from(&[u8])` needs contiguous UTF-8 | UTF-8 conversion/scoped parsing included |
| xml | `EventReader<R: Read>` supports incremental reading | slice reader in timings |

A contiguous tokenizer requires up to **64 MiB of input per in-flight response**, before output/scratch; 4096 maximum-sized concurrent buffers alone would be **256 GiB**. This is a concurrency/memory-budget implication, not a measured production peak. Pull token streaming from a buffered page still parses JSON once and does not build a JSON tree; it does not satisfy a stronger socket-resume requirement. Incremental readers avoid requiring the whole wire page but must still bound individual tokens (the 1 MiB string test is intentional), nesting, and emitted output. Chunk-mode throughput/allocations were not measured, so its slice-mode numbers must not be advertised as socket throughput.

Normal transitive counts come from `cargo tree --locked -e normal --prefix none --format '{p}' -p <candidate>` in this standalone crate. Added counts include the candidate itself, subtract the existing aws-wire dependency closure in the worktree root Cargo.lock, and are contextual to this lock/feature unification, not universal counts. They exclude the separate future inventory-core/normalization integration cost. Shared dependencies of benchmark candidates can unify features; a standalone production dependency graph can differ. Own unsafe counts are **lines containing `unsafe` in the package's `src/**/*.rs`**, equivalent to summed `grep -c unsafe`: comments, attributes, tests, and feature-disabled source count too; this is not a count of executed unsafe blocks. The shared first-party code forbids unsafe.

| Candidate | Exact version | License | Own unsafe lines | Normal transitive packages | Added to existing aws-wire | Adapter physical lines |
|---|---|---|---:|---:|---:|---:|
| aws-smithy-json | 0.65.1 | Apache-2.0 | 2 | 34 | 6 | 22 |
| jiter | 0.19.0, no default features | MIT | 60 | 10 | 10 | 37 |
| struson | 0.8.0 | MIT OR Apache-2.0 | 0 | 6 | 2 | 33 |
| serde_json | 1.0.154 | MIT OR Apache-2.0 | 16 | 4 | 2 | 59 |
| minimal | local | MIT | 0 | 0 | 0 | 126 |
| json-event-parser | 0.2.3 | MIT OR Apache-2.0 | 5 | 0 | 1 | 40 |
| quick-xml | 0.42.0 | MIT | 10 | 1 | 1 | 83 shared XML adapter module |
| xmlparser | 0.13.6 | MIT/Apache-2.0 | 2 | 0 | 1 | 83 shared XML adapter module |
| aws-smithy-xml | 0.64.1 | Apache-2.0 | 0 | 35 | 7 | 26 |
| xml | 1.4.0 | MIT | 1 | 0 | 1 | 83 shared XML adapter module |

Common record/shape/scalar code is **172 lines** (87 sink + 44 shape + 41 scalar). XML additionally shares **106 lines** (83 bridge + 23 entities); the 83-line XML event module serves three candidates and includes a 16-line chunk reader. These counts are physical lines including comments/blank lines, not claims that all 83 lines belong to each individual XML candidate. Full source totals **1058 Rust lines**; no Rust file exceeds 87 lines. The adapters are intentionally small, but duplicated non-production traversal and evidence code remains a liability rather than a suggested product architecture.

No online issue search or release-date API was used: network was restricted to Cargo. Cargo metadata supplies version/license/dependency information, not a publication timestamp; normalized .crate tar timestamps are not release dates. Accordingly **release dates and current open soundness issues are unknown**, and no candidate was rejected on either basis. Local cached RustSec advisories RUSTSEC-2026-0194 and RUSTSEC-2026-0195 concern quick-xml duplicate-attribute CPU amplification and namespace-allocation amplification; both list **>=0.41.0 as patched**, so the tested 0.42.0 satisfies those advisory version constraints. That is not a comprehensive or freshly synchronized security audit, and Jiter's 60 unsafe-containing lines remain dependency-review risk. Pinning and these scenarios do not prove third-party soundness.

## Counters and bottleneck assessment

`perf stat` measured **200 full large-page record decodes** of each recommendation, pinned to CPU 7. These whole-process counters include one generator run/startup; they are not isolated tokenizer instructions and do not establish cache or inlining causes.

| Candidate | cycles:u | instructions:u | cache-misses:u | branches:u | branch-misses:u | elapsed s |
|---|---:|---:|---:|---:|---:|---:|
| jiter | 3366526829 | 12527778896 | 2724792 | 2529820662 | 3235556 | 0.791915414 |
| quick-xml | 7499102721 | 25988347946 | 6578866 | 5614274293 | 22221599 | 1.608099281 |

**Loopback HTTP comparison: unknown.** No transport benchmark was measured here; no other agent's transport number is cited. Measured decoding costs are about 4 µs JSON / 8 µs XML on small pages and 3.880 ms / 7.033 ms on large pages. Whether these are plausibly the bottleneck *relative to a loopback HTTP round trip* cannot be concluded without that round-trip measurement and realistic response-size/concurrency distributions. This evidence does not justify reviving page hashes (V33) or claiming transport is slower.

## Reproduction and executed commands

Run only in this standalone crate in the decoder-bench worktree. Cargo.lock is committed evidence; the root workspace is not built. Export the private target location, never the main checkout's shared target:

```bash
cd /home/lewis/src/aws-inventory-worktrees/decoder-bench/evidence/decoder-bench
export CARGO_TARGET_DIR=/home/lewis/src/aws-inventory-worktrees/decoder-bench/evidence/decoder-bench/target
cargo run --release -j 8
# Exact final table run (same command, CPU pinned and output retained locally):
taskset -c 7 cargo run --release -j 8 > target/run-final.txt 2> target/build-final.txt
cargo clippy -j 8 --all-targets -- -D warnings
cargo run --release -j 8 -- --scenarios
# Optional focused evidence lanes:
cargo run --release -j 8 -- --metadata
cargo run --release -j 8 -- jiter --adversarial
perf stat -e cycles,instructions,cache-misses,branches,branch-misses -- taskset -c 7 target/release/decoder-bench jiter --profile
perf stat -e cycles,instructions,cache-misses,branches,branch-misses -- taskset -c 7 target/release/decoder-bench quick-xml --profile
```

**Actually executed:** Cargo search/info for the named libraries (including xml-rs → xml), git show of the prior codec RESULTS/main, lscpu, free -b, uname -a, rustc --version, toolchain cp, cargo run --release -j 8 (initial compile failures, repaired; then an initial complete run), cargo clippy -j 8 --all-targets -- -D warnings (initial warnings, repaired), focused `cargo run --release -j 8 -- --scenarios` (found and repaired Serde error erasure; then PASS), the exact pinned final run above (exit 0), both actual decoder perf commands above (exit 0), and final strict clippy (exit 0). Cargo metadata/tree commands are executed by the binary and succeed as part of the final run. A preliminary `perf stat ... -- true` established counter access only and is not decoder evidence. Initial optional checksum-file inspection found no .cargo-checksum.json; no registry source was changed.

Final scenario output:

```text
Given/When/Then: byte parity + typed fields; depth-64 retained document limit; chunked reader parity: PASS
```

Final clippy output:

```text
    Checking decoder-bench v0.1.0 (/home/lewis/src/aws-inventory-worktrees/decoder-bench/evidence/decoder-bench)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.17s
```

Not run: workspace builds/tests, cargo fmt, cargo test/nextest, mutation/fuzz/formal tools, Miri, audit/deny/vet/geiger, assembly/IR, API compatibility/SBOM, or loopback HTTP. They are outside this standalone evidence acceptance or require separate assurance; no passes are claimed. The executable scenarios run before every default benchmark and enforce the relevant behavior without a separate test harness. Output tables are reproducible structurally, not bit-for-bit timing promises.

## Scope and assurance notes

Read VISION V03/V21/V34, wire engine 5.2, runtime targets 8 and ROADMAP M2.4a–d/M3.5. Reference files read before implementation: `holzman-rust/references/latency-throughput-playbook.md`, `runtime-performance-architecture.md`, `zero-cost-abstractions.md`, `nasa-jpl-standards.md`; Rust corpus `BEST_PRACTICES.md`; relevant skill instructions for exploration, corpus grounding and BDD.

Power of Ten: explicit parser/task stacks and input-bounded iteration, checked arithmetic/access, short functions, checked Results, static candidate dispatch, and zero first-party unsafe/panic/unwrap/lint suppressions. The requested Serde Visitor candidate necessarily uses bounded nested deserialization (depth <=96), unlike the nonrecursive recommended adapters: it is not strict recursion-free Rule 1 code. Rule 3's absolute no-post-init-allocation is deliberately not claimed: this performance evidence counts library/item/record allocations instead. Actual compiler/clippy and scenario evidence is given above; no general panic-freedom, allocator-failure, third-party-soundness, or XML-conformance proof is asserted.

Files written (all under this directory; physical line counts): .gitignore 1, Cargo.toml 43, Cargo.lock 681, rust-toolchain.toml 6; src/adversarial.rs 55, candidate.rs 27, entities.rs 23, error.rs 20, json_event.rs 40, json_jiter.rs 37, json_minimal.rs 40, json_serde.rs 59, json_smithy.rs 22, json_struson.rs 33, main.rs 43, measure.rs 44, metadata.rs 74, minimal_lex.rs 86, sample.rs 24, scalar.rs 41, scenarios.rs 67, shape.rs 44, sink.rs 87, xml_bridge.rs 83, xml_events.rs 83, xml_smithy.rs 26; RESULTS.md 237. No product files changed, no commit/push performed.
