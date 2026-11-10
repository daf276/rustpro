// Blast Juggler
use crate::prelude::*;
use crate::testing::{MZONE, OPPONENT};

const ID: u32 = 70138455;

// On the field with Sangan (1000 ATK) on both sides to destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (26202165, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_TRIGGER_O + EFFECT_TYPE_FIELD);
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_code(duel, EVENT_PHASE + PHASE_STANDBY);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, |duel, a| duel.is_turn_player(a.tp));
    e1.set_cost(duel, Cost::self_tribute);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_attack_below(duel, 1000)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && filter(chkc, duel));
    }
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, LOCATION_MZONE, LOCATION_MZONE, 2, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, filter, tp, LOCATION_MZONE, LOCATION_MZONE, 2, 2, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 2, 0, 0);
    Ok(false)
}

fn desfilter(c: CardId, duel: &Duel, e: EffectId) -> bool {
    c.is_relate_to_effect(duel, e) && c.is_faceup(duel) && c.is_attack_below(duel, 1000)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let g: Option<CardSet> = duel.get_chain_info(0, CHAININFO_TARGET_CARDS);
    let sg = g.unwrap_or_default().filter(duel, |c, duel| desfilter(c, duel, e), None);
    duel.destroy(sg, REASON_EFFECT)?;
    Ok(())
}
