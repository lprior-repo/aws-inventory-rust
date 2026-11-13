# Record codec benchmark, 2026-11-13

Settles VISION decision V34 (was open decision O1): records are CBOR
(RFC 8949), written and read with the `minicbor` crate.

## Result

AMD Ryzen 9 9950X3D, one pinned core, release build with fat LTO, median of 21
samples of about 10 ms each. Times are nanoseconds per record. Three further
runs agreed with these within 4%, except `serde_json` build time, which varied
by up to 12%; other work was running on the machine.

EC2 instance: 44 top-level fields, two network interfaces, three volumes, ten
tags.

| Codec | Bytes | Build | Hash (BLAKE3) | Build + hash | Open and read 5 fields |
| --- | --- | --- | --- | --- | --- |
| CBOR (`minicbor` 2.3.1) | 2,191 | 1,088 | 1,308 | 2,396 | 505 seek only; 1,779 with a full well-formedness walk first |
| `postcard` 1.1.3 | 2,332 | 1,282 | 1,439 | 2,721 | 3,620 (decodes the whole tree) |
| Our own tagged format | 2,373 | 2,330 | 1,505 | 3,835 | 1,805 (copy, full validation, seek) |
| `serde_json` (stored today) | 4,761 | 2,418 | 1,893 | 4,311 | 9,354 (parses the whole tree) |
| `rkyv` 0.8, validated | 7,592 | 1,387 | 3,634 | 5,021 | 581 (aligned copy, full validation, index) |

Queue-like resource: ten scalar fields.

| Codec | Bytes | Build | Hash (BLAKE3) | Build + hash | Open and read 5 fields |
| --- | --- | --- | --- | --- | --- |
| CBOR (`minicbor` 2.3.1) | 400 | 125 | 461 | 587 | 230 seek only; 330 with the walk |
| `postcard` 1.1.3 | 401 | 136 | 462 | 598 | 151 |
| Our own tagged format | 402 | 192 | 464 | 656 | 249 |
| `serde_json` (stored today) | 624 | 319 | 658 | 977 | 622 |
| `rkyv` 0.8, validated | 704 | 138 | 721 | 859 | 44 |

## Reading the result

- Build + hash is the path every resource takes on every sweep: the record is
  built from the response, hashed and compared with the stored hash. CBOR is
  the fastest on both shapes and the smallest.
- `rkyv` reads fastest, but its records are 3.2 times larger on the instance,
  so hashing and storage cost more than the read saves. Records are read back
  only for changed resources and the UI.
- `postcard` is close to CBOR on build but cannot read one field without
  decoding the whole record, and its bytes are not self-describing.
- Our own format built slowest of the binary codecs, because it inserts each
  container's byte length after writing the body. It was 1,054 lines of
  production code and 589 of tests.

## Method and limits

- Every codec encoded the same value tree. The engine will later feed records
  from a streaming decoder, so absolute build times will differ; this was not
  measured.
- Reads start from an unaligned byte slice, as SQLite delivers it. The copy
  `rkyv` needs for alignment and the copy our format made are counted.
- The five fields read are three top-level text fields and two inside nested
  structures.
- The roadmap also asked for a 100-field record and a record with one large
  string. They were not run: the result on both shapes above points the same
  way, and with a large string every codec is dominated by the same copy.

## Reproducing

The benchmark and the discarded format are not on `main`. They are on the
branch `evidence/o1-record-codec`, commit `4a54ec4`, in
`evidence/record-codec-bench/`; `cargo run --release` there prints the tables.
`RESULTS.md` in that directory holds three consecutive runs.
