// Neo-Spacian Grand Mole
use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT};

const ID: u32 = 80344569;

// Copies on both fields so that they battle each other, and the opponent's Sangan.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE | OPPONENT),
    (ID, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_BATTLE_START);
    e1.set_condition(duel, |duel, a| matches!(duel.get_battle_monster(a.tp), (Some(_), Some(_))));
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let (Some(monster1), Some(monster2)) = duel.get_battle_monster(tp) else {
        return Ok(false);
    };
    if chk == 0 {
        return Ok(monster1.is_able_to_hand(duel) && monster2.is_able_to_hand(duel));
    }
    let g = CardSet::from_iter([monster1, monster2]);
    duel.set_operation_info(0, CATEGORY_TOHAND, Some(g), g.len() as u8, tp, 0);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    if let (Some(monster1), Some(monster2)) = duel.get_battle_monster(a.tp)
        && monster1.is_relate_to_battle(duel)
        && monster2.is_relate_to_battle(duel)
    {
        duel.send_to_hand(CardSet::from_iter([monster1, monster2]), PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}
