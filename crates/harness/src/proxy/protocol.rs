#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum BodyPolicy {
    Head,
    Normal,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum Connection {
    KeepAlive,
    Close,
}
impl Connection {
    pub(crate) fn combine(self, other: Self) -> Self {
        match (self, other) {
            (Self::KeepAlive, Self::KeepAlive) => Self::KeepAlive,
            _ => Self::Close,
        }
    }
}

pub(crate) enum TransportDisposition {
    Forward,
    Handled,
}
