use clap::Parser;

type MyResult<T> = Result<T, Box<dyn std::error::Error>>;

#[derive(Parser, Debug)]
#[command(
    name = "lsr",
    version = "0.1.0",
    author = "Radish-Miyazaki <y.hidaka.kobe@gmail.com>",
    about = "Rust ls"
)]
pub struct Args {
    #[arg(help = "Files and/or directories", default_value = ".")]
    paths: Vec<String>,
    #[arg(help = "Long listing", short, long)]
    long: bool,
    #[arg(help = "Show all files", short = 'a', long = "all")]
    show_hidden: bool,
}

pub fn run() -> MyResult<()> {
    let args = Args::parse();
    println!("{:#?}", args);

    Ok(())
}
