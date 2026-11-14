use crate::{
    error::{Error, Result, syntax},
    sink::Sink,
    xml_bridge::Bridge,
};
use aws_smithy_xml::decode::{Document, try_data};
// XmlToken's inner token is private. The public depth-first start-element API
// can drive a shape, but hides end-element/error events consumed by next_start_element.
// Closing at the next depth is the only public non-recursive traversal here;
// adversarial tests deliberately expose its resulting truncation acceptance.
pub fn parse(input: &[u8], sink: &mut Sink) -> Result<()> {
    let mut doc = Document::try_from(input).map_err(syntax)?;
    let mut bridge = Bridge::new(sink);
    std::iter::from_fn(|| {
        doc.next_start_element().map(|el| {
            let depth = el.depth();
            close_to(&mut bridge, depth)?;
            bridge.start(el.local())?;
            if bridge.scalar() {
                bridge.text(&try_data(&mut doc.scoped_to(el)).map_err(syntax)?)?;
                bridge.close()?;
            }
            Ok::<_, Error>(())
        })
    })
    .take(input.len().saturating_add(1))
    .try_for_each(|r| r)?;
    close_to(&mut bridge, 0)?;
    bridge.finish()
}
fn close_to(bridge: &mut Bridge<'_>, depth: usize) -> Result<()> {
    let count = bridge
        .depth()
        .checked_sub(depth)
        .ok_or(Error::Shape("Smithy XML depth"))?;
    (0..count).try_for_each(|_| bridge.close())
}
