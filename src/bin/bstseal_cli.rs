use clap::Parser;
use std::fs::File;
use std::io::{Read, Write, BufReader, BufWriter};
use std::path::PathBuf;
use std::time::Instant;

// Импортируем функции из нашей библиотеки bstseal
// Убедитесь, что ваш крейт `bstseal` доступен как библиотека.
// Если `main.rs` или `lib.rs` в корне `bstseal_rust_project/src` определяет его как `bstseal`,
// то это должно работать. Возможно, потребуется `use bstseal;` или `use bstseal_lib;`
// в зависимости от того, как назван ваш библиотечный крейт в Cargo.toml.
// Для простоты предположим, что функции доступны через `bstseal::encode::*`
// Если ваш библиотечный крейт называется иначе, нужно будет это исправить.
// extern crate bstseal; // Может понадобиться, если структура проекта сложная

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Parser, Debug)]
enum Commands {
    /// Encodes a file using bstseal compression
    Encode {
        /// Input file to encode
        #[clap(short, long, value_parser)]
        input: PathBuf,

        /// Output file to write encoded data
        #[clap(short, long, value_parser)]
        output: PathBuf,
    },
    /// Decodes a file previously encoded with bstseal
    Decode {
        /// Input file to decode
        #[clap(short, long, value_parser)]
        input: PathBuf,

        /// Output file to write decoded data
        #[clap(short, long, value_parser)]
        output: PathBuf,
    },
}

fn main() -> Result<(), anyhow::Error> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Encode { input, output } => {
            println!("Encoding file: {:?} to {:?}", input, output);

            let mut input_file = BufReader::new(File::open(&input)?);
            let mut input_data = Vec::new();
            input_file.read_to_end(&mut input_data)?;

            let start_time = Instant::now();
            // Предполагаем, что функции из вашего крейта доступны так:
            let encoded_data = bstseal::encode::encode_parallel(&input_data);
            let duration = start_time.elapsed();

            let mut output_file = BufWriter::new(File::create(&output)?);
            output_file.write_all(&encoded_data)?;

            println!("Operation: encode");
            println!("Input file: {:?}", input);
            println!("Output file: {:?}", output);
            println!("Original size: {} bytes", input_data.len());
            println!("Compressed size: {} bytes", encoded_data.len());
            println!("Time taken: {:.2?}", duration);
        }
        Commands::Decode { input, output } => {
            println!("Decoding file: {:?} to {:?}", input, output);

            let mut input_file = BufReader::new(File::open(&input)?);
            let mut input_data = Vec::new();
            input_file.read_to_end(&mut input_data)?;

            let start_time = Instant::now();
            // Предполагаем, что функции из вашего крейта доступны так:
            let decoded_data_result = bstseal::encode::decode_parallel(&input_data);
            let duration = start_time.elapsed();

            match decoded_data_result {
                Ok(decoded_data) => {
                    let mut output_file = BufWriter::new(File::create(&output)?);
                    output_file.write_all(&decoded_data)?;

                    println!("Operation: decode");
                    println!("Input file: {:?}", input);
                    println!("Output file: {:?}", output);
                    println!("Compressed size: {} bytes", input_data.len());
                    println!("Original size: {} bytes", decoded_data.len());
                    println!("Time taken: {:.2?}", duration);
                }
                Err(e) => {
                    eprintln!("Decoding error: {}", e);
                    // Возвращаем ошибку, чтобы программа завершилась с ненулевым кодом
                    return Err(anyhow::anyhow!("Decoding failed: {}", e)); 
                }
            }
        }
    }
    Ok(())
}
