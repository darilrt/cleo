#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub enum LexerError {
    #[default]
    Default,
    InvalidCharacter {
        character: char,
        position: usize,
    },
}
