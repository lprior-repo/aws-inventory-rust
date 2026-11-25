use super::MemberPath;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MapKeyExclusion {
    pub path: MemberPath,
    pub key: String,
}
