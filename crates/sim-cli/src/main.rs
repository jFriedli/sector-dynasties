//! Headless CLI runner for the simulation core.
//!
//! Usage:
//!   sim-cli run --seed <u64> [--years <u32>] [--json]
//!
//! Prints a one-line human summary by default, or the full serialized
//! state with --json. Exists so a bug report can be reduced to
//! "seed X, N years, see summary/state Y" without touching the UI.

use std::env;
use std::process::ExitCode;

use sim_core::SimState;

struct Args {
    seed: u64,
    years: u32,
    json: bool,
}

fn parse_args(raw: &[String]) -> Result<Args, String> {
    let mut seed = 42u64;
    let mut years = 1u32;
    let mut json = false;

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
            other => return Err(format!("unrecognized argument: {other}")),
        }
        i += 1;
    }

    Ok(Args { seed, years, json })
}

fn run(args: &Args) -> String {
    let mut state = SimState::new(args.seed);
    state.step_days(args.years * 360);

    let violations = sim_core::invariants::check_invariants(&state);
    if !violations.is_empty() {
        eprintln!("invariant violations after run:");
        for v in &violations {
            eprintln!("  - {}", v.0);
        }
    }

    if args.json {
        state.to_json()
    } else {
        let s = state.summary();
        format!(
            "seed={} tick={} year={} systems={} cities={} population={} dynasty=\"{}\" wealth={:.2}",
            args.seed, s.tick, s.year, s.system_count, s.city_count, s.total_population, s.dynasty_name, s.dynasty_wealth
        )
    }
}

fn main() -> ExitCode {
    let raw: Vec<String> = env::args().skip(1).collect();

    if raw.first().map(String::as_str) != Some("run") {
        eprintln!("usage: sim-cli run --seed <u64> [--years <u32>] [--json]");
        return ExitCode::FAILURE;
    }

    match parse_args(&raw[1..]) {
        Ok(args) => {
            println!("{}", run(&args));
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
        let args = Args {
            seed: 100,
            years: 5,
            json: true,
        };
        let args2 = Args {
            seed: 100,
            years: 5,
            json: true,
        };
        assert_eq!(run(&args), run(&args2));
    }

    #[test]
    fn parses_flags_in_any_order() {
        let raw = vec![
            "--years".to_string(),
            "10".to_string(),
            "--seed".to_string(),
            "7".to_string(),
            "--json".to_string(),
        ];
        let args = parse_args(&raw).unwrap();
        assert_eq!(args.seed, 7);
        assert_eq!(args.years, 10);
        assert!(args.json);
    }
}
