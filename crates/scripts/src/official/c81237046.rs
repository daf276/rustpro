// Enneacraft - Archa.TAIL
use super::c28454232::{flip_quick, register_counter, register_destroy, register_flip_flag, register_set_summon};
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT, PZONE};

const ID: u32 = 81237046;

pub const COUNTER_PLACE_LIST: &[u16] = &[COUNTER_EC];
pub const LISTED_SERIES: &[u16] = &[SET_ENNEACRAFT];

// In the hand, a Pendulum Zone and face-down on the field, with the opponent's Raigeki and Dark Hole to answer and Sangan to
// destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, PZONE),
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (12580477, HAND | OPPONENT),
    (53129443, HAND | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    register_counter(duel, c);
    register_destroy::<ID>(duel, c);
    register_set_summon::<ID, 1>(duel, c);
    let e4 = flip_quick::<ID>(duel, c, 0, 0);
    e4.set_condition(duel, indescon);
    e4.set_operation(duel, indesop);
    c.register_effect(duel, e4);
    register_flip_flag::<ID, 3>(duel, c);
    let e5b = duel.create_effect(c);
    e5b.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e5b.set_code(duel, EVENT_TO_GRAVE);
    e5b.set_range(duel, LOCATION_MZONE);
    e5b.set_condition(duel, |duel, a| {
        a.e.get_handler(duel).has_flag_effect(duel, ID)
            && a.eg
                .is_some_and(|eg| eg.is_exists(duel, |c, duel| damconfilter(c, duel, a.tp), 1, None))
    });
    e5b.set_operation(duel, damop);
    c.register_effect(duel, e5b);
}

fn indescon(duel: &Duel, a: &Args) -> bool {
    let (tp, ev) = (a.tp, a.ev as u8);
    if a.rp == tp || duel.has_flag_effect(tp, ID) || a.e.get_handler(duel).is_faceup(duel) {
        return false;
    }
    if a.re.is_some_and(|re| re.is_has_category(duel, CATEGORY_NEGATE))
        && duel
            .get_chain_info::<Option<EffectId>>(ev - 1, CHAININFO_TRIGGERING_EFFECT)
            .is_some_and(|te| te.is_has_type(duel, EFFECT_TYPE_ACTIVATE))
    {
        return false;
    }
    duel.get_operation_info(ev, CATEGORY_DESTROY)
        .and_then(|info| info.op_cards)
        .is_some_and(|tg| tg.filter_count(duel, CardId::is_on_field, None) > 0)
}

fn indesop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if duel.has_flag_effect(tp, ID) {
        return Ok(());
    }
    duel.register_flag_effect(tp, ID, RESET_PHASE | PHASE_END, 0, 1);
    let c = a.e.get_handler(duel);
    aux::register_client_hint(duel, c, 0, tp, 1, 0, Some(aux::stringid(ID, 5)));
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_SET_AVAILABLE);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e1.set_target_range(duel, LOCATION_ONFIELD, 0);
    e1.set_target_card(duel, |duel, _, c| {
        c.is_location(duel, LOCATION_MZONE) || (c.is_faceup(duel) && c.is_spell(duel) && c.is_set_card(duel, &[SET_ENNEACRAFT]))
    });
    e1.set_value(duel, 1);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    Ok(())
}

fn damconfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_monster(duel) && c.is_controler(duel, 1 - tp)
}

fn damop(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.hint(HINT_CARD, 1 - a.tp, ID.into());
    duel.damage(1 - a.tp, 900, REASON_EFFECT)?;
    Ok(())
}
