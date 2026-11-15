// Explicit dispositions from .local/aws-models/models/databrew/service/2017-07-25/databrew-2017-07-25.json; never a runtime read-name approval rule.
pub(super) const ENTRIES: &[(&str, &str)] = &[
    ("UpdateRecipe", "Requires Name; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Modifies the definition of the LATEST_WORKING version of a DataBrew recipe."),
    ("UpdateRecipeJob", "Requires Name, RoleArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Modifies the definition of an existing DataBrew recipe job."),
    ("UpdateRuleset", "Requires Name, Rules; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates specified ruleset."),
    ("UpdateSchedule", "Requires CronExpression, Name; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Modifies the definition of an existing DataBrew schedule."),
];
