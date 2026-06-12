use rand::Rng;

pub fn roll_dice(number: i32) -> i32 {
    rand::rng().random_range(1..(number + 1))
}