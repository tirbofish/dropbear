use clap::Parser;
use magna_carta::generator::{Generator, jvm::KotlinJVMGenerator, native::KotlinNativeGenerator};
use magna_carta::{KotlinProcessor, ScriptManifest, Target};
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "magna-carta-cli")]
#[command(about = "Generate script manifests for Kotlin projects")]
struct Cli {
    #[arg(short, long, help = "Input directory containing Kotlin source files")]
    input: PathBuf,

    #[arg(
        short,
        long,
        help = "Output directory for generated files (ignored if --stdout is used)"
    )]
    output: Option<PathBuf>,

    #[arg(short, long, help = "Target platform")]
    target: Target,

    #[arg(
        long,
        help = "Print generated manifest to stdout instead of writing to file"
    )]
    stdout: bool,

    #[arg(long, help = "Print manifest raw")]
    raw: bool,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    if !(cli.stdout || cli.output.is_some() || cli.raw) {
        return Err(anyhow::anyhow!(
            "No output given. --stdout, --output <target> or --raw must be used."
        ));
    }

    if cli.stdout && cli.output.is_some() {
        return Err(anyhow::anyhow!(
            "--stdout and --output cannot be used together. Choose one output destination."
        ));
    }

    let mut processor = KotlinProcessor::new()?;
    let mut manifest = ScriptManifest::new();

    if !cli.input.exists() {
        return Err(anyhow::anyhow!(
            "Input directory does not exist: {:?}",
            cli.input
        ));
    }

    magna_carta::visit_kotlin_files(&cli.input, &mut processor, &mut manifest)?;

    let generated_content = match cli.target {
        Target::Jvm => {
            let generator = KotlinJVMGenerator;
            generator.generate(&manifest)?
        }
        Target::Native => {
            let generator = KotlinNativeGenerator;
            generator.generate(&manifest)?
        }
    };

    if cli.raw {
        println!("{:#?}", manifest);
    }

    if cli.stdout {
        print!("{}", generated_content);
    } else if let Some(output_dir) = cli.output {
        fs::create_dir_all(&output_dir)?;

        let filename = match cli.target {
            Target::Jvm => "RunnableRegistry.kt",
            Target::Native => "ScriptManifest.kt",
        };
        let output_path = output_dir.join(filename);
        fs::write(&output_path, generated_content)?;
        println!(
            "Generated {:?} manifest at: {}",
            cli.target,
            output_path.display()
        );
    }

    println!("Found {} script classes", manifest.items().len());
    Ok(())
}
