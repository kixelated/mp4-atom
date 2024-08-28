use std::{
    fs::File,
    io::{stdin, BufReader, Read, Seek},
    path::PathBuf,
};

use clap::{Parser, Subcommand};

use mp4box::*;

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
    let args = Args::parse();

    match args.input {
        Some(path) => {
            let file = File::open(path)?;
            info(file)
        }
        None => info(stdin()),
    }
}

fn info<R: Read>(input: R) -> anyhow::Result<()> {
    let mut reader = BufReader::new(input);
    let mut reader = Reader::new(&mut reader);

    while let Some(header) = reader.header()? {
        reader = match header.kind() {
            Mdat::KIND => {
                println!("Mdat {{ size: {:?} }}", header.size());
                header.skip()?
            }
            _ => {
                let (atom, reader) = header.atom()?;
                println!("{:?}", atom);
                reader
            }
        }
    }

    Ok(())
}
