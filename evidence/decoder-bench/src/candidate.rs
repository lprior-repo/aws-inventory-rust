use crate::{
    error::{Error, Result},
    sink::{Mode, Sink},
};
#[derive(Clone, Copy, Debug)]
pub enum Candidate {
    SmithyJson,
    Jiter,
    Struson,
    Serde,
    Minimal,
    Event,
    Quick,
    Xmlparser,
    SmithyXml,
    Xml,
}
pub const ALL: &[Candidate] = &[
    Candidate::SmithyJson,
    Candidate::Jiter,
    Candidate::Struson,
    Candidate::Serde,
    Candidate::Minimal,
    Candidate::Event,
    Candidate::Quick,
    Candidate::Xmlparser,
    Candidate::SmithyXml,
    Candidate::Xml,
];
impl Candidate {
    pub fn name(self) -> &'static str {
        match self {
            Self::SmithyJson => "aws-smithy-json",
            Self::Jiter => "jiter",
            Self::Struson => "struson",
            Self::Serde => "serde_json",
            Self::Minimal => "minimal",
            Self::Event => "json-event-parser",
            Self::Quick => "quick-xml",
            Self::Xmlparser => "xmlparser",
            Self::SmithyXml => "aws-smithy-xml",
            Self::Xml => "xml",
        }
    }
    pub fn xml(self) -> bool {
        matches!(
            self,
            Self::Quick | Self::Xmlparser | Self::SmithyXml | Self::Xml
        )
    }
    pub fn decode(self, input: &[u8], mode: Mode) -> Result<Sink> {
        if input.len() > 67_108_864 {
            return Err(Error::Bound);
        }
        let mut sink = Sink::new(mode);
        match self {
            Self::SmithyJson => crate::json_smithy::parse(input, &mut sink),
            Self::Jiter => crate::json_jiter::parse(input, &mut sink),
            Self::Struson => crate::json_struson::parse(input, &mut sink),
            Self::Serde => crate::json_serde::parse(input, &mut sink),
            Self::Minimal => crate::json_minimal::parse(input, &mut sink),
            Self::Event => crate::json_event::parse(input, &mut sink),
            Self::Quick => crate::xml_events::quick(input, &mut sink),
            Self::Xmlparser => crate::xml_events::raw(input, &mut sink),
            Self::SmithyXml => crate::xml_smithy::parse(input, &mut sink),
            Self::Xml => crate::xml_events::xml_rs(input, &mut sink),
        }?;
        sink.finish()
    }
}
