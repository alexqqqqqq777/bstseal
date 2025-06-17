use clap::Parser;
use std::fs::{self, File};
use std::io::{self, Read, Write, BufReader, BufWriter};
use std::path::PathBuf;
use std::time::Instant;
use walkdir::WalkDir;
use bstseal_core::encode::{decode_parallel, encode_parallel};

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
    /// Verifies integrity footer of a bstseal file
    Fsck {
        /// File to check
        #[clap(value_parser)]
        input: PathBuf,
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
    /// Packs multiple files into an archive
    Pack {
        /// Output archive file
        #[clap(short, long)]
        output: PathBuf,
        /// Input files/dirs to include
        #[clap(required = true)]
        inputs: Vec<PathBuf>,
    },
    /// Unpacks archive to directory
    Unpack {
        /// Archive to unpack
        archive: PathBuf,
        /// Output directory (default '.')
        #[clap(short, long, default_value = ".")]
        out_dir: PathBuf,
    },
    /// Lists archive contents
    List {
        archive: PathBuf,
    },
    /// Outputs single file to stdout
    Cat {
        archive: PathBuf,
        /// Path inside archive
        file: String,
    },
    /// Runs micro-benchmark
    Bench {
        /// Optional sample file
        #[clap(short, long)]
        file: Option<PathBuf>,
    },
}

fn main() -> Result<(), anyhow::Error> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Pack { output, inputs } => pack_archive(output, inputs)?,
        Commands::Unpack { archive, out_dir } => unpack_archive(archive, out_dir)?,
        Commands::List { archive } => list_archive(archive)?,
        Commands::Cat { archive, file } => cat_file(archive, file)?,
        Commands::Bench { file } => run_bench(file)?,
        Commands::Encode { input, output } => {
            println!("Encoding file: {:?} to {:?}", input, output);

            let mut input_file = BufReader::new(File::open(&input)?);
            let mut input_data = Vec::new();
            input_file.read_to_end(&mut input_data)?;

            let start_time = Instant::now();
            let compressed = encode_parallel(&input_data)?;
            let encoded_data = bstseal_core::integrity::add_footer(&compressed);
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
            let payload = match bstseal_core::integrity::verify_footer(&input_data) {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("Integrity check failed: {e}");
                    std::process::exit(1);
                }
            };
            let decoded_data_result = decode_parallel(payload);
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
                    return Err(anyhow::anyhow!("Decoding failed: {}", e));
                }
            }
        }
        Commands::Fsck { input } => {
            let mut file = BufReader::new(File::open(&input)?);
            let mut data = Vec::new();
            file.read_to_end(&mut data)?;
            match bstseal_core::integrity::verify_footer(&data) {
                Ok(_) => {
                    println!("{}: OK", input.display());
                }
                Err(e) => {
                    eprintln!("{}: FAILED – {e}", input.display());
                    std::process::exit(1);
                }
            }
        }
    }

    Ok(())
}

// ---------------- archive helpers ----------------
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

const MAGIC: &[u8; 8] = b"BSTSEAL\0";

struct IndexEntry {
    path: String,
    offset: u64,
    size:   u64,
}

fn pack_archive(output: PathBuf, inputs: Vec<PathBuf>) -> anyhow::Result<()> {
    let mut files = Vec::new();
    for input in inputs {
        if input.is_dir() {
            for entry in WalkDir::new(&input).into_iter().filter_map(Result::ok).filter(|e| e.path().is_file()) {
                files.push(entry.path().to_path_buf());
            }
        } else {
            files.push(input);
        }
    }
    if files.is_empty() {
        anyhow::bail!("no input files");
    }

    // Compress all files first to know sizes
    let mut payloads = Vec::new(); // (path, data)
    for path in &files {
        let mut buf = Vec::new();
        BufReader::new(File::open(path)?).read_to_end(&mut buf)?;
        let compressed = encode_parallel(&buf)?;
        let with_footer = bstseal_core::integrity::add_footer(&compressed);
        payloads.push((path.strip_prefix(&std::env::current_dir()?)?.to_string_lossy().to_string(), with_footer));
    }

    // Compute header length
    let entry_count = payloads.len() as u32;
    let mut header_len: usize = 8 + 4; // MAGIC + count
    for (path_str, data) in &payloads {
        header_len += 2 + path_str.len() + 8 + 8;
    }

    // Prepare header in memory
    let mut header = Vec::with_capacity(header_len);
    header.extend_from_slice(MAGIC);
    header.write_u32::<LittleEndian>(entry_count)?;
    let mut offset_acc = header_len as u64;
    for (path_str, data) in &payloads {
        let path_bytes = path_str.as_bytes();
        header.write_u16::<LittleEndian>(path_bytes.len() as u16)?;
        header.extend_from_slice(path_bytes);
        header.write_u64::<LittleEndian>(offset_acc)?;
        header.write_u64::<LittleEndian>(data.len() as u64)?;
        offset_acc += data.len() as u64;
    }

    // Write archive file
    let mut out = BufWriter::new(File::create(output)?);
    out.write_all(&header)?;
    for (_, data) in payloads {
        out.write_all(&data)?;
    }
    out.flush()?;
    Ok(())
}

fn read_index(mut reader: &mut (impl Read + Seek)) -> anyhow::Result<Vec<IndexEntry>> {
    let mut magic = [0u8; 8];
    reader.read_exact(&mut magic)?;
    if &magic != MAGIC {
        anyhow::bail!("invalid archive magic");
    }
    let count = reader.read_u32::<LittleEndian>()?;
    let mut entries = Vec::with_capacity(count as usize);
    for _ in 0..count {
        let path_len = reader.read_u16::<LittleEndian>()? as usize;
        let mut path_buf = vec![0u8; path_len];
        reader.read_exact(&mut path_buf)?;
        let offset = reader.read_u64::<LittleEndian>()?;
        let size = reader.read_u64::<LittleEndian>()?;
        let path = String::from_utf8(path_buf)?;
        entries.push(IndexEntry { path, offset, size });
    }
    Ok(entries)
}

fn list_archive(archive: PathBuf) -> anyhow::Result<()> {
    let mut file = BufReader::new(File::open(archive)?);
    let entries = read_index(&mut file)?;
    println!("{:<8} {:<12} {}", "Offset", "Size", "Path");
    for e in entries {
        println!("{:<8} {:<12} {}", e.offset, e.size, e.path);
    }
    Ok(())
}

use std::io::Seek;
use std::io::SeekFrom;

fn unpack_archive(archive: PathBuf, out_dir: PathBuf) -> anyhow::Result<()> {
    fs::create_dir_all(&out_dir)?;
    let mut file = File::open(&archive)?;
    let mut reader = BufReader::new(&file);
    let entries = read_index(&mut reader)?;
    for e in entries {
        let out_path = out_dir.join(&e.path);
        if let Some(p) = out_path.parent() { fs::create_dir_all(p)?; }
        file.seek(SeekFrom::Start(e.offset))?;
        let mut compressed = vec![0u8; e.size as usize];
        file.read_exact(&mut compressed)?;
        let payload = bstseal_core::integrity::verify_footer(&compressed)?;
        let data = decode_parallel(payload)?;
        BufWriter::new(File::create(out_path)?).write_all(&data)?;
    }
    Ok(())
}

fn cat_file(archive: PathBuf, file_path: String) -> anyhow::Result<()> {
    let mut file = File::open(&archive)?;
    let mut reader = BufReader::new(&file);
    let entries = read_index(&mut reader)?;
    let target = entries.into_iter().find(|e| e.path == file_path)
        .ok_or_else(|| anyhow::anyhow!("path not found in archive"))?;
    file.seek(SeekFrom::Start(target.offset))?;
    let mut compressed = vec![0u8; target.size as usize];
    file.read_exact(&mut compressed)?;
    let payload = bstseal_core::integrity::verify_footer(&compressed)?;
    let data = decode_parallel(payload)?;
    io::stdout().write_all(&data)?;
    Ok(())
}

fn run_bench(sample: Option<PathBuf>) -> anyhow::Result<()> {
    use std::time::Instant;
    let data = if let Some(file) = sample {
        let mut v = Vec::new();
        BufReader::new(File::open(file)?).read_to_end(&mut v)?;
        v
    } else {
        vec![0u8; 32 * 1024] // 32KB zeros as sample
    };
    let t0 = Instant::now();
    let compressed = encode_parallel(&data)?;
    let encode_ms = t0.elapsed().as_secs_f64() * 1000.0;
    let t1 = Instant::now();
    let _ = decode_parallel(&compressed)?;
    let decode_ms = t1.elapsed().as_secs_f64() * 1000.0;
    println!("encode: {:.2} ms ({:.1} MB/s)", encode_ms, data.len() as f64 / 1e6 / (encode_ms/1000.0));
    println!("decode: {:.2} ms ({:.1} MB/s)", decode_ms, data.len() as f64 / 1e6 / (decode_ms/1000.0));
    Ok(())
}
