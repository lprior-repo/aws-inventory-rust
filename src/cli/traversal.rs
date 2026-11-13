use super::*;

#[derive(Args, Debug)]
pub struct TraversalArgs {
    #[arg(long, value_enum, default_value_t = crate::analysis::Direction::Both)]
    pub direction: crate::analysis::Direction,
    #[arg(long, default_value_t = 2)]
    pub max_depth: usize,
    #[arg(long, default_value_t = 1000)]
    pub max_nodes: usize,
    #[arg(long, default_value_t = 4000)]
    pub max_edges: usize,
}
