// D.D. Warrior
use crate::prelude::*;
use crate::testing::{MZONE, OPPONENT};

const ID: u32 = 37043180;

// Two on the field to be attacked, and the opponent's strong attackers.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, MZONE), (13597785, MZONE | OPPONENT), (13597785, MZONE | OPPONENT)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_BATTLED);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn battling_cards(duel: &Duel) -> CardSet {
    let g: CardSet = [duel.get_attacker(), duel.get_attack_target()].into_iter().flatten().collect();
    g.filter(duel, CardId::is_relate_to_battle, None)
}

fn target(duel: &mut Duel, _: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.get_attack_target().is_some());
    }
    let rg = battling_cards(duel);
    duel.set_operation_info(0, CATEGORY_REMOVE, Some(rg), rg.len() as u8, 0, 0);
    Ok(false)
}

fn operation(duel: &mut Duel, _: &Args) -> Script<()> {
    let rg = battling_cards(duel);
    duel.remove(rg, POS_FACEUP, REASON_EFFECT)?;
    Ok(())
}
