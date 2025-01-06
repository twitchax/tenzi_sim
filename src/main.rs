#![feature(portable_simd)]
#![feature(test)]

extern crate test;

mod types;
mod rand;
mod mode;

use std::{simd::{cmp::SimdPartialEq, Simd}, sync::{atomic::{AtomicUsize, Ordering}, OnceLock}};

use clap::{arg, command, Parser};
use colored::Colorize;
use rand::roll;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use types::{AtomicNum, Float, Num};

fn main() {
    let args = Args::parse();

    let num_sides = args.sides;
    let num_dice = args.dice;
    let num_simulations = args.simulations;

    let strategy = match args.strategy.as_str() {
        "naive" => SimulationType::Naive(NaiveSimulation::new(num_sides, num_dice)),
        "divide" => SimulationType::Divide(DivideSimulation),
        "merge" => SimulationType::Merge(MergeSimulation),
        _ => panic!("Invalid strategy"),
    };

    println!("Running {} \"tenzi\" monte carlo simulations with {} {}-sided die, and strategy: `{}`.", num_simulations.to_string().cyan(), num_dice.to_string().cyan(), num_sides.to_string().cyan(), args.strategy.to_string().cyan());

    let output = monte_carlo(strategy, num_simulations);

    println!("Average rolls:      {:.8}.", output.average.to_string().green());
    println!("Standard deviation: {:.8}.", output.std_dev.to_string().yellow());
    println!("Duration:           {:.8}µs.", output.duration.as_micros().to_string().red());
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
    average: Float,
    std_dev: Float,
    duration: std::time::Duration,
}

/// Runs an entire monte carlo simulation.
/// Returns the average number of rolls it took to achieve a "tenzi", and
/// the standard deviation, and the clock time it took to run.
fn monte_carlo(strategy_type: SimulationType, num_simulations: Num) -> MonteCarloOutput {
    let total_rolls = AtomicNum::new(0);
    let total_squared_rolls = AtomicNum::new(0);

    let start = std::time::Instant::now();

    (0..num_simulations).into_par_iter().map(|_| {
        let rolls = sim(strategy_type.clone());
        (rolls, rolls * rolls)
    }).for_each(|(rolls, squared_rolls)| {
        total_rolls.fetch_add(rolls, Ordering::Relaxed);
        total_squared_rolls.fetch_add(squared_rolls, Ordering::Relaxed);
    });

    let total_rolls = total_rolls.load(Ordering::Relaxed);
    let total_squared_rolls = total_squared_rolls.load(Ordering::Relaxed);
    
    let average = (total_rolls as Float) / (num_simulations as Float);
    let variance = (total_squared_rolls as Float) / (num_simulations as Float) - (average * average as Float);
    let std_dev = variance.sqrt();

    let duration = start.elapsed();

    MonteCarloOutput {
        average,
        std_dev,
        duration,
    }
}

/// Returns the number of rolls it took to achieve a "tenzi".
fn sim(mut simulation_type: SimulationType) -> Num {
    let strategy = simulation_type.as_strategy_mut();

    let mut num_rolls = 0;

    while !strategy.done() {
        // Make another roll, and add to the count.
        num_rolls += strategy.roll();

        // Run a step.
        strategy.step();
    }

    num_rolls
}

// The strategy helpers.

#[derive(Clone)]
enum SimulationType {
    Naive(NaiveSimulation),
    Divide(DivideSimulation),
    Merge(MergeSimulation),
}

impl SimulationType {
    fn as_strategy(&self) -> &dyn Strategy {
        match self {
            SimulationType::Naive(sim) => sim as &dyn Strategy,
            SimulationType::Divide(sim) => sim as &dyn Strategy,
            SimulationType::Merge(sim) => sim as &dyn Strategy,
        }
    }

    fn as_strategy_mut(&mut self) -> &mut dyn Strategy {
        match self {
            SimulationType::Naive(sim) => sim as &mut dyn Strategy,
            SimulationType::Divide(sim) => sim as &mut dyn Strategy,
            SimulationType::Merge(sim) => sim as &mut dyn Strategy,
        }
    }
}

/// A simulation for the game "tenzi".
trait Simulation: Send + Sync {
    /// Returns a mutable reference to the dice.
    fn dice(&mut self) -> &mut [Num];

    /// Returns the number of sides on the die.
    fn num_sides(&self) -> Num;

    /// Returns whether or not a "tenzi" has been achieved.
    fn done(&self) -> bool;

    /// Rolls the dice, and returns the number rolled.
    fn roll(&mut self) -> Num {
        let num_sides = self.num_sides();
        let mut num_rolls = 0;

        for die in self.dice().iter_mut() {
            if *die == 0 {
                *die = roll(num_sides);
                num_rolls += 1;
            }
        }

        num_rolls
    }
}

/// A simulation strategy for the game "tenzi".
trait Strategy: Simulation {
    /// Takes the rolls, and returns the indexes to re-roll.
    /// Zeroes out the rolls that the strategy would like re-rolled.
    /// The dice that are not zeroed out are the ones that are kept.
    /// 
    /// We use this method as it prevents unnecessary allocations just to keep track of which dice to re-roll.
    fn step(&mut self);
}

/// Always keep the most from the first roll.
#[derive(Clone)]
struct NaiveSimulation {
    dice: Vec<Num>,
    num_sides: Num,
    mode: Option<Num>,
    done: bool,
}

impl NaiveSimulation {
    fn new(num_sides: Num, num_dice: Num) -> Self {
        Self {
            dice: vec![0; num_dice],
            num_sides,
            mode: None,
            done: false,
        }
    }
}

/// Keep the two most from the first roll.
#[derive(Clone)]
struct DivideSimulation;

/// Only roll the group(s) with the lowest amount.
#[derive(Clone)]
struct MergeSimulation;

impl Simulation for NaiveSimulation {
    fn dice(&mut self) -> &mut [Num] {
        &mut self.dice
    }

    fn num_sides(&self) -> Num {
        self.num_sides
    }

    fn done(&self) -> bool {
        self.done
    }
}

impl Strategy for NaiveSimulation {
    fn step(&mut self) {
        #[allow(unused_variables)]
        let num_sides = self.num_sides();
        
        let mode = self.mode.unwrap_or_else(|| {
            mode::get_mode(&mut self.dice, num_sides)
        });

        self.mode = Some(mode);

        let mut done = true;
        for roll in self.dice.iter_mut() {
            if *roll != mode {
                *roll = 0;
                done = false;
            }
        }

        self.done = done;
    }
}

impl Simulation for DivideSimulation {
    fn dice(&mut self) -> &mut [Num] {
        unimplemented!();
    }

    fn num_sides(&self) -> Num {
        unimplemented!();
    }

    fn done(&self) -> bool {
        unimplemented!();
    }
}

impl Strategy for DivideSimulation {
    fn step(&mut self) {
        unimplemented!();
    }
}

impl Simulation for MergeSimulation {
    fn dice(&mut self) -> &mut [Num] {
        unimplemented!();
    }

    fn num_sides(&self) -> Num {
        unimplemented!();
    }

    fn done(&self) -> bool {
        unimplemented!();
    }
}

impl Strategy for MergeSimulation {
    fn step(&mut self) {
        unimplemented!();
    }
}