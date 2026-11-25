//! Entity: where a `self::` falls in a type as written, so that a name can
//! drop each one that starts a path.

/// Where a `self::` of a type as written falls.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Place {
    /// The first piece, which no `self::` precedes.
    TextStart,
    /// Where it starts a path, so that it is dropped.
    PathStart,
    /// After an identifier or a `::`, where it is kept as written, so that
    /// rustc still reports a path such as `super::self::X`.
    InsidePath,
}

/// The text so far with `piece` added after the `self::` at `place`, and
/// where the `self::` after `piece` falls.
pub(super) fn rejoin((mut text, place): (String, Place), piece: &str) -> (String, Place) {
    if place == Place::InsidePath {
        text.push_str("self::");
    }
    text.push_str(piece);
    let next = match piece.chars().last() {
        None if place == Place::TextStart => Place::PathStart,
        None => Place::InsidePath,
        Some(c) if c.is_alphanumeric() || c == '_' || c == ':' => Place::InsidePath,
        Some(_) => Place::PathStart,
    };
    (text, next)
}
