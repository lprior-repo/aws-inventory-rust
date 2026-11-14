 
 
 
 

use std::hint::black_box;
use std::time::Instant;

use inventory_core::record::{FieldId, Float, Record, Time, Value, ValueRef};

mod sample;

 
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
            let ValueRef::Struct(inner) = value else {
                panic!()
            };
            value = inner.get(FieldId::new(*b).unwrap()).unwrap();
        }
        let ValueRef::Text(text) = value else {
            panic!()
        };
        sum += text.len();
    }
    sum
}

include!("codecs.rs");

 

 
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
    let find =
        |fields: &[(&'static str, S)], name: &str| fields.iter().position(|f| f.0 == name).unwrap();
    names
        .iter()
        .map(|(a, b)| {
            let at = find(fields, a);
            let nested = b.map(|b| {
                let S::Struct(inner) = &fields[at].1 else {
                    panic!()
                };
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
    let (custom, archived, bits, json) = (
        to_custom(root),
        to_rkyv(root),
        to_bitcode(root),
        to_json(root),
    );

    let ours = Record::encode(&custom).unwrap().into_bytes();
    let cbor = cbor_encode(&custom);
    let rk = rkyv::to_bytes::<rkyv::rancor::Error>(&archived)
        .unwrap()
        .to_vec();
    let bc = postcard::to_allocvec(&bits).unwrap();
    let js = serde_json::to_vec(&json).unwrap();

    let want = custom_read(&ours, &paths);
    assert_eq!(cbor_read(&cbor, &paths, true), want);
    assert_eq!(rkyv_read(&rk, &paths), want);
    assert_eq!(bitcode_read(&bc, &paths), want);
    assert_eq!(json_read(&js, &paths), want);

    println!("\n### {title}\n");
    println!(
        "| Codec | Bytes | Build ns | Hash ns | Build + hash ns | Open + 5 fields ns | Read path |"
    );
    println!("|---|---|---|---|---|---|---|");
    row(
        "Our format",
        &ours,
        time(|| Record::encode(black_box(&custom)).unwrap()),
        time(|| custom_read(black_box(&ours), &paths)),
        "copy, validate all, seek",
    );
    row(
        "CBOR (minicbor)",
        &cbor,
        time(|| cbor_encode(black_box(&custom))),
        time(|| cbor_read(black_box(&cbor), &paths, true)),
        "walk all once, seek",
    );
    row(
        "CBOR, no pre-walk",
        &cbor,
        time(|| cbor_encode(black_box(&custom))),
        time(|| cbor_read(black_box(&cbor), &paths, false)),
        "seek only",
    );
    row(
        "rkyv",
        &rk,
        time(|| rkyv::to_bytes::<rkyv::rancor::Error>(black_box(&archived)).unwrap()),
        time(|| rkyv_read(black_box(&rk), &paths)),
        "aligned copy, validate all, index",
    );
    row(
        "postcard",
        &bc,
        time(|| postcard::to_allocvec(black_box(&bits)).unwrap()),
        time(|| bitcode_read(black_box(&bc), &paths)),
        "decode whole tree",
    );
    row(
        "serde_json",
        &js,
        time(|| serde_json::to_vec(black_box(&json)).unwrap()),
        time(|| json_read(black_box(&js), &paths)),
        "parse whole tree",
    );
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
