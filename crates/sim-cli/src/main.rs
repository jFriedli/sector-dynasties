//! Headless CLI runner for the simulation core.
//!
//! Usage:
//!   sim-cli run --seed <u64> [--years <u32>] [--json] [--save <path>]
//!   sim-cli run --preset <name> [--years <u32>] [--json] [--save <path>]
//!   sim-cli load --path <path> [--years <u32>] [--json]
//!
//! `run` starts a fresh game from a seed; `--save` writes the resulting
//! state to a plain JSON file. `--preset <name>` is a memorable alternative
//! to `--seed`: it resolves to a known (seed, system count) pair from
//! `sim_core::presets` so agents and reviewers don't have to guess raw u64
//! seeds for demo scenarios (see issue #21). `--seed` and `--preset` are
//! mutually exclusive. `load` resumes a save (written by `--save`, or by the
//! WASM bridge/UI, since they share the same JSON shape) and advances it
//! further. Exists so a bug report can be reduced to "seed X, N years, see
//! summary/state Y" - or "this exact save file, N more years" - without
//! touching the UI. See docs/ARCHITECTURE.md's persistence section.

use std::env;
use std::fs;
use std::process::ExitCode;

use sim_core::presets;
use sim_core::SimState;

#[derive(Debug)]
struct RunArgs {
    seed: u64,
    system_count: Option<u32>,
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
    let mut seed_given = false;
    let mut preset_name: Option<String> = None;
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
                seed_given = true;
            }
            "--preset" => {
                i += 1;
                preset_name = Some(raw.get(i).ok_or("--preset requires a name")?.clone());
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

    let system_count = match preset_name {
        Some(name) => {
            if seed_given {
                return Err("--seed and --preset are mutually exclusive".to_string());
            }
            let preset = presets::resolve(&name).ok_or_else(|| {
                format!(
                    "unknown preset \"{name}\", known presets: {}",
                    presets::names().join(", ")
                )
            })?;
            seed = preset.seed;
            Some(preset.system_count)
        }
        None => None,
    };

    Ok(RunArgs {
        seed,
        system_count,
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
    let mut state = match args.system_count {
        Some(system_count) => SimState::new_with_system_count(args.seed, system_count),
        None => SimState::new(args.seed),
    };
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
        _ => Err(format!(
            "usage: sim-cli run --seed <u64> [--years <u32>] [--json] [--save <path>]\n       sim-cli run --preset <name> [--years <u32>] [--json] [--save <path>]\n       sim-cli load --path <path> [--years <u32>] [--json]\nknown presets: {}",
            presets::names().join(", ")
        )),
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
            system_count: None,
            years: 5,
            json: true,
            save_path: None,
        };
        let args2 = RunArgs {
            seed: 100,
            system_count: None,
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
    fn preset_resolves_to_its_seed_and_system_count() {
        let raw = vec!["--preset".to_string(), "sprawling".to_string()];
        let args = parse_run_args(&raw).unwrap();
        let preset = sim_core::presets::resolve("sprawling").unwrap();
        assert_eq!(args.seed, preset.seed);
        assert_eq!(args.system_count, Some(preset.system_count));
    }

    #[test]
    fn unknown_preset_name_is_a_clean_error() {
        let raw = vec!["--preset".to_string(), "does-not-exist".to_string()];
        let err = parse_run_args(&raw).unwrap_err();
        assert!(err.contains("does-not-exist"));
    }

    #[test]
    fn seed_and_preset_together_is_a_clean_error() {
        let raw = vec![
            "--seed".to_string(),
            "1".to_string(),
            "--preset".to_string(),
            "tutorial".to_string(),
        ];
        let err = parse_run_args(&raw).unwrap_err();
        assert!(err.contains("mutually exclusive"));
    }

    #[test]
    fn running_a_preset_produces_a_non_empty_and_deterministic_summary() {
        let raw = vec!["--preset".to_string(), "tutorial".to_string()];
        let args_a = parse_run_args(&raw).unwrap();
        let args_b = parse_run_args(&raw).unwrap();
        let output_a = run(&args_a).unwrap();
        let output_b = run(&args_b).unwrap();
        assert_eq!(output_a, output_b);
        assert!(output_a.contains("systems=2"));
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
            system_count: None,
            years: 4,
            json: true,
            save_path: None,
        })
        .unwrap();

        run(&RunArgs {
            seed: 9,
            system_count: None,
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
