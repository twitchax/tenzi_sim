#![feature(portable_simd)]
#![feature(once_cell_get_mut)]
#![feature(test)]

extern crate test;

mod types;
mod rand;
mod mode;
mod simulation;

use std::sync::atomic::Ordering;

use clap::{arg, command, Parser};
use colored::Colorize;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use simulation::{DivideSimulation, MergeSimulation, NaiveSimulation, SimulationType};
use types::{AtomicNum, Float, Num};

fn main() {
    let args = Args::parse();

    let num_sides = args.sides;
    let num_dice = args.dice;
    let num_simulations = args.simulations;

    let strategy = match args.strategy.as_str() {
        "naive" => SimulationType::Naive(NaiveSimulation::new(num_sides, num_dice)),
        "divide" => SimulationType::Divide(DivideSimulation::new(num_sides, num_dice)),
        "merge" => SimulationType::Merge(MergeSimulation::new(num_sides, num_dice)),
        _ => panic!("Invalid strategy"),
    };

    println!("Running {} \"tenzi\" monte carlo simulations with {} {}-sided die, and strategy: `{}`.", num_simulations.to_string().cyan(), num_dice.to_string().cyan(), num_sides.to_string().cyan(), args.strategy.to_string().cyan());

    let output = monte_carlo(strategy, num_simulations);

    println!("Average rolls:            {:.8}.", output.average_rolls.to_string().green());
    println!("Standard deviation rolls: {:.8}.", output.std_dev_rolls.to_string().yellow());
    println!("Average steps:            {:.8}.", output.average_steps.to_string().green());
    println!("Standard deviation steps: {:.8}.", output.std_dev_steps.to_string().yellow());
    println!("Duration:                 {:.8}µs.", output.duration.as_micros().to_string().red());
}

/// A monte carlo simulator for the game "tenzi".
#[derive(Parser, Debug)]
#[command(version, about, long_about)]
struct Args {
    /// The number of sides on each die.
    #[arg(short, long, default_value_t = 6)]
    sides: Num,

    /// The number of die to roll.
    #[arg(short, long, default_value_t = 10)]
    dice: Num,

    /// The number of simulations to run.
    #[arg(short = 'm', long, default_value_t = 10_000)]
    simulations: Num,

    /// The strategy to use.
    /// Options are "naive", "divide", and "merge".
    /// The default is "naive".
    #[arg(short = 't', long, default_value = "naive")]
    strategy: String,
}

/// The output of a monte carlo simulation.
/// Contains the average number of rolls it took to achieve a "tenzi",
/// and the standard deviation, and the clock time it took to run.
struct MonteCarloOutput {
    average_rolls: Float,
    std_dev_rolls: Float,
    average_steps: Float,
    std_dev_steps: Float,
    duration: std::time::Duration,
}

/// Runs an entire monte carlo simulation.
/// Returns the average number of rolls it took to achieve a "tenzi", and
/// the standard deviation, and the clock time it took to run.
fn monte_carlo(strategy_type: SimulationType, num_simulations: Num) -> MonteCarloOutput {
    let total_rolls = AtomicNum::new(0);
    let total_squared_rolls = AtomicNum::new(0);
    let total_steps = AtomicNum::new(0);
    let total_squared_steps = AtomicNum::new(0);

    let start = std::time::Instant::now();

    (0..num_simulations).into_par_iter().map(|_| {
        let (rolls, steps) = sim(strategy_type.clone());
        (rolls, rolls * rolls, steps, steps * steps)
    }).for_each(|(rolls, squared_rolls, steps, squared_steps)| {
        total_rolls.fetch_add(rolls, Ordering::Relaxed);
        total_squared_rolls.fetch_add(squared_rolls, Ordering::Relaxed);
        total_steps.fetch_add(steps, Ordering::Relaxed);
        total_squared_steps.fetch_add(squared_steps, Ordering::Relaxed);
    });

    let total_rolls = total_rolls.load(Ordering::Relaxed);
    let total_squared_rolls = total_squared_rolls.load(Ordering::Relaxed);
    let total_steps = total_steps.load(Ordering::Relaxed);
    let total_squared_steps = total_squared_steps.load(Ordering::Relaxed);
    
    let average_rolls = (total_rolls as Float) / (num_simulations as Float);
    let variance_rolls = (total_squared_rolls as Float) / (num_simulations as Float) - (average_rolls * average_rolls as Float);
    let std_dev_rolls = variance_rolls.sqrt();

    let average_steps = (total_steps as Float) / (num_simulations as Float);
    let variance_steps = (total_squared_steps as Float) / (num_simulations as Float) - (average_steps * average_steps as Float);
    let std_dev_steps = variance_steps.sqrt();


    let duration = start.elapsed();

    MonteCarloOutput {
        average_rolls,
        std_dev_rolls,
        average_steps,
        std_dev_steps,
        duration,
    }
}

/// Returns the number of rolls it took to achieve a "tenzi".
fn sim(mut simulation_type: SimulationType) -> (Num, Num) {
    let strategy = simulation_type.as_strategy_mut();

    while !strategy.done() {
        // Run a step.
        strategy.step();
    }

    (strategy.num_rolls(), strategy.num_steps())
}