use std::{
    fs::File,
    io::{stdin, Read},
    path::PathBuf,
};

use clap::{Parser, Subcommand};
use mp4_atom::ReadFrom;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(short, long, help = "Path to the input file; default is stdin")]
    input: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Info {
        #[arg(short, long)]
        verbose: bool,
    },
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let args = Args::parse();

    match args.input {
        Some(path) => {
            let mut file = File::open(path)?;
            info(&mut file)
        }
        None => info(&mut stdin()),
    }
}

fn info<R: Read>(input: &mut R) -> anyhow::Result<()> {
    while let Some(atom) = Option::<mp4_atom::Any>::read_from(input)? {
        match atom {
            mp4_atom::Any::Mdat(mdat) => {
                println!("Mdat {{ size: {:?} }}", mdat.data.len());
            }
            mp4_atom::Any::Unknown(kind, data) => {
                println!("Unknown {{ kind: {:?}, size: {:?} }}", kind, data.len());
            }
            _ => {
                println!("{:#?}", atom);
            }
        }
    }

    Ok(())
}
