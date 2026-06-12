use crate::models::mnstr::Mnstr;
use crate::battle::helpers::roll_dice;

pub fn attack(attacker: &mut Mnstr, defender: &mut Mnstr) -> (bool, i32) {
    let attacker_roll = roll_dice(20) + (attacker.current_magic / 20) as i32;
    let defender_roll = roll_dice(20) + (defender.current_magic / 20) as i32;

    let mut hit = false;
    let mut damage = 0;
    let difference = attacker_roll - defender_roll;

    if difference > 0 {
        hit = true;
        if difference > defender.current_health {
            damage = defender.current_health;
            defender.current_health = 0;
        } else {
            damage = difference;
            defender.current_health -= damage;
        }

        let new_magic = attacker.current_magic - 1;
        if new_magic < 0 {
            attacker.current_magic = 0;
        } else {
            attacker.current_magic = new_magic;
        }

        let new_magic = defender.current_magic - 1;
        if new_magic < 0 {
            defender.current_magic = 0;
        } else {
            defender.current_magic = new_magic;
        }
    }

    attacker.current_magic -= 1;
    if attacker.current_magic < 0 {
        attacker.current_magic = 0;
    }

    defender.current_magic -= 1;
    if defender.current_magic < 0 {
        defender.current_magic = 0;
    }

    (hit, damage)
}