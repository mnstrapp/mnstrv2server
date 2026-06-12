use crate::models::mnstr::Mnstr;
use crate::battle::helpers::roll_dice;

pub fn attack(attacker: &mut Mnstr, defender: &mut Mnstr) -> (bool, i32) {
    let attacker_roll = roll_dice(20)
        + (attacker.current_speed / 20) as i32
        + (attacker.current_attack / 20) as i32;
    let defender_roll = roll_dice(20)
        + (defender.current_intelligence / 20) as i32
        + (defender.current_defense / 20) as i32;

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
    }

    attacker.current_attack -= 1;
    if attacker.current_attack < 0 {
        attacker.current_attack = 0;
    }

    attacker.current_speed -= 1;
    if attacker.current_speed < 0 {
        attacker.current_speed = 0;
    }
    
    defender.current_defense -= 1;
    if defender.current_defense < 0 {
        defender.current_defense = 0;
    }
    
    defender.current_intelligence -= 1;
    if defender.current_intelligence < 0 {
        defender.current_intelligence = 0;
    }

    (hit, damage)
}