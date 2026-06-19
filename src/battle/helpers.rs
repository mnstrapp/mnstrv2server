use rand::prelude::*;

pub fn roll_dice(number: i32) -> i32 {
    let mut rng = rand::rng();
    rng.random_range(1..(number + 1)) as i32
}