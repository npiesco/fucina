use anyhow::{Context, Result, bail};
use clap::Parser;
use std::path::{Path, PathBuf};
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

    /// Recursively find and repair all Rust projects under a directory
    #[arg(long, short)]
    recursive: bool,

    /// Root directory for recursive mode (defaults to current directory)
    #[arg(long, short, default_value = ".")]
    path: PathBuf,
}

fn main() {
    let args = Args::parse();

    if args.recursive {
        std::process::exit(run_recursive(&args));
    }

    if let Err(e) = run_pipeline(&args.path, &args) {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

fn run_recursive(args: &Args) -> i32 {
    let root = args
        .path
        .canonicalize()
        .unwrap_or_else(|_| args.path.clone());

    let projects = discover_projects(&root);
    if projects.is_empty() {
        eprintln!("No Cargo.toml files found under {}", root.display());
        return 1;
    }

    println!(
        "Found {} Rust project(s) under {}",
        projects.len(),
        root.display()
    );
    println!();

    let mut passed = 0usize;
    let mut failed: Vec<(PathBuf, String)> = Vec::new();

    for (i, project_dir) in projects.iter().enumerate() {
        let label = project_dir
            .strip_prefix(&root)
            .unwrap_or(project_dir)
            .display();
        println!("━━━ [{}/{}] {} ━━━", i + 1, projects.len(), label);

        match run_pipeline(project_dir, args) {
            Ok(()) => passed += 1,
            Err(e) => {
                eprintln!("✗ Failed: {e}");
                failed.push((project_dir.clone(), format!("{e}")));
            }
        }
        println!();
    }

    // Summary
    println!("━━━ Summary ━━━");
    println!(
        "  {} passed, {} failed, {} total",
        passed,
        failed.len(),
        projects.len()
    );
    for (dir, err) in &failed {
        let label = dir.strip_prefix(&root).unwrap_or(dir).display();
        eprintln!("  ✗ {label}: {err}");
    }

    if failed.is_empty() { 0 } else { 1 }
}

fn discover_projects(root: &Path) -> Vec<PathBuf> {
    let mut projects = Vec::new();
    walk_for_cargo(root, &mut projects);
    projects.sort();
    projects
}

fn walk_for_cargo(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    let has_cargo = dir.join("Cargo.toml").is_file();
    if has_cargo {
        out.push(dir.to_path_buf());
        // Don't recurse into subdirs of a project (nested crates are handled by cargo)
        return;
    }

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let name = entry.file_name();
            let skip = name == "target" || name == "node_modules" || name == ".git";
            if !skip {
                walk_for_cargo(&path, out);
            }
        }
    }
}

fn run_pipeline(project_dir: &Path, args: &Args) -> Result<()> {
    println!("Running Fucina repair pipeline...");

    verify_cargo()?;

    // Step 1: Format
    println!("Formatting...");
    run_cargo(project_dir, &["fmt", "--all"], args.dry_run)?;

    // Step 2: Auto-fix with clippy
    println!("Auto-fixing...");
    let mut clippy_fix_args = vec![
        "clippy",
        "--fix",
        "--allow-dirty",
        "--allow-staged",
        "--allow-no-vcs",
        "--all-targets",
    ];
    if args.all_features {
        clippy_fix_args.push("--all-features");
    }
    run_cargo(project_dir, &clippy_fix_args, args.dry_run)?;

    // Step 3: Verify no remaining issues
    println!("Verifying...");
    let mut clippy_verify_args = vec!["clippy", "--all-targets", "--", "-D", "warnings"];
    if args.all_features {
        clippy_verify_args.insert(2, "--all-features");
    }
    run_cargo(project_dir, &clippy_verify_args, args.dry_run)?;

    // Step 4: Tests
    if !args.no_test {
        println!("Testing...");
        let mut test_args = vec!["test"];
        if args.all_features {
            test_args.push("--all-features");
        }
        run_cargo(project_dir, &test_args, args.dry_run)?;
    }

    println!("✓ Pipeline passed");
    Ok(())
}

fn verify_cargo() -> Result<()> {
    which::which("cargo").context("cargo not found in PATH. Is Rust installed?")?;
    Ok(())
}

fn run_cargo(project_dir: &Path, args: &[&str], dry_run: bool) -> Result<()> {
    if dry_run {
        println!(
            "  Would run: cargo {} (in {})",
            args.join(" "),
            project_dir.display()
        );
        return Ok(());
    }

    let status = Command::new("cargo")
        .args(args)
        .current_dir(project_dir)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .with_context(|| format!("Failed to execute: cargo {}", args.join(" ")))?;

    if !status.success() {
        bail!("cargo {} failed", args[0]);
    }

    Ok(())
}
