use std::{fs, io::Write};

fn main() {
    generate_level_xp();
    generate_mnstr_xp();
}

fn generate_level_xp() {
    let mut levels = vec![];
    levels.push(0);
    levels.push(100);
    for i in 2..101 {
        let previous_xp = levels[i - 1];
        let xp = (previous_xp + ((previous_xp as f64).log10() * 100.0).ceil() as i32) as i32;
        levels.push(xp);
    }
    let ouput = format!("pub const XP_FOR_LEVEL: [i32; 101] = {:?};", levels);
    let mut file = fs::File::create("src/models/generated/level_xp.rs").unwrap();
    file.write_all(ouput.as_bytes()).unwrap();
}

fn generate_mnstr_xp() {
    let mut levels = vec![];
    levels.push(50);
    for i in 1..101 {
        let previous_xp = levels[i - 1];
        let xp = (previous_xp + ((previous_xp as f64).log10() * 10.0).ceil() as i32) as i32;
        levels.push(xp);
    }
    let ouput = format!("pub const XP_FOR_LEVEL: [i32; 101] = {:?};", levels);
    let mut file = fs::File::create("src/models/generated/mnstr_xp.rs").unwrap();
    file.write_all(ouput.as_bytes()).unwrap();
}
