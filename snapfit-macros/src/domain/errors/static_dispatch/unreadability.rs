//! Error: why a dispatch attribute cannot be read. Plays the role of its parent.

use derive_more::Display;

#[derive(Debug, Clone, PartialEq, Eq, Display)]
pub(crate) enum Unreadability {
    /// The words of syn, which could not parse the tokens.
    #[display("{_0}")]
    Syn(String),
    #[display("it needs its settings in parentheses")]
    WithoutParentheses,
    #[display("it takes its settings in parentheses, not after '='")]
    AfterEquals,
    /// What follows the name, as written.
    #[display("it found {_0} after the name, where nothing may follow")]
    AfterName(String),
    /// The first token stream after the list, as written.
    #[display("it takes one list after the name, and found {_0} after it")]
    SecondList(String),
}
