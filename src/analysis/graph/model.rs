use super::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(super) struct Node {
    pub(super) id: String,
    pub(super) kind: Option<String>,
    pub(super) name: Option<String>,
    pub(super) observed: bool,
    pub(super) distance: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(super) struct Relationship {
    pub(super) from_id: String,
    pub(super) to_id: String,
    pub(super) relation: String,
    pub(super) source: Source,
    pub(super) observed_at: i64,
    pub(super) from_kind: Option<String>,
    pub(super) from_name: Option<String>,
    pub(super) to_kind: Option<String>,
    pub(super) to_name: Option<String>,
    pub(super) source_observed: u8,
    pub(super) target_observed: u8,
    pub(super) owner_resolution: Option<String>,
}

impl Relationship {
    pub(super) fn validate(&self) -> Result<()> {
        validate_identifier(&self.from_id, "edge source")?;
        validate_identifier(&self.to_id, "edge target")?;
        validate_identifier(&self.relation, "edge relation")?;
        anyhow::ensure!(
            self.source_observed <= 1 && self.target_observed <= 1,
            "invalid observed-endpoint flag"
        );
        anyhow::ensure!(self.observed_at >= 0, "negative edge observation time");
        Ok(())
    }

    pub(super) fn key(&self) -> EdgeKey {
        (
            self.from_id.clone(),
            self.to_id.clone(),
            self.relation.clone(),
        )
    }

    pub(super) fn neighbor<'a>(&'a self, current: &str, direction: Direction) -> Result<&'a str> {
        match direction {
            Direction::Outgoing if self.from_id == current => Ok(&self.to_id),
            Direction::Incoming if self.to_id == current => Ok(&self.from_id),
            Direction::Both if self.from_id == current => Ok(&self.to_id),
            Direction::Both if self.to_id == current => Ok(&self.from_id),
            _ => anyhow::bail!("adjacency row does not match traversal direction"),
        }
    }

    pub(super) fn endpoint(&self, id: &str, distance: usize) -> Result<Node> {
        if self.from_id == id {
            return Ok(Node {
                id: id.into(),
                kind: self.from_kind.clone(),
                name: self.from_name.clone(),
                observed: self.source_observed == 1,
                distance,
            });
        }
        anyhow::ensure!(self.to_id == id, "endpoint does not belong to edge");
        Ok(Node {
            id: id.into(),
            kind: self.to_kind.clone(),
            name: self.to_name.clone(),
            observed: self.target_observed == 1,
            distance,
        })
    }
}

#[derive(Default, Serialize)]
pub(super) struct Truncation {
    pub(super) node_limit: bool,
    pub(super) edge_limit: bool,
    pub(super) detail_byte_limit: bool,
    pub(super) inspected_edge_limit: bool,
}

impl Truncation {
    pub(super) fn any(&self) -> bool {
        self.node_limit || self.edge_limit || self.detail_byte_limit || self.inspected_edge_limit
    }
}

pub(super) struct Predecessor {
    pub(super) from: String,
    pub(super) edge: EdgeKey,
}

pub(super) struct Walk {
    pub(super) nodes: BTreeMap<String, Node>,
    pub(super) edges: BTreeMap<EdgeKey, Relationship>,
    pub(super) parents: BTreeMap<String, Predecessor>,
    pub(super) queue: VecDeque<String>,
    pub(super) options: TraversalOptions,
    pub(super) target: Option<String>,
    pub(super) found: bool,
    pub(super) expanded_nodes: usize,
    pub(super) depth_boundary_nodes: usize,
    pub(super) inspected_edges: usize,
    pub(super) inspection_limit: usize,
    pub(super) detail_bytes: usize,
    pub(super) truncation: Truncation,
}

impl Walk {
    pub(super) fn new(seed: Node, options: TraversalOptions, target: Option<&str>) -> Result<Self> {
        options.validate()?;
        let mut queue = VecDeque::new();
        queue
            .try_reserve(options.max_nodes)
            .context("reserve traversal queue")?;
        queue.push_back(seed.id.clone());
        let found = target.is_some_and(|target| target == seed.id);
        let detail_bytes = encoded_len(&seed)?;
        let nodes = BTreeMap::from([(seed.id.clone(), seed)]);
        let inspection_limit = options
            .max_edges
            .checked_mul(2)
            .and_then(|value| value.checked_add(1))
            .context("inspection bound overflow")?;
        Ok(Self {
            nodes,
            edges: BTreeMap::new(),
            parents: BTreeMap::new(),
            queue,
            options,
            target: target.map(str::to_owned),
            found,
            expanded_nodes: 0,
            depth_boundary_nodes: 0,
            inspected_edges: 0,
            inspection_limit,
            detail_bytes,
            truncation: Truncation::default(),
        })
    }

    pub(super) fn stopped(&self) -> bool {
        self.found || self.truncation.any()
    }

    pub(super) fn accept(
        &mut self,
        current: &str,
        distance: usize,
        edge: Relationship,
    ) -> Result<()> {
        edge.validate()?;
        self.inspected_edges = self
            .inspected_edges
            .checked_add(1)
            .context("edge count overflow")?;
        if self.inspected_edges > self.inspection_limit {
            self.truncation.inspected_edge_limit = true;
            return Ok(());
        }
        let neighbor = edge.neighbor(current, self.options.direction)?;
        let key = edge.key();
        if self.edges.contains_key(&key) {
            return Ok(());
        }
        if self.edges.len() == self.options.max_edges {
            self.truncation.edge_limit = true;
            return Ok(());
        }
        let candidate = if self.nodes.contains_key(neighbor) {
            None
        } else {
            Some(edge.endpoint(neighbor, distance)?)
        };
        if candidate.is_some() && self.nodes.len() == self.options.max_nodes {
            self.truncation.node_limit = true;
            return Ok(());
        }
        self.insert(current, key, candidate, edge)
    }

    pub(super) fn insert(
        &mut self,
        current: &str,
        key: EdgeKey,
        candidate: Option<Node>,
        edge: Relationship,
    ) -> Result<()> {
        let node_bytes = candidate
            .as_ref()
            .map(encoded_len)
            .transpose()?
            .unwrap_or_default();
        let edge_bytes = encoded_len(&edge)?;
        let next = self
            .detail_bytes
            .checked_add(node_bytes)
            .and_then(|value| value.checked_add(edge_bytes))
            .context("traversal detail byte count overflow")?;
        anyhow::ensure!(
            edge_bytes <= GRAPH_DETAIL_BYTES,
            "edge metadata exceeds graph bound"
        );
        if next > GRAPH_DETAIL_BYTES {
            self.truncation.detail_byte_limit = true;
            return Ok(());
        }
        if let Some(node) = candidate {
            self.add_node(current, &key, node)?;
        }
        self.detail_bytes = next;
        anyhow::ensure!(
            self.edges.insert(key, edge).is_none(),
            "duplicate traversal edge insertion"
        );
        Ok(())
    }

    pub(super) fn add_node(&mut self, current: &str, key: &EdgeKey, node: Node) -> Result<()> {
        self.found = self
            .target
            .as_deref()
            .is_some_and(|target| target == node.id);
        let predecessor = Predecessor {
            from: current.into(),
            edge: key.clone(),
        };
        anyhow::ensure!(
            self.parents.insert(node.id.clone(), predecessor).is_none(),
            "duplicate predecessor insertion"
        );
        self.queue.push_back(node.id.clone());
        anyhow::ensure!(
            self.nodes.insert(node.id.clone(), node).is_none(),
            "duplicate node insertion"
        );
        Ok(())
    }

    pub(super) fn path(&self, start: &str, target: &str) -> Result<Value> {
        if !self.found {
            return Ok(json!({"found":false,"result":"not_found_within_limits",
                "nodes":[],"steps":[],"effective_reachability_inferred":false}));
        }
        let mut cursor = target.to_owned();
        let mut nodes = vec![cursor.clone()];
        let mut steps = Vec::new();
        steps
            .try_reserve(self.options.max_depth)
            .context("reserve path steps")?;
        for _ in 0..=super::super::MAX_TRAVERSAL_DEPTH {
            if cursor == start {
                break;
            }
            let parent = self
                .parents
                .get(&cursor)
                .context("path predecessor missing")?;
            let edge = self.edges.get(&parent.edge).context("path edge missing")?;
            steps.push(json!({"traversed_from":parent.from,"traversed_to":cursor,
                "reversed":edge.from_id != parent.from,"relationship":edge}));
            cursor.clone_from(&parent.from);
            nodes.push(cursor.clone());
        }
        anyhow::ensure!(cursor == start, "path reconstruction exceeded depth bound");
        nodes.reverse();
        steps.reverse();
        Ok(
            json!({"found":true,"result":"observed_relationship_path","hops":steps.len(),
            "nodes":nodes,"steps":steps,"shortest_for_selected_direction":!self.truncation.any(),
            "effective_reachability_inferred":false}),
        )
    }
}
