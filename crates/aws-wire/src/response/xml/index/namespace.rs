use super::{DecodeError, Document, Token};
use inventory_core::catalog::XmlNamespace;
use quick_xml::{XmlVersion, events::BytesStart};

#[derive(Clone, Copy)]
enum Kind {
    Element,
    Attribute,
}

struct Query<'a> {
    prefix: &'a [u8],
    expected: &'a str,
    kind: Kind,
}

impl Document<'_> {
    pub fn validate_namespace(
        &self,
        node: usize,
        expected: Option<XmlNamespace<'_>>,
    ) -> Result<(), DecodeError> {
        let Some(expected) = expected else {
            return Ok(());
        };
        let name = self.start(node)?.name();
        let prefix = prefix(name.as_ref().as_bytes(), expected.prefix);
        self.validate_qname(
            node,
            Query {
                prefix,
                expected: expected.uri,
                kind: Kind::Element,
            },
        )
    }

    pub fn validate_attribute_namespace(
        &self,
        node: usize,
        name: &[u8],
        expected: Option<&str>,
    ) -> Result<(), DecodeError> {
        let Some(expected) = expected else {
            return Ok(());
        };
        self.validate_qname(
            node,
            Query {
                prefix: prefix(name, None),
                expected,
                kind: Kind::Attribute,
            },
        )
    }

    fn validate_qname(&self, node: usize, query: Query<'_>) -> Result<(), DecodeError> {
        if query.prefix == b"xml" {
            return query.check("http://www.w3.org/XML/1998/namespace");
        }
        let mut ancestor = Some(node);
        while let Some(index) = ancestor {
            if self.namespace_at(index, &query)? {
                return Ok(());
            }
            ancestor = self.parents.get(index).copied().flatten();
        }
        if query.prefix.is_empty()
            && (matches!(query.kind, Kind::Element) || query.expected.is_empty())
        {
            return Ok(());
        }
        Err(DecodeError::Invalid("unbound XML namespace"))
    }

    fn namespace_at(&self, node: usize, query: &Query<'_>) -> Result<bool, DecodeError> {
        for attribute in self.start(node)?.attributes().with_checks(true) {
            let attribute =
                attribute.map_err(|_| DecodeError::Invalid("XML namespace attribute"))?;
            if query.matches(attribute.key.as_ref().as_bytes()) {
                let uri = attribute
                    .normalized_value(XmlVersion::Implicit1_0)
                    .map_err(|_| DecodeError::Invalid("XML namespace value"))?;
                query.check(&uri)?;
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn start(&self, node: usize) -> Result<&BytesStart<'_>, DecodeError> {
        match self.tokens.get(node) {
            Some(Token::Start(start) | Token::Empty(start)) => Ok(start),
            _ => Err(DecodeError::Invalid("XML namespace element")),
        }
    }
}

impl Query<'_> {
    fn matches(&self, key: &[u8]) -> bool {
        if self.prefix.is_empty() && matches!(self.kind, Kind::Element) {
            key == b"xmlns"
        } else {
            key.strip_prefix(b"xmlns:")
                .is_some_and(|found| found == self.prefix)
        }
    }

    fn check(&self, uri: &str) -> Result<(), DecodeError> {
        if uri == self.expected {
            Ok(())
        } else {
            Err(DecodeError::Invalid("XML namespace"))
        }
    }
}

fn prefix<'a>(name: &'a [u8], model: Option<&'a str>) -> &'a [u8] {
    match model {
        Some(prefix) => prefix.as_bytes(),
        None => name
            .iter()
            .position(|byte| *byte == b':')
            .and_then(|colon| name.get(..colon))
            .map_or(&[], |prefix| prefix),
    }
}
