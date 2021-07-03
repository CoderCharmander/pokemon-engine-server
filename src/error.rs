use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct StringError(pub String);
impl std::error::Error for StringError {}
impl Display for StringError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
