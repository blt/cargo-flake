use cargo_flake::{parse_test_names, Config, TestResult, TestSetup};
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::process::{Command, Stdio};
use std::str;
use tabular::{Row, Table};

pub fn get_test_names(config: &Config) -> Result<Vec<String>, std::io::Error> {
    let list_cmd = if let Some(ref features) = config.features {
        format!("cargo test --features \"{}\" -- --list", features)
    } else {
        "cargo test -- --list".into()
    };

    let output = Command::new("sh").arg("-c").arg(list_cmd).output()?;
    let stdout = str::from_utf8(&output.stdout).expect("stdout not utf8");
    Ok(parse_test_names(stdout))
}

pub fn run_single_test(setup: TestSetup) -> Result<TestResult, std::io::Error> {
    let range = 0..(setup.iterations as usize);

    let result = range
        .into_par_iter()
        .fold_with(TestResult::new(setup.name.clone()), |mut result, _| {
            let status = Command::new("sh")
                .arg("-c")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .arg(&setup.command)
                .status()
                .unwrap();
            result.iterations += 1;
            if status.success() {
                result.successes += 1;
            } else {
                result.failures += 1;
            }
            result
        })
        .reduce(
            || TestResult::new(setup.name.clone()),
            |mut l: TestResult, r: TestResult| {
                l.iterations += r.iterations;
                l.successes += r.successes;
                l.failures += r.failures;
                l
            },
        );

    Ok(result)
}

fn main() -> Result<(), std::io::Error> {
    let cpus = num_cpus::get();
    let config: Config = argh::from_env();
    let tolerable_failures = config.tolerable_failures.unwrap_or(0);

    rayon::ThreadPoolBuilder::new()
        .num_threads(config.threads.unwrap_or(cpus))
        .build_global()
        .unwrap();

    let names = get_test_names(&config)?;
    let total_tests = names.len() as u64;
    let bar = ProgressBar::new(total_tests);
    bar.enable_steady_tick(100);
    bar.set_style(
        ProgressStyle::default_bar().template("[{elapsed_precise}] {wide_bar} {pos}/{len}"),
    );

    let results: Vec<TestResult> = names
        .into_par_iter()
        .progress_with(bar)
        .map(|name| {
            let cargo_cmd = if let Some(ref features) = config.features {
                format!("cargo test --features {} {}", features, name)
            } else {
                format!("cargo test {}", name)
            };

            let setup = TestSetup {
                name,
                command: cargo_cmd,
                iterations: config.iterations.unwrap_or(1_000),
            };

            run_single_test(setup).unwrap()
        })
        .filter(|result| result.failures > tolerable_failures)
        .collect();

    if !results.is_empty() {
        let mut table = Table::new("{:<}    {:<}    {:<}");
        table.add_row(
            Row::new()
                .with_cell("FAILURES")
                .with_cell("SUCCESSES")
                .with_cell("TEST"),
        );
        for result in results {
            table.add_row(
                Row::new()
                    .with_cell(result.failures)
                    .with_cell(result.successes)
                    .with_cell(result.name),
            );
        }
        println!("{}", table);
    } else {
        println!("no flakey tests detected")
    }

    Ok(())
}
