use super::state_tests::{graph, row};
use super::{
    apply_filter_enqueue_result, apply_graph_data, apply_selection_enqueue_result,
    apply_snapshot_data,
};
use crate::data::{Counts, Data, Filter, Graph};

#[test]
fn given_accepted_page_work_when_candidate_filter_is_rejected_then_accepted_filter_and_work_survive()
 {
    // Given an accepted filter, selected resource, and its live graph/affiliations.
    let mut data = Data::default();
    let accepted = Filter {
        query: "accepted".to_owned(),
        ..Filter::default()
    };
    data.set_filter(accepted.clone());
    let current = row("accepted-current");
    data.replace_page(vec![current.clone()], 1);
    let current_graph = graph(&current);
    data.set_graph(current_graph.clone(), None);
    let candidate = Filter {
        query: "rejected".to_owned(),
        tag: Some(cmdb_view::TagFilter {
            key: "Environment".to_owned(),
            value: Some("rejected".to_owned()),
        }),
        ..Filter::default()
    };

    // When the real consumer handles an enqueue rejection, the candidate never becomes active.
    assert!(!apply_filter_enqueue_result(
        &mut data,
        candidate,
        Err("inventory worker queue is full".to_owned())
    ));
    assert_eq!(data.filter(), &accepted);
    assert_eq!(data.selected_row(), Some(&current));
    assert_eq!(data.visual_graph(), current_graph);
    assert!(data.status_line().contains("queue is full"));

    // Then an already accepted page remains deliverable under its correct filter.
    let completed = row("accepted-completed");
    apply_snapshot_data(
        &mut data,
        Counts {
            resources: 1,
            ..Counts::default()
        },
        vec![completed.clone()],
        1,
        None,
    );
    assert_eq!(data.filter(), &accepted);
    assert_eq!(data.selected_row(), Some(&completed));
    assert!(!data.status_line().contains("queue is full"));
}

#[test]
fn given_a_candidate_filter_when_enqueue_succeeds_then_all_filter_fields_commit_together() {
    // Given a candidate covering every supported filter, including an exact empty tag value.
    let mut data = Data::default();
    let candidate = Filter {
        account: Some("111111111111".to_owned()),
        region: Some("us-east-1".to_owned()),
        kind: Some("AWS::EC2::VPC".to_owned()),
        query: "new".to_owned(),
        tag: Some(cmdb_view::TagFilter {
            key: " owner ".to_owned(),
            value: Some(String::new()),
        }),
    };
    // When the request is accepted, its owned candidate becomes the active filter.
    assert!(apply_filter_enqueue_result(
        &mut data,
        candidate.clone(),
        Ok(())
    ));
    assert_eq!(data.filter(), &candidate);
    // Then its completed page is displayed with the same committed filter.
    let fresh = row("new");
    apply_snapshot_data(&mut data, Counts::default(), vec![fresh.clone()], 1, None);
    assert_eq!(data.filter(), &candidate);
    assert_eq!(data.selected_row(), Some(&fresh));
}

#[test]
fn given_accepted_graph_work_when_new_selection_is_rejected_then_original_selection_and_work_survive()
 {
    // Given a selected resource with live affiliations and another selectable resource.
    let mut data = Data::default();
    let current = row("current");
    let candidate = row("candidate");
    data.replace_page(vec![current.clone(), candidate.clone()], 2);
    let current_graph = graph(&current);
    data.set_graph(current_graph.clone(), None);
    // When expansion enqueue fails, selecting the candidate is not committed.
    assert!(!apply_selection_enqueue_result(
        &mut data,
        candidate,
        Err("inventory worker queue is full".to_owned())
    ));
    assert_eq!(data.selected_row(), Some(&current));
    assert_eq!(data.visual_graph(), current_graph);
    assert!(data.graph_status().contains("queue is full"));
    // Then the accepted graph completion still matches the retained selection and applies.
    let completed = Graph {
        truncated: false,
        tags_truncated: false,
        ..current_graph
    };
    assert!(apply_graph_data(&mut data, completed.clone(), None));
    assert_eq!(data.visual_graph(), completed);
    assert!(!data.graph_status().contains("queue is full"));
}

#[test]
fn given_an_accepted_expansion_when_selection_commits_then_old_details_and_tag_affiliations_are_cleared()
 {
    // Given details and graph data belonging to the old selected resource.
    let mut data = Data::default();
    let old = row("old");
    let candidate = row("new");
    data.replace_page(vec![old.clone(), candidate.clone()], 2);
    data.set_graph(graph(&old), Some("old graph failure".to_owned()));
    // When expansion is accepted, commit the candidate through the production consumer.
    assert!(apply_selection_enqueue_result(
        &mut data,
        candidate.clone(),
        Ok(())
    ));
    // Then metadata belongs to the new resource, without old graph warning/affiliations.
    assert_eq!(data.selected_row(), Some(&candidate));
    assert!(data.visual_graph().tag_affiliations.is_empty());
    assert!(!data.visual_graph().tags_truncated);
    assert!(!data.graph_status().contains("old graph failure"));
}
