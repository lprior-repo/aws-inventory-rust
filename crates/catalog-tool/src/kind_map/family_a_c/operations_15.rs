use crate::kind_map::OperationMapping;

pub(super) const ENTRIES: &[OperationMapping<'static>] = &[
    OperationMapping { service: "codebuild", operation: "ListProjects", kind: "AWS::CodeBuild::Project" },
    OperationMapping { service: "codebuild", operation: "ListReportGroups", kind: "AWS::CodeBuild::ReportGroup" },
    OperationMapping { service: "codebuild", operation: "ListReports", kind: "AWS::ApiOnly::CodeBuildReport" },
    OperationMapping { service: "codebuild", operation: "ListSandboxes", kind: "AWS::ApiOnly::CodeBuildSandbox" },
    OperationMapping { service: "codecommit", operation: "GetRepository", kind: "AWS::CodeCommit::Repository" },
    OperationMapping { service: "codecommit", operation: "ListApprovalRuleTemplates", kind: "AWS::ApiOnly::CodeCommitApprovalRuleTemplate" },
    OperationMapping { service: "codecommit", operation: "ListRepositories", kind: "AWS::CodeCommit::Repository" },
    OperationMapping { service: "codeconnections", operation: "GetConnection", kind: "AWS::CodeStarConnections::Connection" },
    OperationMapping { service: "codeconnections", operation: "GetRepositoryLink", kind: "AWS::CodeStarConnections::RepositoryLink" },
    OperationMapping { service: "codeconnections", operation: "ListConnections", kind: "AWS::CodeStarConnections::Connection" },
    OperationMapping { service: "codeconnections", operation: "ListHosts", kind: "AWS::ApiOnly::CodeStarConnectionsHost" },
    OperationMapping { service: "codeconnections", operation: "ListRepositoryLinks", kind: "AWS::CodeStarConnections::RepositoryLink" },
    OperationMapping { service: "codedeploy", operation: "GetApplication", kind: "AWS::CodeDeploy::Application" },
    OperationMapping { service: "codedeploy", operation: "GetDeploymentConfig", kind: "AWS::CodeDeploy::DeploymentConfig" },
    OperationMapping { service: "codedeploy", operation: "ListApplications", kind: "AWS::CodeDeploy::Application" },
    OperationMapping { service: "codedeploy", operation: "ListDeploymentConfigs", kind: "AWS::CodeDeploy::DeploymentConfig" },
    OperationMapping { service: "codedeploy", operation: "ListDeployments", kind: "AWS::ApiOnly::CodeDeployDeployment" },
    OperationMapping { service: "codedeploy", operation: "ListOnPremisesInstances", kind: "AWS::ApiOnly::CodeDeployOnPremisesInstance" },
    OperationMapping { service: "codeguru-reviewer", operation: "DescribeRepositoryAssociation", kind: "AWS::CodeGuruReviewer::RepositoryAssociation" },
    OperationMapping { service: "codeguru-reviewer", operation: "ListRepositoryAssociations", kind: "AWS::CodeGuruReviewer::RepositoryAssociation" },
];
