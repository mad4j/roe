use clap::ValueEnum;

/// Output format for command results
#[derive(ValueEnum, Clone, Debug, Default)]
pub enum OutputFormat {
    Json,
    #[default]
    Table,
}
