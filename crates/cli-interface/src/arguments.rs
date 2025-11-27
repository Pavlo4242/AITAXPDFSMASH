use clap::{Args, Parser, Subcommand};

#[derive(Args, Debug, Clone)]
pub struct DictionaryArgs {
    #[clap(required = true)]
    pub wordlist: String,
}

#[derive(Args, Debug, Clone)]
pub struct RangeArgs {
    #[clap(short, long)]
    pub add_preceding_zeros: bool,
    pub lower_bound: usize,
    pub upper_bound: usize,
}

#[derive(Args, Debug, Clone)]
pub struct CustomQueryArgs {
    pub custom_query: String,
    #[clap(short, long)]
    pub add_preceding_zeros: bool,
}

#[derive(Args, Debug, Clone)]
pub struct DefaultQueryArgs {
    #[clap(long, default_value_t = 4)]
    pub min_length: u32,
    #[clap(long)]
    pub max_length: u32,
}

#[derive(Args, Debug, Clone)]
pub struct DateArgs {
    pub start: usize,
    pub end: usize,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Method {
    Wordlist(DictionaryArgs),
    Range(RangeArgs),
    CustomQuery(CustomQueryArgs),
    Date(DateArgs),
    DefaultQuery(DefaultQueryArgs),
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Arguments {
    #[clap(short, long, default_value_t = 4)]
    pub number_of_threads: usize,

    #[clap(short, long)]
    pub filename: String,

    #[command(subcommand)]
    pub subcommand: Method,
}

pub fn args() -> Arguments {
    Arguments::parse()
}
