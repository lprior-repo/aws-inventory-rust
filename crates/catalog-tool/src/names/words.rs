use super::{NameError, output::Output};

pub(super) fn append_plain(part: &str, output: &mut Output<'_>) -> Result<(), NameError> {
    if part.is_empty() {
        return Ok(());
    }
    output.separator()?;
    plain_chars(part).try_for_each(|(ch, boundary)| {
        if boundary {
            output.separator()?;
        }
        output.push(ch.to_ascii_lowercase())
    })
}

fn plain_chars(part: &str) -> impl Iterator<Item = (char, bool)> + '_ {
    let version = part
        .rsplit_once('V')
        .filter(|(_, digits)| !digits.is_empty() && digits.chars().all(|ch| ch.is_ascii_digit()))
        .map(|(prefix, _)| prefix.len());
    let previous = std::iter::once(None).chain(part.chars().map(Some));
    let next = part.chars().skip(1).map(Some).chain(std::iter::once(None));
    part.char_indices()
        .zip(previous)
        .zip(next)
        .map(move |(((position, ch), previous), next)| {
            let boundary = previous.is_some_and(|prev| word_boundary(prev, ch, next))
                || (position != 0 && version == Some(position));
            (ch, boundary)
        })
}

fn word_boundary(previous: char, current: char, next: Option<char>) -> bool {
    current.is_ascii_uppercase()
        && (previous.is_ascii_lowercase()
            || previous.is_ascii_digit()
            || (previous.is_ascii_uppercase() && next.is_some_and(|ch| ch.is_ascii_lowercase())))
}
