#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shape {
    Page,
    Items,
    Item,
    State,
    Groups,
    Group,
    Tags,
    Text,
    Int,
    Float,
    Bool,
    Null,
    Time,
    Blob,
    Document,
    Skip,
}
#[derive(Clone, Copy)]
pub struct Member {
    pub json: &'static str,
    pub xml: &'static str,
    pub id: u32,
    pub shape: Shape,
}
const fn m(json: &'static str, xml: &'static str, id: u32, shape: Shape) -> Member {
    Member {
        json,
        xml,
        id,
        shape,
    }
}
pub const ITEM: &[Member] = &[
    m("InstanceId", "instanceId", 1, Shape::Text),
    m("State", "instanceState", 2, Shape::State),
    m("SecurityGroups", "groupSet", 3, Shape::Groups),
    m("Tags", "tagSet", 4, Shape::Tags),
    m("Description", "description", 5, Shape::Text),
    m("CpuCount", "cpuCount", 6, Shape::Int),
    m("Ratio", "ratio", 7, Shape::Float),
    m("LargeLong", "largeLong", 8, Shape::Int),
    m("Enabled", "enabled", 9, Shape::Bool),
    m("Optional", "optional", 10, Shape::Null),
    m("LaunchTime", "launchTime", 11, Shape::Time),
    m("UserData", "userData", 12, Shape::Blob),
    m("Document", "document", 13, Shape::Document),
];
const STATE: &[Member] = &[
    m("Code", "code", 1, Shape::Int),
    m("Name", "name", 2, Shape::Text),
];
const GROUP: &[Member] = &[
    m("GroupId", "groupId", 1, Shape::Text),
    m("GroupName", "groupName", 2, Shape::Text),
];
const PAGE: &[Member] = &[m("Items", "instancesSet", 1, Shape::Items)];
pub fn members(shape: Shape) -> &'static [Member] {
    match shape {
        Shape::Item => ITEM,
        Shape::State => STATE,
        Shape::Group => GROUP,
        Shape::Page => PAGE,
        _ => &[],
    }
}
pub fn member(shape: Shape, name: &str) -> Option<Member> {
    members(shape)
        .iter()
        .find(|m| m.json == name || m.xml == name)
        .copied()
}
pub fn child(shape: Shape, name: &str) -> Shape {
    match shape {
        Shape::Items => Shape::Item,
        Shape::Groups => Shape::Group,
        Shape::Tags => Shape::Text,
        Shape::Document => Shape::Document,
        _ => member(shape, name).map_or(Shape::Skip, |m| m.shape),
    }
}
impl Shape {
    pub fn object(self) -> bool {
        matches!(
            self,
            Self::Page | Self::Item | Self::State | Self::Group | Self::Tags
        )
    }
    pub fn list(self) -> bool {
        matches!(self, Self::Items | Self::Groups)
    }
}
