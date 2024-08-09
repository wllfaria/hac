pub mod collection;
pub mod command;
pub mod fs;
pub mod net;
pub mod syntax;
pub mod text_object;

#[derive(Debug, Copy, Clone)]
pub enum AuthKind {
    Bearer,
    None,
}

#[derive(Default)]
pub struct AuthKindIter {
    inner: u8,
}

impl std::fmt::Display for AuthKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthKind::None => write!(f, "None"),
            AuthKind::Bearer => write!(f, "Bearer"),
        }
    }
}

impl AuthKind {
    pub fn iter() -> AuthKindIter {
        AuthKindIter::default()
    }
}

impl Iterator for AuthKindIter {
    type Item = AuthKind;

    fn next(&mut self) -> Option<Self::Item> {
        let variant = match self.inner {
            0 => Some(AuthKind::Bearer),
            1 => Some(AuthKind::None),
            _ => None,
        };
        self.inner += 1;
        variant
    }
}
