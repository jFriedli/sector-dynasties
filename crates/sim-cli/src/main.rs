//! Headless CLI runner for the simulation core.
//!
//! Usage:
//!   sim-cli run --seed <u64> [--years <u32>] [--json] [--save <path>]
//!   sim-cli load --path <path> [--years <u32>] [--json]
//!
//! `run` starts a fresh game from a seed; `--save` writes the resulting
//! state to a plain JSON file. `load` resumes a save (written by `--save`,
//! or by the WASM bridge/UI, since they share the same JSON shape) and
//! advances it further. Exists so a bug report can be reduced to "seed X,
//! N years, see summary/state Y" - or "this exact save file, N more years" -
//! without touching the UI. See docs/ARCHITECTURE.md's persistence section.

use std::env;
use std::fs;
use std::process::ExitCode;

use sim_core::SimState;

struct RunArgs {
    seed: u64,
    years: u32,
    json: bool,
    save_path: Option<String>,
}

struct LoadArgs {
    path: String,
    years: u32,
    json: bool,
}

fn parse_run_args(raw: &[String]) -> Result<RunArgs, String> {
    let mut seed = 42u64;
    let mut years = 1u32;
    let mut json = false;
    let mut save_path = None;

    let mut i = 0;
    while i < raw.len() {
        match raw[i].as_str() {
            "--seed" => {
                i += 1;
                seed = raw
                    .get(i)
                    .ok_or("--seed requires a value")?
                    .parse()
                    .map_err(|_| "--seed must be a u64")?;
            }
            "--years" => {
                i += 1;
                years = raw
                    .get(i)
                    .ok_or("--years requires a value")?
                    .parse()
                    .map_err(|_| "--years must be a u32")?;
            }
            "--json" => json = true,
            "--save" => {
                i += 1;
                save_path = Some(raw.get(i).ok_or("--save requires a path")?.clone());
            }
            other => return Err(format!("unrecognized argument: {other}")),
        }
        i += 1;
    }

    Ok(RunArgs {
        seed,
        years,
        json,
        save_path,
    })
}

fn parse_load_args(raw: &[String]) -> Result<LoadArgs, String> {
    let mut path = None;
    let mut years = 0u32;
    let mut json = false;

    let mut i = 0;
    while i < raw.len() {
        match raw[i].as_str() {
            "--path" => {
                i += 1;
                path = Some(raw.get(i).ok_or("--path requires a value")?.clone());
            }
            "--years" => {
                i += 1;
                years = raw
                    .get(i)
                    .ok_or("--years requires a value")?
                    .parse()
                    .map_err(|_| "--years must be a u32")?;
            }
            "--json" => json = true,
            other => return Err(format!("unrecognized argument: {other}")),
        }
        i += 1;
    }

    Ok(LoadArgs {
        path: path.ok_or("load requires --path <file>")?,
        years,
        json,
    })
}

fn summarize(state: &SimState) -> String {
    let violations = sim_core::invariants::check_invariants(state);
    if !violations.is_empty() {
        eprintln!("invariant violations after run:");
        for v in &violations {
            eprintln!("  - {}", v.0);
        }
    }

    let s = state.summary();
    format!(
        "tick={} year={} systems={} cities={} population={} dynasty=\"{}\" wealth={:.2}",
        s.tick,
        s.year,
        s.system_count,
        s.city_count,
        s.total_population,
        s.dynasty_name,
        s.dynasty_wealth
    )
}

fn run(args: &RunArgs) -> Result<String, String> {
    let mut state = SimState::new(args.seed);
    state.step_days(args.years * 360);

    if let Some(path) = &args.save_path {
        fs::write(path, state.to_json())
            .map_err(|e| format!("failed to write save to {path}: {e}"))?;
    }

    Ok(if args.json {
        state.to_json()
    } else {
        format!("seed={} {}", args.seed, summarize(&state))
    })
}

fn load(args: &LoadArgs) -> Result<String, String> {
    let json =
        fs::read_to_string(&args.path).map_err(|e| format!("failed to read {}: {e}", args.path))?;
    let mut state =
        SimState::from_json(&json).map_err(|e| format!("failed to load {}: {e}", args.path))?;
    state.step_days(args.years * 360);

    Ok(if args.json {
        state.to_json()
    } else {
        summarize(&state)
    })
}

fn main() -> ExitCode {
    let raw: Vec<String> = env::args().skip(1).collect();

    let result = match raw.first().map(String::as_str) {
        Some("run") => parse_run_args(&raw[1..]).and_then(|a| run(&a)),
        Some("load") => parse_load_args(&raw[1..]).and_then(|a| load(&a)),
        _ => Err(
            "usage: sim-cli run --seed <u64> [--years <u32>] [--json] [--save <path>]\n       sim-cli load --path <path> [--years <u32>] [--json]"
                .to_string(),
        ),
    };

    match result {
        Ok(output) => {
            println!("{output}");
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_and_years_produce_identical_output() {
        let args = RunArgs {
            seed: 100,
            years: 5,
            json: true,
            save_path: None,
        };
        let args2 = RunArgs {
            seed: 100,
            years: 5,
            json: true,
            save_path: None,
        };
        assert_eq!(run(&args).unwrap(), run(&args2).unwrap());
    }

    #[test]
    fn parses_run_flags_in_any_order() {
        let raw = vec![
            "--years".to_string(),
            "10".to_string(),
            "--seed".to_string(),
            "7".to_string(),
            "--json".to_string(),
        ];
        let args = parse_run_args(&raw).unwrap();
        assert_eq!(args.seed, 7);
        assert_eq!(args.years, 10);
        assert!(args.json);
    }

    #[test]
    fn parses_load_flags() {
        let raw = vec![
            "--path".to_string(),
            "save.json".to_string(),
            "--years".to_string(),
            "3".to_string(),
        ];
        let args = parse_load_args(&raw).unwrap();
        assert_eq!(args.path, "save.json");
        assert_eq!(args.years, 3);
        assert!(!args.json);
    }

    #[test]
    fn saving_and_loading_a_file_continues_the_same_future_as_an_uninterrupted_run() {
        let path = std::env::temp_dir().join(format!("sim-cli-test-{}.json", std::process::id()));
        let path_str = path.to_str().unwrap().to_string();

        let uninterrupted = run(&RunArgs {
            seed: 9,
            years: 4,
            json: true,
            save_path: None,
        })
        .unwrap();

        run(&RunArgs {
            seed: 9,
            years: 2,
            json: false,
            save_path: Some(path_str.clone()),
        })
        .unwrap();
        let resumed = load(&LoadArgs {
            path: path_str.clone(),
            years: 2,
            json: true,
        })
        .unwrap();

        std::fs::remove_file(&path_str).ok();
        assert_eq!(uninterrupted, resumed);
    }

    #[test]
    fn loading_a_missing_file_is_a_clean_error_not_a_panic() {
        let result = load(&LoadArgs {
            path: "/nonexistent/path/does-not-exist.json".to_string(),
            years: 0,
            json: false,
        });
        assert!(result.is_err());
    }

    #[test]
    fn loading_a_corrupt_file_is_a_clean_error_not_a_panic() {
        let path =
            std::env::temp_dir().join(format!("sim-cli-corrupt-test-{}.json", std::process::id()));
        let path_str = path.to_str().unwrap().to_string();
        std::fs::write(&path_str, "not valid json").unwrap();

        let result = load(&LoadArgs {
            path: path_str.clone(),
            years: 0,
            json: false,
        });

        std::fs::remove_file(&path_str).ok();
        assert!(result.is_err());
    }
}
