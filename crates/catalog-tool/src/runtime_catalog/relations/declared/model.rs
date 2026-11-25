use super::*;
use serde::Deserialize;

const VISIT_LIMIT: usize = 65_536;
const DECLARATION_LIMIT: usize = 65_536;

#[derive(Clone, Copy)]
pub(super) enum Origin {
    Arn,
    Smithy,
}

pub(super) struct Declaration {
    pub resource: String,
    pub path: String,
    pub members: Option<BTreeMap<String, String>>,
    pub invalid: Option<&'static str>,
    pub origin: Origin,
}

impl Declaration {
    fn at(&self, prefix: &str) -> Self {
        Self {
            resource: self.resource.clone(),
            path: if self.path == "$" {
                root(prefix)
            } else {
                join(prefix, &self.path)
            },
            members: self.members.as_ref().map(|members| {
                members
                    .iter()
                    .map(|(id, path)| (id.clone(), join(prefix, path)))
                    .collect()
            }),
            invalid: self.invalid,
            origin: self.origin,
        }
    }
}

/// Bounds scheduled shape visits without charging their exit markers.
#[derive(Default)]
struct WalkBudget {
    visits: usize,
}

impl WalkBudget {
    fn visit(&mut self) -> Result<(), RuntimeCatalogCompileError> {
        self.visits = self
            .visits
            .checked_add(1)
            .filter(|visits| *visits <= VISIT_LIMIT)
            .ok_or(RuntimeCatalogCompileError::ReferenceWalkLimit { limit: VISIT_LIMIT })?;
        Ok(())
    }
}

#[derive(Deserialize)]
struct Reference {
    resource: String,
    #[serde(default)]
    ids: BTreeMap<String, String>,
}

pub(super) fn declarations(
    service: Service<'_>,
    item: Target,
    references: &reachability::Reachability,
) -> Result<Vec<Declaration>, RuntimeCatalogCompileError> {
    let mut budget = WalkBudget::default();
    budget.visit()?;
    let mut nodes = vec![Node::Enter(item, String::new(), false, 0)];
    let mut active = BTreeSet::new();
    let mut declarations = Vec::new();
    while let Some(node) = nodes.pop() {
        visit(
            service,
            node,
            references,
            &mut active,
            &mut nodes,
            &mut declarations,
            &mut budget,
        )?;
    }
    Ok(declarations)
}

enum Node {
    Enter(Target, String, bool, usize),
    Exit(usize),
}

fn visit(
    service: Service<'_>,
    node: Node,
    references: &reachability::Reachability,
    active: &mut BTreeSet<usize>,
    next: &mut Vec<Node>,
    declarations: &mut Vec<Declaration>,
    budget: &mut WalkBudget,
) -> Result<(), RuntimeCatalogCompileError> {
    let (target, prefix, container, depth) = match node {
        Node::Exit(index) => {
            active.remove(&index);
            return Ok(());
        }
        Node::Enter(target, prefix, container, depth) => (target, prefix, container, depth),
    };
    let Target::Shape(index) = target else {
        return Ok(());
    };
    if !references.contains(target) {
        return Ok(());
    }
    let cut = if active.contains(&index) {
        Some("cyclic reference path")
    } else if depth > 8 {
        Some("reference path exceeds eight members")
    } else {
        None
    };
    append(
        references.direct(target),
        &prefix,
        container,
        cut,
        declarations,
    )?;
    if let Some(reason) = cut {
        return pend_descendants(references, target, &prefix, reason, declarations);
    }
    active.insert(index);
    let shape = service
        .shape(index)?
        .ok_or(RuntimeCatalogCompileError::Capability)?;
    next.push(Node::Exit(index));
    schedule(shape, references, budget, next, (&prefix, container, depth))
}

fn schedule(
    shape: Shape<'_>,
    references: &reachability::Reachability,
    budget: &mut WalkBudget,
    next: &mut Vec<Node>,
    (prefix, container, depth): (&str, bool, usize),
) -> Result<(), RuntimeCatalogCompileError> {
    if let Some(element) = shape
        .element()?
        .or(shape.map_value()?)
        .filter(|target| references.contains(*target))
    {
        budget.visit()?;
        next.push(Node::Enter(element, prefix.to_owned(), true, depth));
    }
    shape.members()?.try_for_each(|member| {
        let member = member?;
        if references.contains(member.target) {
            budget.visit()?;
            next.push(Node::Enter(
                member.target,
                join(prefix, member.name()?),
                container,
                depth.saturating_add(1),
            ));
        }
        Ok(())
    })
}

fn append(
    templates: &[Declaration],
    prefix: &str,
    container: bool,
    cut: Option<&'static str>,
    declarations: &mut Vec<Declaration>,
) -> Result<(), RuntimeCatalogCompileError> {
    for template in templates {
        let mut declaration = template.at(prefix);
        declaration.invalid =
            cut.or(declaration.invalid).or(container
                .then_some("reference path traverses a container (requires E2.2 extraction)"));
        push(declarations, declaration)?;
    }
    Ok(())
}

fn pend_descendants(
    references: &reachability::Reachability,
    target: Target,
    prefix: &str,
    reason: &'static str,
    declarations: &mut Vec<Declaration>,
) -> Result<(), RuntimeCatalogCompileError> {
    for resource in references.resources(target) {
        if !references
            .direct(target)
            .iter()
            .any(|declaration| declaration.resource == resource)
        {
            push(
                declarations,
                Declaration {
                    resource: resource.to_owned(),
                    path: root(prefix),
                    members: None,
                    invalid: Some(reason),
                    origin: Origin::Smithy,
                },
            )?;
        }
    }
    Ok(())
}

fn push(
    declarations: &mut Vec<Declaration>,
    declaration: Declaration,
) -> Result<(), RuntimeCatalogCompileError> {
    if declarations.len() >= DECLARATION_LIMIT {
        return Err(RuntimeCatalogCompileError::ReferenceDeclarationsLimit {
            limit: DECLARATION_LIMIT,
        });
    }
    declarations.push(declaration);
    Ok(())
}

pub(super) fn direct(
    shape: Shape<'_>,
    resources: &resources::Resources<'_, '_>,
) -> Result<Vec<Declaration>, RuntimeCatalogCompileError> {
    let mut declarations = Vec::new();
    collect(shape, resources, &mut declarations)?;
    for member in shape.members()? {
        let member = member?;
        arn(member.arn_reference()?, member.name()?, &mut declarations)?;
    }
    Ok(declarations)
}

fn collect(
    shape: Shape<'_>,
    resources: &resources::Resources<'_, '_>,
    declarations: &mut Vec<Declaration>,
) -> Result<(), RuntimeCatalogCompileError> {
    arn(shape.arn_reference()?, "", declarations)?;
    let Some(payload) = shape.references()? else {
        return Ok(());
    };
    let references: Vec<Reference> =
        serde_json::from_str(payload).map_err(|_| RuntimeCatalogCompileError::Capability)?;
    let structure = shape.kind()? == "structure";
    let names = shape
        .members()?
        .map(|member| member.and_then(|member| member.name()))
        .collect::<Result<BTreeSet<_>, _>>()?;
    references.into_iter().try_for_each(|reference| {
        if !structure && !reference.ids.is_empty() {
            return Err(RuntimeCatalogCompileError::ReferenceShape {
                shape: shape.name()?.to_owned(),
                reason: "ids require a structure",
            });
        }
        let members = if reference.ids.is_empty() && structure {
            resources
                .get(&reference.resource)
                .into_iter()
                .flat_map(|resource| &resource.identifiers)
                .map(|name| ((*name).to_owned(), (*name).to_owned()))
                .collect()
        } else {
            reference.ids
        };
        let invalid = (structure
            && members
                .values()
                .any(|member| !names.contains(member.as_str())))
        .then_some("reference identifier member is absent");
        let path = members
            .values()
            .next()
            .map_or_else(|| root(""), Clone::clone);
        let members = structure.then_some(members);
        push(
            declarations,
            Declaration {
                resource: reference.resource,
                path,
                members,
                invalid,
                origin: Origin::Smithy,
            },
        )?;
        Ok(())
    })
}

fn arn(
    payload: Option<&str>,
    path: &str,
    declarations: &mut Vec<Declaration>,
) -> Result<(), RuntimeCatalogCompileError> {
    let Some(payload) = payload else {
        return Ok(());
    };
    let payload: serde_json::Value =
        serde_json::from_str(payload).map_err(|_| RuntimeCatalogCompileError::Capability)?;
    if let Some(resource) = payload.get("resource").and_then(serde_json::Value::as_str) {
        push(
            declarations,
            Declaration {
                resource: resource.to_owned(),
                path: root(path),
                members: None,
                invalid: None,
                origin: Origin::Arn,
            },
        )?;
    }
    Ok(())
}

fn join(prefix: &str, member: &str) -> String {
    if prefix.is_empty() {
        member.to_owned()
    } else {
        format!("{prefix}.{member}")
    }
}

fn root(path: &str) -> String {
    if path.is_empty() {
        "$".to_owned()
    } else {
        path.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_a_walk_budget_when_exactly_65536_shapes_are_visited_then_only_the_next_visit_is_refused()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut budget = WalkBudget::default();
        for _ in 0..65_536 {
            budget.visit()?;
        }
        assert_eq!(budget.visits, 65_536);
        let error = budget
            .visit()
            .err()
            .ok_or("expected reference walk exhaustion")?;
        assert_eq!(
            error.to_string(),
            "model reference walk exceeds 65536 shape visits"
        );
        assert_eq!(budget.visits, 65_536);
        Ok(())
    }
}
