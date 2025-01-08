use crate::{mode, rand::roll, types::Num};

// Primary enum.

#[derive(Clone)]
pub enum SimulationType {
    Naive(NaiveSimulation),
    Divide(DivideSimulation),
    Merge(MergeSimulation),
}

impl SimulationType {
    pub fn as_strategy_mut(&mut self) -> &mut dyn Strategy {
        match self {
            SimulationType::Naive(sim) => sim as &mut dyn Strategy,
            SimulationType::Divide(sim) => sim as &mut dyn Strategy,
            SimulationType::Merge(sim) => sim as &mut dyn Strategy,
        }
    }
}

// Traits.

/// A trait for a simulator that has "tracked" values.
pub trait Tracked: Send + Sync {
    /// Returns the number of rolls.
    fn num_rolls(&self) -> Num;

    /// Returns the number of steps.
    fn num_steps(&self) -> Num;

    /// Returns whether or not a "tenzi" has been achieved.
    fn done(&self) -> bool;
}

/// A trait for a simulator that allows "tracked" values to be set.
trait SetTracked: Tracked {
    /// Sets the number of rolls.
    fn set_num_rolls(&mut self, num_rolls: Num);

    /// Sets the number of steps.
    fn set_num_steps(&mut self, num_steps: Num);

    /// Sets whether or not a "tenzi" has been achieved.
    fn set_done(&mut self, done: bool);
}

/// A simulation for the game "tenzi".
trait Simulation: Tracked + SetTracked {
    /// Returns a mutable reference to the dice.
    fn buckets(&mut self) -> &mut [Num];

    /// Returns the number of sides on the die.
    fn num_sides(&self) -> Num;

    /// Returns the number of dice to roll.
    fn num_to_roll(&self) -> Num;
}

/// A simulation strategy for the game "tenzi".
#[allow(private_bounds)]
pub trait Strategy: Simulation {
    /// Rolls the dice, and returns the number rolled.
    fn roll(&mut self) {
        let num_to_roll = self.num_to_roll();
        let num_sides = self.num_sides();
        let buckets = self.buckets();

        let mut num_rolls = 0;

        for _ in 0..num_to_roll {
            let roll = roll(num_sides);
            buckets[roll - 1] += 1;
            num_rolls += 1;
        }

        self.set_num_rolls(self.num_rolls() + num_rolls);
    }
    
    /// Takes the rolls, and returns the indexes to re-roll.
    /// Zeroes out the rolls that the strategy would like re-rolled.
    /// The dice that are not zeroed out are the ones that are kept.
    /// 
    /// We use this method as it prevents unnecessary allocations just to keep track of which dice to re-roll.
    fn step(&mut self);
}

// Declarative macros for the different simulation strategies.

macro_rules! impl_tracked {
    ($type:ty) => {
        impl Tracked for $type {
            fn num_rolls(&self) -> Num {
                self.num_rolls
            }

            fn num_steps(&self) -> Num {
                self.num_steps
            }

            fn done(&self) -> bool {
                self.done
            }
        }
    };
}

macro_rules! impl_set_tracked {
    ($type:ty) => {
        impl SetTracked for $type {
            fn set_num_rolls(&mut self, num_rolls: Num) {
                self.num_rolls = num_rolls;
            }

            fn set_num_steps(&mut self, num_steps: Num) {
                self.num_steps = num_steps;
            }

            fn set_done(&mut self, done: bool) {
                self.done = done;
            }
        }
    };
}

macro_rules! impl_simulation {
    ($type:ty) => {
        impl Simulation for $type {
            fn buckets(&mut self) -> &mut [Num] {
                &mut self.buckets
            }
        
            fn num_sides(&self) -> Num {
                self.num_sides
            }

            fn num_to_roll(&self) -> Num {
                self.num_to_roll
            }
        }
    };
}

/// Always keep the most from the first roll.
#[derive(Clone)]
pub struct NaiveSimulation {
    buckets: Vec<Num>,
    num_dice: Num,
    num_sides: Num,
    num_to_roll: Num,

    num_rolls: Num,
    num_steps: Num,
    mode: Option<Num>,
    done: bool,
}

impl NaiveSimulation {
    pub fn new(num_sides: Num, num_dice: Num) -> Self {
        Self {
            buckets: vec![0; num_sides],
            num_dice,
            num_sides,
            num_to_roll: num_dice,

            num_rolls: 0,
            num_steps: 0,
            mode: None,
            done: false,
        }
    }
}

/// Keep the two most from the first roll.
#[derive(Clone)]
pub struct DivideSimulation {
    buckets: Vec<Num>,
    num_dice: Num,
    num_sides: Num,
    num_to_roll: Num,

    num_rolls: Num,
    num_steps: Num,
    done: bool,
}

impl DivideSimulation {
    pub fn new(num_sides: Num, num_dice: Num) -> Self {
        Self {
            buckets: vec![0; num_sides],
            num_dice,
            num_sides,
            num_to_roll: num_dice,

            num_rolls: 0,
            num_steps: 0,
            done: false,
        }
    }
}

/// Only roll the group(s) with the lowest amount.
#[derive(Clone)]
pub struct MergeSimulation {
    buckets: Vec<Num>,
    num_dice: Num,
    num_sides: Num,
    num_to_roll: Num,

    num_rolls: Num,
    num_steps: Num,
    done: bool,
}

impl MergeSimulation {
    pub fn new(num_sides: Num, num_dice: Num) -> Self {
        Self {
            buckets: vec![0; num_sides],
            num_dice,
            num_sides,
            num_to_roll: num_dice,

            num_rolls: 0,
            num_steps: 0,
            done: false,
        }
    }
}

// Implementations.

// NaiveSimulation.

impl_tracked!(NaiveSimulation);
impl_set_tracked!(NaiveSimulation);
impl_simulation!(NaiveSimulation);

impl Strategy for NaiveSimulation {
    fn step(&mut self) {
        // Perform a roll.

        self.roll();

        // Get the mode, and cache it.

        let mode = self.mode.unwrap_or_else(|| {
            mode::mode_from_counts(&self.buckets)
        });

        self.mode = Some(mode);
        let mode_bucket = mode - 1;

        // Zero out the buckets that are not the mode.

        for k in 0..self.buckets.len() {
            if k != mode_bucket {
                self.buckets[k] = 0;
            }
        }

        // Check if we are done; otherwise, compute the number to roll on the next step (i.e., the total dice that are not in the mode bucket).

        if self.buckets[mode_bucket] == self.num_dice {
            self.set_done(true);
        } else {
            self.num_to_roll = self.num_dice - self.buckets[mode_bucket];
        }

        // Update the state.

        self.set_num_steps(self.num_steps() + 1);
    }
}

// DivideSimulation.

impl_tracked!(DivideSimulation);
impl_set_tracked!(DivideSimulation);
impl_simulation!(DivideSimulation);

impl Strategy for DivideSimulation {
    fn step(&mut self) {
        // Perform a roll.

        self.roll();

        // Get the modes.  Need to compute every time, as it may change.

        let (mode1, mode2) = mode::top_two_modes_from_counts(&self.buckets);

        // As soon as one of the modes passes the midpoint, let's then move forward with only that one.

        let (mode1_bucket, mode2_bucket) = if self.buckets[mode1 - 1] >= self.num_dice / 2 {
            (mode1 - 1, mode1 - 1)
        } else {
            (mode1 - 1, mode2 - 1)
        };

        // Zero out the buckets that are not the modes.

        for k in 0..self.buckets.len() {
            if k != mode1_bucket && k != mode2_bucket {
                self.buckets[k] = 0;
            }
        }

        // Check if we are done; otherwise, compute the number to roll on the next step (i.e., the total dice that are not in the mode bucket).

        let num_to_keep = self.buckets.iter().sum::<Num>();

        if num_to_keep == self.num_dice {
            self.set_done(true);
        } else {
            self.num_to_roll = self.num_dice - num_to_keep;
        }

        // Update the state.

        self.set_num_steps(self.num_steps() + 1);
    }
}

// MergeSimulation.

impl_tracked!(MergeSimulation);
impl_set_tracked!(MergeSimulation);
impl_simulation!(MergeSimulation);

impl Strategy for MergeSimulation {
    fn step(&mut self) {
        // Perform a roll.

        self.roll();

        // Find the anti-modes.

        let anti_modes = mode::anti_modes(&self.buckets);

        // Zero out the buckets that are anti modes.

        for k in anti_modes {
            self.buckets[k - 1] = 0;
        }

        // Check if we are done; otherwise, compute the number to roll on the next step (i.e., the total dice that are not in the mode bucket).

        let num_to_keep = self.buckets.iter().sum::<Num>();

        if num_to_keep == self.num_dice {
            self.set_done(true);
        } else {
            self.num_to_roll = self.num_dice - num_to_keep;
        }

        // Update the state.

        self.set_num_steps(self.num_steps() + 1);
    }
}

// Tests.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_naive_simulation() {
        let num_sides = 6;
        let num_dice = 10;
        let mut sim = NaiveSimulation::new(num_sides, num_dice);

        let expected_mode = 5;
        let expected_steps = 20;
        let expected_rols = 58;

        while !sim.done() {
            sim.step();
        }

        let mode = sim.mode.unwrap();

        assert_eq!(mode, expected_mode);
        assert_eq!(sim.num_steps(), expected_steps);
        assert_eq!(sim.num_rolls(), expected_rols);
    }

    #[test]
    fn test_naive_simulation_step() {
        let num_sides = 6;
        let num_dice = 10;
        let mut sim = NaiveSimulation::new(num_sides, num_dice);
        
        assert_eq!(sim.buckets(), &[0, 0, 0, 0, 0, 0]);
        sim.step();
        assert_eq!(sim.buckets(), &[0, 0, 0, 0, 3, 0]);
        sim.step();
        assert_eq!(sim.buckets(), &[0, 0, 0, 0, 5, 0]);
        sim.step();
        assert_eq!(sim.buckets(), &[0, 0, 0, 0, 6, 0]);
    }

    #[test]
    fn test_divide_simulation() {
        let num_sides = 6;
        let num_dice = 20;
        let mut sim = DivideSimulation::new(num_sides, num_dice);

        let expected_steps = 26;
        let expected_rols = 129;

        while !sim.done() {
            sim.step();
        }

        assert_eq!(sim.num_steps(), expected_steps);
        assert_eq!(sim.num_rolls(), expected_rols);
    }

    #[test]
    fn test_divide_simulation_step() {
        let num_sides = 6;
        let num_dice = 20;
        let mut sim = DivideSimulation::new(num_sides, num_dice);
        
        assert_eq!(sim.buckets(), &[0, 0, 0, 0, 0, 0]);
        sim.step();
        assert_eq!(sim.buckets(), &[0, 0, 0, 4, 6, 0]);
        sim.step();
        assert_eq!(sim.buckets(), &[0, 0, 0, 6, 7, 0]);
        sim.step();
        assert_eq!(sim.buckets(), &[0, 0, 0, 7, 7, 0]);
        sim.step();
        assert_eq!(sim.buckets(), &[0, 0, 0, 9, 7, 0]);
        sim.step();
        assert_eq!(sim.buckets(), &[0, 0, 0, 11, 0, 0]);
    }

    #[test]
    fn test_merge_simulation() {
        let num_sides = 6;
        let num_dice = 20;
        let mut sim = MergeSimulation::new(num_sides, num_dice);

        let expected_steps = 46;
        let expected_rols = 111;

        while !sim.done() {
            sim.step();
        }

        assert_eq!(sim.num_steps(), expected_steps);
        assert_eq!(sim.num_rolls(), expected_rols);
    }

    #[test]
    fn test_merge_simulation_step() {
        let num_sides = 6;
        let num_dice = 20;
        let mut sim = MergeSimulation::new(num_sides, num_dice);
        
        assert_eq!(sim.buckets(), &[0, 0, 0, 0, 0, 0]);
        sim.step();
        assert_eq!(sim.buckets(), &[3, 3, 3, 4, 6, 0]);
        sim.step();
        assert_eq!(sim.buckets(), &[0, 4, 0, 4, 6, 0]);
        sim.step();
        assert_eq!(sim.buckets(), &[2, 5, 0, 5, 7, 0]);
        sim.step();
        assert_eq!(sim.buckets(), &[2, 5, 0, 5, 7, 0]);
        sim.step();
        assert_eq!(sim.buckets(), &[2, 5, 0, 5, 7, 0]);
        sim.step();
        assert_eq!(sim.buckets(), &[0, 5, 0, 6, 7, 0]);
        sim.step();
        assert_eq!(sim.buckets(), &[0, 6, 0, 6, 7, 0]);
        sim.step();
        assert_eq!(sim.buckets(), &[0, 7, 0, 0, 7, 0]);
        sim.step();
        assert_eq!(sim.buckets(), &[0, 10, 0, 0, 7, 0]);
        sim.step();
        assert_eq!(sim.buckets(), &[0, 11, 0, 0, 7, 0]);
    }

    #[bench]
    fn bench_naive_simulation(b: &mut test::Bencher) {
        let num_sides = 100;
        let num_dice = 1_000;

        b.iter(|| {
            let mut sim = NaiveSimulation::new(num_sides, num_dice);

            while !sim.done() {
                sim.step();
            }
        });
    }

    #[bench]
    fn bench_divide_simulation(b: &mut test::Bencher) {
        let num_sides = 100;
        let num_dice = 1_000;

        b.iter(|| {
            let mut sim = DivideSimulation::new(num_sides, num_dice);

            while !sim.done() {
                sim.step();
            }
        });
    }

    #[bench]
    fn bench_merge_simulation(b: &mut test::Bencher) {
        let num_sides = 100;
        let num_dice = 1_000;

        b.iter(|| {
            let mut sim = MergeSimulation::new(num_sides, num_dice);

            while !sim.done() {
                sim.step();
            }
        });
    }
}