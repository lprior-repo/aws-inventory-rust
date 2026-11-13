//! Throwaway benchmark: which binary record codec is fastest for one resource.
//!
//! Every codec encodes the same tree, then the same five text fields are read
//! back from an unaligned byte slice, as SQLite would deliver it.

use std::hint::black_box;
use std::time::Instant;

use inventory_core::record::{FieldId, Float, Record, Time, Value, ValueRef};

mod sample;

/// The neutral tree every codec's native tree is built from.
#[derive(Clone)]
pub enum S {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Time(i64, u32),
    Text(String),
    Bytes(Vec<u8>),
    List(Vec<S>),
    Map(Vec<(String, S)>),
    Struct(Vec<(&'static str, S)>),
}

type Path = (u32, &'static str, Option<(u32, &'static str)>);

fn id(index: usize) -> u32 {
    u32::try_from(index + 1).unwrap()
}

// ---------- our format ----------

fn to_custom(s: &S) -> Value {
    match s {
        S::Null => Value::Null,
        S::Bool(b) => Value::Bool(*b),
        S::Int(i) => Value::Int(*i),
        S::Float(f) => Value::Float(Float::from(*f)),
        S::Time(a, b) => Value::Time(Time::new(*a, *b).unwrap()),
        S::Text(t) => Value::Text(t.clone()),
        S::Bytes(b) => Value::Bytes(b.clone()),
        S::List(l) => Value::List(l.iter().map(to_custom).collect()),
        S::Map(m) => Value::Map(m.iter().map(|(k, v)| (k.clone(), to_custom(v))).collect()),
        S::Struct(f) => Value::Struct(
            f.iter()
                .enumerate()
                .map(|(i, (_, v))| (FieldId::new(id(i)).unwrap(), to_custom(v)))
                .collect(),
        ),
    }
}

fn custom_read(bytes: &[u8], paths: &[Path]) -> usize {
    let record = Record::from_bytes(bytes.to_vec()).unwrap();
    let root = record.root();
    let mut sum = 0;
    for (a, _, nested) in paths {
        let mut value = root.get(FieldId::new(*a).unwrap()).unwrap();
        if let Some((b, _)) = nested {
            let ValueRef::Struct(inner) = value else { panic!() };
            value = inner.get(FieldId::new(*b).unwrap()).unwrap();
        }
        let ValueRef::Text(text) = value else { panic!() };
        sum += text.len();
    }
    sum
}

// ---------- CBOR through minicbor ----------

type Enc = minicbor::Encoder<Vec<u8>>;

fn cbor_write(e: &mut Enc, v: &Value) {
    match v {
        Value::Null => drop(e.null().unwrap()),
        Value::Bool(b) => drop(e.bool(*b).unwrap()),
        Value::Int(i) => drop(e.i64(*i).unwrap()),
        Value::Float(f) => drop(e.f64(f.get()).unwrap()),
        Value::Time(t) => {
            e.tag(minicbor::data::Tag::new(1)).unwrap();
            e.array(2).unwrap().i64(t.seconds()).unwrap().u32(t.nanos()).unwrap();
        }
        Value::Text(t) => drop(e.str(t).unwrap()),
        Value::Bytes(b) => drop(e.bytes(b).unwrap()),
        Value::List(l) => {
            e.array(l.len() as u64).unwrap();
            l.iter().for_each(|x| cbor_write(e, x));
        }
        Value::Map(m) => {
            e.map(m.len() as u64).unwrap();
            for (k, x) in m {
                e.str(k).unwrap();
                cbor_write(e, x);
            }
        }
        Value::Struct(m) => {
            e.map(m.len() as u64).unwrap();
            for (k, x) in m {
                e.u32(k.get()).unwrap();
                cbor_write(e, x);
            }
        }
    }
}

fn cbor_encode(v: &Value) -> Vec<u8> {
    let mut e = minicbor::Encoder::new(Vec::new());
    cbor_write(&mut e, v);
    e.into_writer()
}

/// Leaves the decoder at the value of field `want` of the struct it is on.
fn cbor_field(d: &mut minicbor::Decoder<'_>, want: u32) {
    let count = d.map().unwrap().unwrap();
    for _ in 0..count {
        let key = d.u32().unwrap();
        if key == want {
            return;
        }
        assert!(key < want);
        d.skip().unwrap();
    }
    panic!("field absent");
}

fn cbor_read(bytes: &[u8], paths: &[Path], validate: bool) -> usize {
    if validate {
        let mut d = minicbor::Decoder::new(bytes);
        d.skip().unwrap();
        assert_eq!(d.position(), bytes.len());
    }
    let mut sum = 0;
    for (a, _, nested) in paths {
        let mut d = minicbor::Decoder::new(bytes);
        cbor_field(&mut d, *a);
        if let Some((b, _)) = nested {
            cbor_field(&mut d, *b);
        }
        sum += d.str().unwrap().len();
    }
    sum
}

// ---------- rkyv with validation ----------

#[derive(rkyv::Archive, rkyv::Serialize)]
#[rkyv(serialize_bounds(
    __S: rkyv::ser::Writer + rkyv::ser::Allocator,
    __S::Error: rkyv::rancor::Source,
))]
#[rkyv(bytecheck(bounds(__C: rkyv::validation::ArchiveContext, __C::Error: rkyv::rancor::Source)))]
enum R {
    Null,
    Bool(bool),
    Int(i64),
    Float(u64),
    Time(i64, u32),
    Text(String),
    Bytes(Vec<u8>),
    List(#[rkyv(omit_bounds)] Vec<R>),
    Map(#[rkyv(omit_bounds)] Vec<(String, R)>),
    Struct(#[rkyv(omit_bounds)] Vec<(u32, R)>),
}

fn to_rkyv(s: &S) -> R {
    match s {
        S::Null => R::Null,
        S::Bool(b) => R::Bool(*b),
        S::Int(i) => R::Int(*i),
        S::Float(f) => R::Float(f.to_bits()),
        S::Time(a, b) => R::Time(*a, *b),
        S::Text(t) => R::Text(t.clone()),
        S::Bytes(b) => R::Bytes(b.clone()),
        S::List(l) => R::List(l.iter().map(to_rkyv).collect()),
        S::Map(m) => R::Map(m.iter().map(|(k, v)| (k.clone(), to_rkyv(v))).collect()),
        S::Struct(f) => R::Struct(f.iter().enumerate().map(|(i, (_, v))| (id(i), to_rkyv(v))).collect()),
    }
}

fn rkyv_field(v: &ArchivedR, want: u32) -> &ArchivedR {
    let ArchivedR::Struct(fields) = v else { panic!() };
    &fields.iter().find(|entry| entry.0.to_native() == want).unwrap().1
}

fn rkyv_read(bytes: &[u8], paths: &[Path]) -> usize {
    let mut aligned = rkyv::util::AlignedVec::<16>::with_capacity(bytes.len());
    aligned.extend_from_slice(bytes);
    let root = rkyv::access::<ArchivedR, rkyv::rancor::Error>(&aligned).unwrap();
    let mut sum = 0;
    for (a, _, nested) in paths {
        let mut value = rkyv_field(root, *a);
        if let Some((b, _)) = nested {
            value = rkyv_field(value, *b);
        }
        let ArchivedR::Text(text) = value else { panic!() };
        sum += text.len();
    }
    sum
}

// ---------- postcard (full decode) ----------

#[derive(serde::Serialize, serde::Deserialize)]
enum B {
    Null,
    Bool(bool),
    Int(i64),
    Float(u64),
    Time(i64, u32),
    Text(String),
    Bytes(Vec<u8>),
    List(Vec<B>),
    Map(Vec<(String, B)>),
    Struct(Vec<(u32, B)>),
}

fn to_bitcode(s: &S) -> B {
    match s {
        S::Null => B::Null,
        S::Bool(b) => B::Bool(*b),
        S::Int(i) => B::Int(*i),
        S::Float(f) => B::Float(f.to_bits()),
        S::Time(a, b) => B::Time(*a, *b),
        S::Text(t) => B::Text(t.clone()),
        S::Bytes(b) => B::Bytes(b.clone()),
        S::List(l) => B::List(l.iter().map(to_bitcode).collect()),
        S::Map(m) => B::Map(m.iter().map(|(k, v)| (k.clone(), to_bitcode(v))).collect()),
        S::Struct(f) => B::Struct(f.iter().enumerate().map(|(i, (_, v))| (id(i), to_bitcode(v))).collect()),
    }
}

fn bitcode_field(v: &B, want: u32) -> &B {
    let B::Struct(fields) = v else { panic!() };
    &fields.iter().find(|entry| entry.0 == want).unwrap().1
}

fn bitcode_read(bytes: &[u8], paths: &[Path]) -> usize {
    let root: B = postcard::from_bytes(bytes).unwrap();
    let mut sum = 0;
    for (a, _, nested) in paths {
        let mut value = bitcode_field(&root, *a);
        if let Some((b, _)) = nested {
            value = bitcode_field(value, *b);
        }
        let B::Text(text) = value else { panic!() };
        sum += text.len();
    }
    sum
}

// ---------- JSON, as the tree stores today ----------

fn to_json(s: &S) -> serde_json::Value {
    use serde_json::Value as J;
    match s {
        S::Null => J::Null,
        S::Bool(b) => J::Bool(*b),
        S::Int(i) => J::from(*i),
        S::Float(f) => J::from(*f),
        S::Time(a, b) => J::String(format!("{a}.{b:09}")),
        S::Text(t) => J::String(t.clone()),
        S::Bytes(b) => J::String(b.iter().map(|x| format!("{x:02x}")).collect()),
        S::List(l) => J::Array(l.iter().map(to_json).collect()),
        S::Map(m) => J::Object(m.iter().map(|(k, v)| (k.clone(), to_json(v))).collect()),
        S::Struct(f) => J::Object(f.iter().map(|(k, v)| ((*k).to_owned(), to_json(v))).collect()),
    }
}

fn json_read(bytes: &[u8], paths: &[Path]) -> usize {
    let root: serde_json::Value = serde_json::from_slice(bytes).unwrap();
    let mut sum = 0;
    for (_, a, nested) in paths {
        let mut value = &root[*a];
        if let Some((_, b)) = nested {
            value = &value[*b];
        }
        sum += value.as_str().unwrap().len();
    }
    sum
}

// ---------- harness ----------

/// Median nanoseconds per call over 21 samples of about 10 ms each.
fn time<T>(mut f: impl FnMut() -> T) -> f64 {
    let mut iterations = 1u64;
    loop {
        let start = Instant::now();
        for _ in 0..iterations {
            black_box(f());
        }
        if start.elapsed().as_millis() >= 10 {
            break;
        }
        iterations *= 2;
    }
    let mut samples: Vec<f64> = (0..21)
        .map(|_| {
            let start = Instant::now();
            for _ in 0..iterations {
                black_box(f());
            }
            start.elapsed().as_nanos() as f64 / iterations as f64
        })
        .collect();
    samples.sort_by(f64::total_cmp);
    samples[10]
}

fn resolve(root: &S, names: &[(&'static str, Option<&'static str>)]) -> Vec<Path> {
    let S::Struct(fields) = root else { panic!() };
    let find = |fields: &[(&'static str, S)], name: &str| fields.iter().position(|f| f.0 == name).unwrap();
    names
        .iter()
        .map(|(a, b)| {
            let at = find(fields, a);
            let nested = b.map(|b| {
                let S::Struct(inner) = &fields[at].1 else { panic!() };
                (id(find(inner, b)), b)
            });
            (id(at), *a, nested)
        })
        .collect()
}

fn row(name: &str, bytes: &[u8], encode: f64, read: f64, note: &str) {
    let hash = time(|| blake3::hash(black_box(bytes)));
    println!(
        "| {name} | {} | {encode:.0} | {hash:.0} | {:.0} | {read:.0} | {note} |",
        bytes.len(),
        encode + hash
    );
}

fn run(title: &str, root: &S, names: &[(&'static str, Option<&'static str>)]) {
    let paths = resolve(root, names);
    let (custom, archived, bits, json) = (to_custom(root), to_rkyv(root), to_bitcode(root), to_json(root));

    let ours = Record::encode(&custom).unwrap().into_bytes();
    let cbor = cbor_encode(&custom);
    let rk = rkyv::to_bytes::<rkyv::rancor::Error>(&archived).unwrap().to_vec();
    let bc = postcard::to_allocvec(&bits).unwrap();
    let js = serde_json::to_vec(&json).unwrap();

    let want = custom_read(&ours, &paths);
    assert_eq!(cbor_read(&cbor, &paths, true), want);
    assert_eq!(rkyv_read(&rk, &paths), want);
    assert_eq!(bitcode_read(&bc, &paths), want);
    assert_eq!(json_read(&js, &paths), want);

    println!("\n### {title}\n");
    println!("| Codec | Bytes | Build ns | Hash ns | Build + hash ns | Open + 5 fields ns | Read path |");
    println!("|---|---|---|---|---|---|---|");
    row("Our format", &ours, time(|| Record::encode(black_box(&custom)).unwrap()), time(|| custom_read(black_box(&ours), &paths)), "copy, validate all, seek");
    row("CBOR (minicbor)", &cbor, time(|| cbor_encode(black_box(&custom))), time(|| cbor_read(black_box(&cbor), &paths, true)), "walk all once, seek");
    row("CBOR, no pre-walk", &cbor, time(|| cbor_encode(black_box(&custom))), time(|| cbor_read(black_box(&cbor), &paths, false)), "seek only");
    row("rkyv", &rk, time(|| rkyv::to_bytes::<rkyv::rancor::Error>(black_box(&archived)).unwrap()), time(|| rkyv_read(black_box(&rk), &paths)), "aligned copy, validate all, index");
    row("postcard", &bc, time(|| postcard::to_allocvec(black_box(&bits)).unwrap()), time(|| bitcode_read(black_box(&bc), &paths)), "decode whole tree");
    row("serde_json", &js, time(|| serde_json::to_vec(black_box(&json)).unwrap()), time(|| json_read(black_box(&js), &paths)), "parse whole tree");
}

fn main() {
    run(
        "EC2 instance (large, nested)",
        &sample::instance(),
        &[
            ("InstanceId", None),
            ("PrivateDnsName", None),
            ("State", Some("Name")),
            ("Placement", Some("AvailabilityZone")),
            ("VpcId", None),
        ],
    );
    run(
        "Small resource (10 fields)",
        &sample::small(),
        &[
            ("QueueArn", None),
            ("QueueUrl", None),
            ("KmsMasterKeyId", None),
            ("Name", None),
            ("Policy", None),
        ],
    );
}
