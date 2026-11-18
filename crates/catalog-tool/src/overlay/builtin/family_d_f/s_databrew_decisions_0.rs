// Explicit dispositions from .local/aws-models/models/databrew/service/2017-07-25/databrew-2017-07-25.json; never a runtime read-name approval rule.
pub(super) const ENTRIES: &[(&str, &str)] = &[
    (
        "BatchDeleteRecipeVersion",
        "Requires Name, RecipeVersions; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes one or more versions of a recipe at a time.",
    ),
    (
        "CreateDataset",
        "Requires Name, Input; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates a new DataBrew dataset.",
    ),
    (
        "CreateProfileJob",
        "Requires DatasetName, Name, OutputLocation, RoleArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates a new job to analyze a dataset and create its data profile.",
    ),
    (
        "CreateProject",
        "Requires DatasetName, Name, RecipeName, RoleArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates a new DataBrew project.",
    ),
    (
        "CreateRecipe",
        "Requires Name, Steps; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates a new DataBrew recipe.",
    ),
    (
        "CreateRecipeJob",
        "Requires Name, RoleArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates a new job to transform input data, using steps defined in an existing Glue DataBrew recipe",
    ),
    (
        "CreateRuleset",
        "Requires Name, TargetArn, Rules; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates a new ruleset that can be used in a profile job to validate the data quality of a dataset.",
    ),
    (
        "CreateSchedule",
        "Requires CronExpression, Name; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates a new schedule for one or more DataBrew jobs.",
    ),
    (
        "DeleteDataset",
        "Requires Name; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes a dataset from DataBrew.",
    ),
    (
        "DeleteJob",
        "Requires Name; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes the specified DataBrew job.",
    ),
    (
        "DeleteProject",
        "Requires Name; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes an existing DataBrew project.",
    ),
    (
        "DeleteRecipeVersion",
        "Requires Name, RecipeVersion; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes a single version of a DataBrew recipe.",
    ),
    (
        "DeleteRuleset",
        "Requires Name; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes a ruleset.",
    ),
    (
        "DeleteSchedule",
        "Requires Name; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes the specified DataBrew schedule.",
    ),
    (
        "DescribeDataset",
        "Requires Name; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Returns the definition of a specific DataBrew dataset.",
    ),
    (
        "DescribeJob",
        "Requires Name; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Returns the definition of a specific DataBrew job.",
    ),
    (
        "DescribeJobRun",
        "Requires Name, RunId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Represents one run of a DataBrew job.",
    ),
    (
        "DescribeProject",
        "Requires Name; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Returns the definition of a specific DataBrew project.",
    ),
    (
        "DescribeRecipe",
        "Requires Name; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Returns the definition of a specific DataBrew recipe corresponding to a particular version.",
    ),
    (
        "DescribeRuleset",
        "Requires Name; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Retrieves detailed information about the ruleset.",
    ),
    (
        "DescribeSchedule",
        "Requires Name; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Returns the definition of a specific DataBrew schedule.",
    ),
    (
        "ListJobRuns",
        "Requires Name; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Lists all of the previous runs of a particular DataBrew job.",
    ),
    (
        "ListRecipeVersions",
        "Requires Name; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Lists the versions of a particular DataBrew recipe, except for LATEST_WORKING .",
    ),
    (
        "ListRecipes",
        "Recipe includes Steps[].Action.Parameters and ConditionExpressions; recipe transformation bodies are customer content, not a metadata-only summary. Pinned operation: Lists all of the DataBrew recipes that are defined.",
    ),
    (
        "ListTagsForResource",
        "Requires ResourceArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Lists all the tags for a DataBrew resource.",
    ),
    (
        "PublishRecipe",
        "Requires Name; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Publishes a new version of a DataBrew recipe.",
    ),
    (
        "SendProjectSessionAction",
        "Requires Name; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Performs a recipe step within an interactive DataBrew session that's currently open.",
    ),
    (
        "StartJobRun",
        "Requires Name; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Runs a DataBrew job.",
    ),
    (
        "StartProjectSession",
        "Requires Name; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates an interactive session, enabling you to manipulate data in a DataBrew project.",
    ),
    (
        "StopJobRun",
        "Requires Name, RunId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Stops a particular run of a job.",
    ),
    (
        "TagResource",
        "Requires ResourceArn, Tags; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Adds metadata tags to a DataBrew resource, such as a dataset, project, recipe, job, or schedule.",
    ),
    (
        "UntagResource",
        "Requires ResourceArn, TagKeys; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Removes metadata tags from a DataBrew resource.",
    ),
    (
        "UpdateDataset",
        "Requires Name, Input; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Modifies the definition of an existing DataBrew dataset.",
    ),
    (
        "UpdateProfileJob",
        "Requires Name, OutputLocation, RoleArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Modifies the definition of an existing profile job.",
    ),
    (
        "UpdateProject",
        "Requires RoleArn, Name; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Modifies the definition of an existing DataBrew project.",
    ),
];
