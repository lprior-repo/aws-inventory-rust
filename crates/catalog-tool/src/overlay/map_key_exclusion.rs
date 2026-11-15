use super::MemberPath;

/// One literal named key inside a modeled map; other entries remain hash inputs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MapKeyExclusion {
    pub path: MemberPath,
    pub key: String,
}
