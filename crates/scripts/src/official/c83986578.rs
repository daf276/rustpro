// King Tiger Wanghu
use crate::prelude::*;
use crate::testing::{HAND, MZONE, OPPONENT};

const ID: u32 = 83986578;

// On the field, with Sangan and Kuriboh to Summon in either hand.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (26202165, HAND),
    (26202165, HAND | OPPONENT),
    (59053232, HAND),
    (59053232, HAND | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, condition);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e2);
}

fn filter(c: CardId, duel: &Duel, e: Option<EffectId>) -> bool {
    c.is_faceup(duel) && c.is_attack_below(duel, 1400) && e.is_none_or(|e| c.is_relate_to_effect(duel, e))
}

fn condition(duel: &Duel, a: &Args) -> bool {
    let (eg, c) = (a.eg.unwrap_or_default(), a.e.get_handler(duel));
    eg.is_exists(duel, |tc, duel| filter(tc, duel, None), 1, None) && !eg.is_contains(c) && !c.is_status(duel, STATUS_BATTLE_DESTROYED)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let eg = a.eg.unwrap_or_default();
    let g = eg.filter(duel, |tc, duel| filter(tc, duel, None), None);
    duel.set_target_card(eg);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), g.len() as u8, 0, 0);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c) = (a.e, a.e.get_handler(duel));
    if !c.is_relate_to_effect(duel, e) || c.is_facedown(duel) {
        return Ok(());
    }
    let g = a.eg.unwrap_or_default().filter(duel, |tc, duel| filter(tc, duel, Some(e)), None);
    if !g.is_empty() {
        duel.destroy(g, REASON_EFFECT)?;
    }
    Ok(())
}
