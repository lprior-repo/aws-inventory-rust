use super::*;

#[derive(Subcommand, Debug)]
pub enum TagMappingCommand {
    /// Replaces the owner/team/application tag-key mapping in a database. Reads only the
    /// local mapping file and the database; makes no AWS call.
    Apply {
        #[arg(long, default_value = "inventory.sqlite")]
        db: PathBuf,
        #[arg(long)]
        file: PathBuf,
    },
}
