// Big Inflate Dragon
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE};

const ID: u32 = 91337277;

// The cost needs 50 cards in the Deck, which is 40 and gains a Normal Monster (Water Omotics) per DECK entry; Tributes on the field.
pub const SUPPORT: &[(u32, u16)] = &{
    let mut support = [(2483611, DECK); 26];
    support[0] = (ID, HAND);
    support[1] = (ID, HAND);
    support[2] = (ID, HAND);
    support[3] = (2483611, MZONE);
    support[4] = (2483611, MZONE);
    support[5] = (2483611, MZONE);
    support
};
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_ATKCHANGE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_cost(duel, atkcost);
    e1.set_target(duel, |duel, a, chk, _| {
        Ok(chk == 0 && !a.e.get_handler(duel).is_attack(duel, &[10000]))
    });
    e1.set_operation(duel, atkop);
    c.register_effect(duel, e1);
}

fn atkcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let g = duel.get_decktop_group(a.tp, 50);
    if chk == 0 {
        return Ok(g.filter_count(duel, |c, duel| c.is_able_to_remove_as_cost_full(duel, POS_FACEDOWN), None) == 50);
    }
    duel.disable_shuffle_check(true);
    duel.remove(g, POS_FACEDOWN, REASON_COST)?;
    Ok(false)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) && c.is_faceup(duel) {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_SET_ATTACK_FINAL);
        e1.set_value(duel, 10000);
        e1.set_reset(duel, RESETS_STANDARD_DISABLE_PHASE_END);
        c.register_effect(duel, e1);
    }
    Ok(())
}
