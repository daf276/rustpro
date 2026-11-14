// Mementotlan Goblin
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE};

const ID: u32 = 17943271;

pub const LISTED_SERIES: &[u16] = &[SET_MEMENTO];

const MEMENTOTLAN_ANGWITCH: u32 = 54550967;
const MEMENTOTLAN_TATSUNOOTOSHIGO: u32 = 81677154;
const MEMENTOTLAN_BONE_PARTY: u32 = 80722024;

pub const LISTED_NAMES: &[u32] = &[CARD_MEMENTOAL_TECUHTLICA, ID];

// On the field with Mementotlan Angwitch, Tatsunootoshigo and Bone Party in the Deck to send.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (MEMENTOTLAN_ANGWITCH, DECK),
    (MEMENTOTLAN_TATSUNOOTOSHIGO, DECK),
    (MEMENTOTLAN_BONE_PARTY, DECK),
];
// Mementoal Tecuhtlica - Combined Creation, which the first effect needs on the field, is not ported.
pub const CHECKS: &[Check] = &[
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 0))),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER | TIMING_MAIN_END);
    e1.set_condition(duel, untgcon);
    e1.set_cost(duel, Cost::self_discard);
    e1.set_operation(duel, untgop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DESTROY + CATEGORY_TOGRAVE);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, tgtg);
    e2.set_operation(duel, tgop);
    c.register_effect(duel, e2);
}

fn untgcon(duel: &Duel, a: &Args) -> bool {
    duel.is_main_phase()
        && duel.is_existing_matching_card(
            aux::faceup_filter(|c, duel| c.is_code(duel, CARD_MEMENTOAL_TECUHTLICA)),
            a.tp,
            LOCATION_ONFIELD,
            0,
            1,
            None,
        )
}

fn untgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    e1.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
    e1.set_target_card(duel, |duel, _, c| c.is_set_card(duel, &[SET_MEMENTO]));
    e1.set_target_range(duel, LOCATION_MZONE, 0);
    e1.set_value(duel, Value::EffectPlayer(aux::tgoval));
    e1.set_reset(duel, RESET_PHASE + PHASE_END);
    duel.register_effect(e1, tp);
    aux::register_client_hint(duel, c, 0, tp, 1, 0, Some(aux::stringid(ID, 2)));
    Ok(())
}

fn faceup_memento(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_MEMENTO])
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_MEMENTO]) && c.is_able_to_grave(duel) && !c.is_code(duel, ID)
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let g = duel.get_matching_group(faceup_memento, tp, LOCATION_MZONE, 0, None);
    if chk == 0 {
        return Ok(!g.is_empty() && duel.is_existing_matching_card(tgfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), 1, tp, LOCATION_MZONE as i32);
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_matching_card(tp, faceup_memento, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    if !g.is_empty() && duel.destroy(g, REASON_EFFECT)? > 0 {
        let tg = duel.get_matching_group(tgfilter, tp, LOCATION_DECK, 0, None);
        let sg = aux::select_unselect_group(duel, tg, 1, 2, &aux::dncheck, tp, HINTMSG_TOGRAVE)?;
        if !sg.is_empty() {
            duel.send_to_grave(sg, REASON_EFFECT)?;
        }
    }
    Ok(())
}
