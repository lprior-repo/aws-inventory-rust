use super::*;
pub(in crate::response::json) fn decode_payload(
    context: ResponseContext<'_>,
    member: Member<'_>,
    builder: &mut RecordBuilder,
) -> Result<(), DecodeError> {
    if context.limits.excluded_members.contains(member.id) {
        return Ok(());
    }
    if context.body.is_empty() {
        return crate::response::defaults::apply_member(
            builder,
            member,
            context.service,
            context.limits,
        );
    }
    let target = member.target;
    let key = Key::Field(member.id, member.wire()?.timestamp_format()?, false);
    if scalar::write_raw_payload(context, target, key.clone(), builder)? {
        return Ok(());
    }
    Engine::new(context, builder, 0).run(Task::Value(Value {
        target: Some(target),
        key,
        depth: 0,
    }))
}
pub(in crate::response::json) fn payload_member(
    shape: Shape<'_>,
) -> Result<Option<Member<'_>>, DecodeError> {
    for item in shape.members()? {
        let member = item?;
        if matches!(member.wire()?.binding()?, HttpBinding::Payload) {
            return Ok(Some(member));
        }
    }
    Ok(None)
}
