// Reduced pinned ECR shapes, preserving actual operation/member names and pagination traits.
const ECR_MODEL: &str = r#"{"smithy":"2.0","shapes":{
"ecr#Service":{"type":"service","operations":[{"target":"ecr#DescribeRepositories"},{"target":"ecr#DescribeImages"}]},
"ecr#DescribeRepositories":{"type":"operation","input":{"target":"ecr#RepositoriesInput"},"output":{"target":"ecr#RepositoriesOutput"},"traits":{"smithy.api#paginated":{"inputToken":"nextToken","outputToken":"nextToken","items":"repositories"}}},
"ecr#DescribeImages":{"type":"operation","input":{"target":"ecr#ImagesInput"},"output":{"target":"ecr#ImagesOutput"},"traits":{"smithy.api#paginated":{"inputToken":"nextToken","outputToken":"nextToken","items":"imageDetails"}}},
"ecr#RepositoriesInput":{"type":"structure","members":{"nextToken":{"target":"smithy.api#String"}}},
"ecr#ImagesInput":{"type":"structure","members":{"repositoryName":{"target":"smithy.api#String","traits":{"smithy.api#required":{}}},"nextToken":{"target":"smithy.api#String"},"filter":{"target":"ecr#DescribeImagesFilter"}}},
"ecr#DescribeImagesFilter":{"type":"structure","members":{"tagStatus":{"target":"ecr#TagStatus"},"imageStatus":{"target":"ecr#ImageStatusFilter"}}},
"ecr#TagStatus":{"type":"enum","members":{"TAGGED":{"target":"smithy.api#Unit","traits":{"smithy.api#enumValue":"TAGGED"}},"UNTAGGED":{"target":"smithy.api#Unit","traits":{"smithy.api#enumValue":"UNTAGGED"}},"ANY":{"target":"smithy.api#Unit","traits":{"smithy.api#enumValue":"ANY"}}}},
"ecr#ImageStatusFilter":{"type":"enum","members":{"ACTIVE":{"target":"smithy.api#Unit","traits":{"smithy.api#enumValue":"ACTIVE"}},"ARCHIVED":{"target":"smithy.api#Unit","traits":{"smithy.api#enumValue":"ARCHIVED"}},"ACTIVATING":{"target":"smithy.api#Unit","traits":{"smithy.api#enumValue":"ACTIVATING"}},"ANY":{"target":"smithy.api#Unit","traits":{"smithy.api#enumValue":"ANY"}}}},
"ecr#RepositoriesOutput":{"type":"structure","members":{"repositories":{"target":"ecr#Repositories"},"nextToken":{"target":"smithy.api#String"}}},
"ecr#ImagesOutput":{"type":"structure","members":{"imageDetails":{"target":"ecr#Images"},"nextToken":{"target":"smithy.api#String"}}},
"ecr#Repositories":{"type":"list","member":{"target":"ecr#Repository"}},
"ecr#Images":{"type":"list","member":{"target":"ecr#Image"}},
"ecr#Repository":{"type":"structure","members":{"repositoryArn":{"target":"smithy.api#String"},"registryId":{"target":"smithy.api#String"},"repositoryName":{"target":"smithy.api#String"}}},
"ecr#Image":{"type":"structure","members":{"registryId":{"target":"smithy.api#String"},"repositoryName":{"target":"smithy.api#String"},"imageDigest":{"target":"smithy.api#String"},"imageStatus":{"target":"smithy.api#String"}}}
}}"#;
