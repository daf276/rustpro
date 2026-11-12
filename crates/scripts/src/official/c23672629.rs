// Valiant Shark Lancer
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 23672629;

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(5);
    const CT: usize = 2;
}

// Cat Shark (WATER Xyz Monster) on the field for the Quick Effect, Sangan to destroy, the opponent's Raigeki to destroy Cat Shark, and
// Raigeki in the Deck to place.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (84224627, MZONE),
    (26202165, MZONE | OPPONENT),
    (12580477, HAND | OPPONENT),
    (12580477, DECK),
    (12580477, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| !quickcon(duel, a));
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_hint_timing(duel, TIMING_END_PHASE, TIMING_END_PHASE);
    e2.set_condition(duel, quickcon);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_DESTROYED);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_condition(duel, tdcon);
    e3.set_target(duel, tdtg);
    e3.set_operation(duel, tdop);
    c.register_effect(duel, e3);
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_type(duel, TYPE_XYZ) && c.is_attribute(duel, ATTRIBUTE_WATER)
}

fn quickcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    duel.is_existing_matching_card(cfilter, a.tp, LOCATION_MZONE, 0, 1, Some(c.into()))
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(aux::TRUE, tp, 0, LOCATION_MZONE, 1, None)
            && duel.check_remove_overlay_card(tp, LOCATION_MZONE as u8, 0, 1, REASON_EFFECT));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, aux::TRUE, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let tc = duel.get_first_target();
    if duel.remove_overlay_card(tp, LOCATION_MZONE as u8, 0, 1, 1, REASON_EFFECT)? > 0
        && let Some(tc) = tc
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.destroy(tc, REASON_EFFECT)?;
    }
    Ok(())
}

fn tdcfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_reason(duel, REASON_BATTLE | REASON_EFFECT)
        && c.is_previous_location(duel, LOCATION_MZONE)
        && c.is_previous_controler(duel, tp)
        && c.is_type(duel, TYPE_XYZ)
        && c.is_attribute(duel, ATTRIBUTE_WATER)
}

fn tdcon(duel: &Duel, a: &Args) -> bool {
    a.eg.unwrap_or_default()
        .is_exists(duel, |c, duel| tdcfilter(c, duel, a.tp), 1, None)
}

fn tdtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    Ok(chk == 0 && duel.is_existing_matching_card(CardId::is_spell, a.tp, LOCATION_DECK, 0, 1, None))
}

fn tdop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 2));
    let g = duel.select_matching_card(tp, CardId::is_spell, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if let Some(tc) = g.get_first() {
        duel.shuffle_deck(tp)?;
        duel.move_sequence(tc, 0)?;
        duel.confirm_decktop(tp, 1)?;
    }
    Ok(())
}
