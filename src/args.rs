use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Supply custom Java path
    #[arg(long, short)]
    pub java_path: Option<String>,
}
