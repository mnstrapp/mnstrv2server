use crate::models::mnstr::Mnstr;
use crate::battle::helpers::roll_dice;

pub fn rest(defender: &mut Mnstr) -> i32 {
    let mut defense = roll_dice(20)as i32;
    if (defense + defender.current_defense) >= defender.max_defense {
        defense = defender.max_defense;
    }

    defender.current_defense += defense;
    if defender.current_defense > defender.max_defense {
        defender.current_defense = defender.max_defense;
    }

    defender.current_intelligence += defense;
    if defender.current_intelligence > defender.max_intelligence {
        defender.current_intelligence = defender.max_intelligence;
    }

    defense
}