//! A simple command-line MP4 parser.
//!
//! This example reads an MP4 file from stdin or a file and prints the atoms it finds.
//!
//! cargo run --example info  -- <input_file>
use std::{
    fs::File,
    io::{stdin, Read},
};

use mp4_atom::ReadFrom;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    match std::env::args().nth(1) {
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
