use super::*;

fn indices() -> [Parent; 2] {
    ["11111111-1111-1111-1111-111111111111", "22222222-2222-2222-2222-222222222222"].map(|id|
        Parent { binding: id.to_owned(), id: id.to_owned(), kind: "AWS::Kendra::Index",
            row: json!({"Id":id,"Name":"index metadata","Status":"ACTIVE"}) })
}

fn kendra(operation: &'static str, items: &'static str, kind: &'static str) -> Case {
    Case { service: "kendra", date: "2019-02-03", root: "ListIndices", root_items: "IndexConfigurationSummaryItems",
        operation, items, input: "IndexId", token: "NextToken", kind, relation: "IN_INDEX",
        forbidden: &["DescribeDataSource", "DescribeFaq", "DescribeExperience", "DescribeQuerySuggestionsBlockList",
            "DescribeThesaurus", "DescribeFeaturedResultsSet", "DescribeAccessControlConfiguration", "Query", "Retrieve", "GetQuerySuggestions"] }
}

#[tokio::test]
async fn given_two_indexes_when_data_source_pages_resume_then_duplicate_ids_are_index_qualified_without_connector_detail() -> TestResult {
    exercise(kendra("ListDataSources", "SummaryItems", "AWS::Kendra::DataSource"), indices()).await
}

#[tokio::test]
async fn given_two_indexes_when_faq_pages_resume_then_duplicate_ids_are_index_qualified_without_faq_content() -> TestResult {
    exercise(kendra("ListFaqs", "FaqSummaryItems", "AWS::Kendra::Faq"), indices()).await
}

#[tokio::test]
async fn given_two_indexes_when_experience_pages_resume_then_home_endpoint_metadata_and_index_edges_survive_without_search() -> TestResult {
    exercise(kendra("ListExperiences", "SummaryItems", "AWS::ApiOnly::KendraExperience"), indices()).await
}

#[tokio::test]
async fn given_two_indexes_when_blocklist_pages_resume_then_duplicate_ids_are_index_qualified_without_blocked_phrases() -> TestResult {
    exercise(kendra("ListQuerySuggestionsBlockLists", "BlockListSummaryItems", "AWS::Kendra::QuerySuggestionsBlockList"), indices()).await
}

#[tokio::test]
async fn given_two_indexes_when_thesaurus_pages_resume_then_duplicate_ids_are_index_qualified_without_thesaurus_content() -> TestResult {
    exercise(kendra("ListThesauri", "ThesaurusSummaryItems", "AWS::Kendra::Thesaurus"), indices()).await
}

#[tokio::test]
async fn given_two_indexes_when_featured_result_explicit_tokens_resume_then_parent_qualified_ids_survive_without_query_documents() -> TestResult {
    exercise(kendra("ListFeaturedResultsSets", "FeaturedResultsSetSummaryItems", "AWS::Kendra::FeaturedResultsSet"), indices()).await
}

#[tokio::test]
async fn given_two_indexes_when_acl_identifier_pages_resume_then_configurations_remain_index_qualified_without_document_acls() -> TestResult {
    exercise(kendra("ListAccessControlConfigurations", "AccessControlConfigurations", "AWS::ApiOnly::KendraAccessControlConfiguration"), indices()).await
}

#[tokio::test]
async fn given_two_keyspaces_when_scalar_type_pages_resume_then_duplicate_names_remain_qualified_without_type_or_table_reads() -> TestResult {
    let parents = ["keyspace_a", "keyspace_b"].map(|name| {
        let arn = format!("arn:aws:cassandra:us-east-1:123456789012:/keyspace/{name}/");
        Parent { binding: name.to_owned(), id: arn.clone(), kind: "AWS::Cassandra::Keyspace",
            row: json!({"keyspaceName":name,"resourceArn":arn}) }
    });
    exercise(Case { service: "keyspaces", date: "2022-02-10", root: "ListKeyspaces", root_items: "keyspaces",
        operation: "ListTypes", items: "types", input: "keyspaceName", token: "nextToken",
        kind: "AWS::Cassandra::Type", relation: "IN_KEYSPACE", forbidden: &["GetType"] }, parents).await
}
