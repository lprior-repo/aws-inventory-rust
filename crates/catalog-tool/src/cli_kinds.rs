pub(super) fn kinds(
    arguments: &mut std::slice::Iter<'_, String>,
) -> Result<(), Box<dyn std::error::Error>> {
    if arguments.next().map(String::as_str) != Some("generate") {
        return Err("kinds requires the generate subcommand".into());
    }
    let inputs = super::report_inputs(arguments)?;
    catalog_tool::kind_map::generate::run(&inputs.root, &inputs.source, inputs.mode)
}
