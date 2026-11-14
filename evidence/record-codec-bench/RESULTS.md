# Record codec benchmark, 2026-11-13

Historical O1 experiment: `inventory-core` is pinned to
`4a54ec49edc36a772126b93f151db9ad4409c992`, which contains the superseded
custom format. Production now uses minicbor-backed CBOR. The original rows
below are not measurements of the current production record implementation.

The benchmark source was split to satisfy the repository's 300-line file
limit. Its original source and dependency lock remain available at that Git
revision; the tables below have not been regenerated.

Machine: AMD Ryzen 9 9950X3D, one pinned core (`taskset -c 7`), release, fat LTO, median of 21 samples of about 10 ms each. Other work was running on the machine.

Run: `cargo run --release --locked` in this directory. Three original consecutive runs follow.

## Run 1

### EC2 instance (large, nested)

| Codec | Bytes | Build ns | Hash ns | Build + hash ns | Open + 5 fields ns | Read path |
|---|---|---|---|---|---|---|
| Our format | 2373 | 2333 | 1495 | 3828 | 1809 | copy, validate all, seek |
| CBOR (minicbor) | 2191 | 1098 | 1303 | 2400 | 1812 | walk all once, seek |
| CBOR, no pre-walk | 2191 | 1080 | 1302 | 2382 | 496 | seek only |
| rkyv | 7592 | 1340 | 3611 | 4951 | 580 | aligned copy, validate all, index |
| postcard | 2332 | 1234 | 1426 | 2659 | 3582 | decode whole tree |
| serde_json | 4761 | 2635 | 1881 | 4517 | 9317 | parse whole tree |

### Small resource (10 fields)

| Codec | Bytes | Build ns | Hash ns | Build + hash ns | Open + 5 fields ns | Read path |
|---|---|---|---|---|---|---|
| Our format | 402 | 191 | 459 | 651 | 246 | copy, validate all, seek |
| CBOR (minicbor) | 400 | 125 | 458 | 583 | 329 | walk all once, seek |
| CBOR, no pre-walk | 400 | 126 | 459 | 585 | 234 | seek only |
| rkyv | 704 | 137 | 720 | 858 | 45 | aligned copy, validate all, index |
| postcard | 401 | 137 | 460 | 597 | 151 | decode whole tree |
| serde_json | 624 | 329 | 655 | 983 | 613 | parse whole tree |

## Run 2

### EC2 instance (large, nested)

| Codec | Bytes | Build ns | Hash ns | Build + hash ns | Open + 5 fields ns | Read path |
|---|---|---|---|---|---|---|
| Our format | 2373 | 2325 | 1494 | 3819 | 1792 | copy, validate all, seek |
| CBOR (minicbor) | 2191 | 1085 | 1300 | 2385 | 1857 | walk all once, seek |
| CBOR, no pre-walk | 2191 | 1081 | 1301 | 2382 | 507 | seek only |
| rkyv | 7592 | 1349 | 3623 | 4971 | 588 | aligned copy, validate all, index |
| postcard | 2332 | 1306 | 1438 | 2745 | 3575 | decode whole tree |
| serde_json | 4761 | 2456 | 1876 | 4332 | 9378 | parse whole tree |

### Small resource (10 fields)

| Codec | Bytes | Build ns | Hash ns | Build + hash ns | Open + 5 fields ns | Read path |
|---|---|---|---|---|---|---|
| Our format | 402 | 194 | 457 | 651 | 251 | copy, validate all, seek |
| CBOR (minicbor) | 400 | 125 | 459 | 583 | 328 | walk all once, seek |
| CBOR, no pre-walk | 400 | 124 | 459 | 583 | 232 | seek only |
| rkyv | 704 | 137 | 717 | 854 | 44 | aligned copy, validate all, index |
| postcard | 401 | 138 | 457 | 595 | 150 | decode whole tree |
| serde_json | 624 | 360 | 654 | 1014 | 614 | parse whole tree |

## Run 3

### EC2 instance (large, nested)

| Codec | Bytes | Build ns | Hash ns | Build + hash ns | Open + 5 fields ns | Read path |
|---|---|---|---|---|---|---|
| Our format | 2373 | 2316 | 1488 | 3805 | 1776 | copy, validate all, seek |
| CBOR (minicbor) | 2191 | 1074 | 1302 | 2376 | 1801 | walk all once, seek |
| CBOR, no pre-walk | 2191 | 1076 | 1327 | 2403 | 495 | seek only |
| rkyv | 7592 | 1356 | 3683 | 5039 | 585 | aligned copy, validate all, index |
| postcard | 2332 | 1242 | 1426 | 2668 | 3580 | decode whole tree |
| serde_json | 4761 | 2709 | 1884 | 4593 | 9386 | parse whole tree |

### Small resource (10 fields)

| Codec | Bytes | Build ns | Hash ns | Build + hash ns | Open + 5 fields ns | Read path |
|---|---|---|---|---|---|---|
| Our format | 402 | 189 | 458 | 646 | 248 | copy, validate all, seek |
| CBOR (minicbor) | 400 | 125 | 459 | 584 | 349 | walk all once, seek |
| CBOR, no pre-walk | 400 | 125 | 455 | 580 | 233 | seek only |
| rkyv | 704 | 139 | 714 | 853 | 44 | aligned copy, validate all, index |
| postcard | 401 | 134 | 460 | 594 | 146 | decode whole tree |
| serde_json | 624 | 356 | 654 | 1010 | 624 | parse whole tree |
