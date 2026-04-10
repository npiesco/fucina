use anyhow::{Context, Result, bail};
use clap::Parser;
use std::process::{Command, Stdio};

#[derive(Parser, Debug)]
#[command(name = "fucina")]
#[command(about = "Automated Rust repair pipeline")]
struct Args {
    /// Run in dry-run mode (show commands without executing)
    #[arg(long)]
    dry_run: bool,

    /// Skip tests
    #[arg(long)]
    no_test: bool,

    /// Allow features that may be unstable
    #[arg(long)]
    all_features: bool,
}

fn main() {
    let args = Args::parse();

    if let Err(e) = run_pipeline(&args) {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

fn run_pipeline(args: &Args) -> Result<()> {
    println!("Running Fucina repair pipeline...");

    verify_cargo()?;

    // Step 1: Format
    println!("Formatting...");
    run_cargo(&["fmt", "--all"], args.dry_run)?;

    // Step 2: Auto-fix with clippy
    println!("Auto-fixing...");
    let mut clippy_fix_args = vec![
        "clippy",
        "--fix",
        "--allow-dirty",
        "--allow-staged",
        "--all-targets",
    ];
    if args.all_features {
        clippy_fix_args.push("--all-features");
    }
    run_cargo(&clippy_fix_args, args.dry_run)?;

    // Step 3: Verify no remaining issues
    println!("Verifying...");
    let mut clippy_verify_args = vec!["clippy", "--all-targets", "--", "-D", "warnings"];
    if args.all_features {
        clippy_verify_args.insert(2, "--all-features");
    }
    run_cargo(&clippy_verify_args, args.dry_run)?;

    // Step 4: Tests
    if !args.no_test {
        println!("Testing...");
        let mut test_args = vec!["test"];
        if args.all_features {
            test_args.push("--all-features");
        }
        run_cargo(&test_args, args.dry_run)?;
    }

    println!("Fucina pipeline passed");
    Ok(())
}

fn verify_cargo() -> Result<()> {
    which::which("cargo").context("cargo not found in PATH. Is Rust installed?")?;
    Ok(())
}

fn run_cargo(args: &[&str], dry_run: bool) -> Result<()> {
    if dry_run {
        println!("  Would run: cargo {}", args.join(" "));
        return Ok(());
    }

    let status = Command::new("cargo")
        .args(args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .with_context(|| format!("Failed to execute: cargo {}", args.join(" ")))?;

    if !status.success() {
        bail!("cargo {} failed", args[0]);
    }

    Ok(())
}
