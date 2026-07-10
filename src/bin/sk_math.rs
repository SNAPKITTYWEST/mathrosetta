use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;
use std::fs;

#[derive(Parser)]
#[command(name = "sk-math")]
#[command(about = "SnapKitty Rosetta Math Engine")]
#[command(version = "0.2.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, default_value = "json")]
    format: OutputFormat,

    #[arg(short, long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    Translate {
        input: String,
        #[arg(short, long)]
        from: InputFormat,
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    Solve {
        input: String,
        #[arg(short, long, default_value = "auto")]
        from: InputFormat,
        #[arg(short, long, default_value = "witness")]
        proof: ProofLevel,
        #[arg(short, long)]
        backend: Option<String>,
    },
    Normalize {
        input: String,
    },
    Dispatch {
        input: String,
    },
    Emit {
        input: String,
        #[arg(short, long, default_value = "theorem")]
        theory: String,
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    Bundle {
        input: String,
        #[arg(short, long, default_value = "theorem")]
        theory: String,
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    Worm {
        input: String,
        #[arg(short, long, default_value = "theorem")]
        theory: String,
        #[arg(short, long)]
        chain_file: Option<PathBuf>,
    },
    ParseNatural {
        input: String,
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    Schema {
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

#[derive(ValueEnum, Clone)]
enum InputFormat {
    Auto,
    Latex,
    Python,
    Sympy,
    Lean,
    Mathir,
    Natural,
}

#[derive(ValueEnum, Clone)]
enum OutputFormat {
    Json,
    Latex,
    Pretty,
}

#[derive(ValueEnum, Clone)]
enum ProofLevel {
    None,
    Witness,
    Full,
    Lean,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Translate { input, from, output } => {
            let expr = parse_input(&input, &from)?;
            let json = serde_json::to_string_pretty(&expr)?;
            match output {
                Some(path) => fs::write(path, &json)?,
                None => println!("{}", json),
            }
        }
        Commands::Solve { input, from, proof: _, backend: _ } => {
            let expr = parse_input(&input, &from)?;
            let normalizer = mathir::Normalizer::new();
            let normalized = normalizer.normalize(&expr);

            let dispatcher = mathir::Dispatcher::new();
            let dispatch = dispatcher.dispatch(&normalized);

            let result = serde_json::json!({
                "mathir_original": expr,
                "mathir_normalized": normalized,
                "equation_class": dispatch.equation_class,
                "solver": dispatch.solver,
                "proof_requirement": dispatch.proof,
                "confidence": dispatch.confidence,
            });

            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        Commands::Normalize { input } => {
            let expr: mathir::MathIR = serde_json::from_str(&input)?;
            let normalizer = mathir::Normalizer::new();
            let normalized = normalizer.normalize(&expr);
            println!("{}", serde_json::to_string_pretty(&normalized)?);
        }
        Commands::Dispatch { input } => {
            let expr: mathir::MathIR = serde_json::from_str(&input)?;
            let dispatcher = mathir::Dispatcher::new();
            let dispatch = dispatcher.dispatch(&expr);
            println!("{}", serde_json::to_string_pretty(&dispatch)?);
        }
        Commands::Emit { input, theory, output } => {
            let expr = parse_input(&input, &InputFormat::Auto)?;
            let normalizer = mathir::Normalizer::new();
            let normalized = normalizer.normalize(&expr);

            let targets = mathir::emit_all(&normalized, &theory);

            let result = serde_json::json!({
                "theory": theory,
                "targets": targets.iter().map(|t| serde_json::json!({
                    "backend": format!("{:?}", t.backend),
                    "status": "emitted_pending_proof",
                    "source": t.source,
                })).collect::<Vec<_>>(),
            });

            let json = serde_json::to_string_pretty(&result)?;
            match output {
                Some(path) => fs::write(path, &json)?,
                None => println!("{}", json),
            }
        }
        Commands::Bundle { input, theory, output } => {
            let expr = parse_input(&input, &InputFormat::Auto)?;
            let normalizer = mathir::Normalizer::new();
            let normalized = normalizer.normalize(&expr);

            let bundle = mathir::ProofBundle::new(&input, expr, normalized, &theory);
            let export = bundle.to_export();

            let json = serde_json::to_string_pretty(&export)?;
            match output {
                Some(path) => fs::write(path, &json)?,
                None => println!("{}", json),
            }
        }
        Commands::Worm { input, theory, chain_file } => {
            let expr = parse_input(&input, &InputFormat::Auto)?;
            let normalizer = mathir::Normalizer::new();
            let normalized = normalizer.normalize(&expr);

            let bundle = mathir::ProofBundle::new(&input, expr, normalized, &theory);

            let mut chain = match &chain_file {
                Some(path) => {
                    let data = fs::read_to_string(path).unwrap_or_default();
                    serde_json::from_str(&data).unwrap_or_else(|_| mathir::WormChain::genesis())
                }
                None => mathir::WormChain::genesis(),
            };

            let receipt = chain.append(&bundle);

            let output_data = serde_json::json!({
                "receipt": receipt,
                "chain_length": chain.len(),
                "chain_valid": chain.verify_chain(),
            });

            match chain_file {
                Some(path) => {
                    fs::write(&path, serde_json::to_string_pretty(&chain)?)?;
                    println!("Chain written to {}", path.display());
                    println!("{}", serde_json::to_string_pretty(&output_data)?);
                }
                None => {
                    println!("{}", serde_json::to_string_pretty(&output_data)?);
                }
            }
        }
        Commands::ParseNatural { input, output } => {
            let expr = mathir::NaturalParser::parse(&input)?;
            let json = serde_json::to_string_pretty(&expr)?;
            match output {
                Some(path) => fs::write(path, &json)?,
                None => println!("{}", json),
            }
        }
        Commands::Schema { output } => {
            let schema = mathir::json_schema::generate_mathir_schema();
            match output {
                Some(path) => fs::write(path, &schema)?,
                None => println!("{}", schema),
            }
        }
    }

    Ok(())
}

fn parse_input(input: &str, format: &InputFormat) -> Result<mathir::MathIR, Box<dyn std::error::Error>> {
    if let Ok(expr) = serde_json::from_str::<mathir::MathIR>(input) {
        return Ok(expr);
    }

    if let Ok(content) = fs::read_to_string(input) {
        if let Ok(expr) = serde_json::from_str::<mathir::MathIR>(&content) {
            return Ok(expr);
        }
    }

    match format {
        InputFormat::Natural => {
            Ok(mathir::NaturalParser::parse(input)?)
        }
        InputFormat::Latex | InputFormat::Auto => {
            let parser = mathir::parser::latex::LatexParser;
            Ok(mathir::parser::Parser::parse(&parser, input)?)
        }
        InputFormat::Sympy | InputFormat::Python => Err("SymPy parser not yet implemented".into()),
        InputFormat::Lean => Err("Lean parser not yet implemented".into()),
        InputFormat::Mathir => Err("Could not parse input as MathIR JSON".into()),
    }
}
