use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[clap(index = 1)]
    pub query: String,

    #[arg(short, long, default_value_t = 0)]
    pub sel: usize,

    #[arg(long, default_value_t = 20)]
    pub retries: usize,
}
