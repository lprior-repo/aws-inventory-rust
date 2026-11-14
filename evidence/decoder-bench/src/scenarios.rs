use crate::{
    candidate::{ALL, Candidate},
    error::{Error, Result},
    sample,
    sink::{Mode, Sink},
};
use inventory_core::record::{FieldId, MapRef, RecordError, ValueRef};
pub fn run() -> Result<()> {
     
     
    let sample = sample::generate(2, 450)?;
    crate::measure::equivalence(ALL, &sample)?;
    ALL.iter().try_for_each(|&c| {
        let input = if c.xml() { &sample.xml } else { &sample.json };
        let output = c.decode(input, Mode::Records)?;
        let first = output
            .records
            .first()
            .ok_or(Error::Shape("scenario missing record"))?;
        fields(MapRef::from_bytes(first)?)
    })?;
    document()?;
    chunks(&sample)?;
    println!(
        "Given/When/Then: byte parity + typed fields; depth-64 retained document limit; chunked reader parity: PASS"
    );
    Ok(())
}
fn fields(root: MapRef<'_>) -> Result<()> {
    let field = |id| root.get_field(FieldId::new(id)?);
    let valid = matches!(field(1)?, Some(ValueRef::Text("i-00000000000000000")))
        && matches!(field(6)?, Some(ValueRef::Int(8)))
        && matches!(field(8)?, Some(ValueRef::Int(9_223_372_036_854_775_806)))
        && matches!(field(9)?, Some(ValueRef::Bool(true)))
        && matches!(field(10)?, Some(ValueRef::Null))
        && matches!(field(11)?, Some(ValueRef::Time(t)) if t.seconds() == 1_735_689_600 && t.nanoseconds() == 125_000_000)
        && matches!(field(12)?, Some(ValueRef::Bytes([0, 1, 2, 253, 254, 255])))
        && matches!(field(5)?, Some(ValueRef::Text(s)) if s.contains("quote\" slash\\ line\n snowman☃ rocket🚀"));
    if !valid {
        return Err(Error::Shape("typed field scenario"));
    }
    Ok(())
}
fn document() -> Result<()> {
     
    let input = format!(
        "{{\"Items\":[{{\"Document\":{}0{}}}]}}",
        "[".repeat(64),
        "]".repeat(64)
    );
    ALL.iter().filter(|c| !c.xml()).try_for_each(|c| {
        c.decode(input.as_bytes(), Mode::Tokens)?;
        match c.decode(input.as_bytes(), Mode::Records) {
            Err(Error::Record(RecordError::DepthLimit)) => Ok(()),
            Err(error) => Err(Error::Syntax(format!("{} depth64: {error}", c.name()))),
            Ok(_) => Err(Error::Syntax(format!(
                "{} accepted retained depth64",
                c.name()
            ))),
        }
    })
}
struct Chunked<'a> {
    input: &'a [u8],
    size: usize,
}
impl std::io::Read for Chunked<'_> {
    fn read(&mut self, out: &mut [u8]) -> std::io::Result<usize> {
        let len = out.len().min(self.size).min(self.input.len());
        let source = self
            .input
            .get(..len)
            .ok_or_else(|| std::io::Error::other("chunk source"))?;
        out.get_mut(..len)
            .ok_or_else(|| std::io::Error::other("chunk destination"))?
            .copy_from_slice(source);
        self.input = self
            .input
            .get(len..)
            .ok_or_else(|| std::io::Error::other("chunk remainder"))?;
        Ok(len)
    }
}
fn chunks(sample: &sample::Sample) -> Result<()> {
     
     
    let expected = Candidate::Jiter.decode(&sample.json, Mode::Records)?;
    [1, 7, 4096].into_iter().try_for_each(|size| {
        let mut sink = Sink::new(Mode::Records);
        crate::xml_events::quick_reader(
            std::io::BufReader::new(Chunked {
                input: &sample.xml,
                size,
            }),
            &mut sink,
            sample.xml.len(),
        )?;
        if sink.finish()?.records != expected.records {
            return Err(Error::Shape("XML chunk parity"));
        }
        let mut sink = Sink::new(Mode::Records);
        crate::json_event::parse_reader(
            Chunked {
                input: &sample.json,
                size,
            },
            &mut sink,
            sample.json.len(),
        )?;
        if sink.finish()?.records != expected.records {
            return Err(Error::Shape("JSON chunk parity"));
        }
        Ok(())
    })
}
