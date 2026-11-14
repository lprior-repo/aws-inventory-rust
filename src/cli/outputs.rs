use super::*;

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum ExportFormat {
    Json,
    #[default]
    Jsonl,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum ExportProvenance {
    #[default]
    Normalized,
    AllObservations,
}

#[derive(Args, Debug)]
pub struct ExportArgs {
    #[arg(long, default_value = "inventory.sqlite")]
    pub db: PathBuf,
    #[arg(long)]
    pub scan_id: String,
    #[arg(long)]
    pub output: PathBuf,
    #[arg(long, value_enum, default_value_t = ExportFormat::Jsonl)]
    pub format: ExportFormat,
    #[arg(long, value_enum, default_value_t = ExportProvenance::Normalized)]
    pub provenance: ExportProvenance,
    #[arg(long)]
    pub allow_partial: bool,

    #[arg(long = "universal-references")]
    pub derive_references: bool,
}

impl ExportArgs {
    pub fn reference_mode(&self) -> inventory_engine::export::ReferenceMode {
        inventory_engine::export::ReferenceMode::from_flag(self.derive_references)
    }
}

#[derive(Args, Debug)]
pub struct DeliverArgs {
    #[arg(long, default_value = "inventory.sqlite")]
    pub db: PathBuf,
    #[arg(long)]
    pub scan_id: String,
    #[command(flatten)]
    pub delivery: DeliveryArgs,
}

#[derive(Args, Clone, Debug, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct DeliveryArgs {
    #[arg(long)]
    pub output_dir: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = ExportFormat::Jsonl)]
    pub export_format: ExportFormat,
    #[arg(long, value_enum, default_value_t = ExportProvenance::Normalized)]
    pub provenance: ExportProvenance,

    #[arg(long, requires = "output_dir")]
    pub falkor: bool,
    #[arg(long, default_value = "FALKORDB_URL")]
    pub falkor_url_env: String,
    #[arg(long, default_value = "aws_inventory")]
    pub graph: String,
    #[arg(long, default_value_t = 5)]
    pub delivery_attempts: u32,

    #[arg(long, default_value_t = 900)]
    pub delivery_timeout_seconds: u64,

    #[arg(long)]
    pub allow_partial: bool,
}

impl Default for DeliveryArgs {
    fn default() -> Self {
        Self {
            output_dir: None,
            export_format: ExportFormat::Jsonl,
            provenance: ExportProvenance::Normalized,
            falkor: false,
            falkor_url_env: "FALKORDB_URL".into(),
            graph: "aws_inventory".into(),
            delivery_attempts: 5,
            delivery_timeout_seconds: 900,
            allow_partial: false,
        }
    }
}
