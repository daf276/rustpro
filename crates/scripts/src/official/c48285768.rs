// Springans Merrymaker
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, MZONE};

const ID: u32 = 48285768;

pub const LISTED_NAMES: &[u32] = &[CARD_ALBAZ];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(4);
    const CT: usize = 2;
}

// In the Extra Deck, Xyz Summoned with two Springans Kitt, with Springans Kitt in the Deck to send and Albion the Branded Dragon in the Extra Deck to
// send as well.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (45484331, MZONE),
    (45484331, MZONE),
    (45484331, DECK),
    (45484331, DECK),
    (87746184, EXTRA),
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
    e1.set_category(duel, CATEGORY_TOGRAVE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_DAMAGE_STEP);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_previous_location(duel, LOCATION_EXTRA));
    e1.set_target(duel, tgtg);
    e1.set_operation(duel, tgop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_REMOVE + CATEGORY_TOGRAVE);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMING_BATTLE_START | TIMING_BATTLE_END);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| {
        duel.is_turn_player(1 - a.tp) && (duel.is_main_phase() || duel.is_battle_phase())
    });
    e2.set_target(duel, bantg);
    e2.set_operation(duel, banop);
    c.register_effect(duel, e2);
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel) && c.is_set_card(duel, &[SET_SPRINGANS]) && c.is_able_to_grave(duel)
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, tgfilter, LOCATION_DECK, CATEGORY_TOGRAVE)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::send_to_grave(duel, a, tgfilter, LOCATION_DECK)
}

fn bantg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_able_to_remove(duel));
    }
    duel.set_operation_info(0, CATEGORY_REMOVE, Some(c.into()), 1, 0, 0);
    duel.set_possible_operation_info(0, CATEGORY_TOGRAVE, None, 1, a.tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn edfilter(c: CardId, duel: &Duel) -> bool {
    c.is_type(duel, TYPE_FUSION) && c.lists_code_as_material(duel, &[CARD_ALBAZ])
}

fn banop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !(c.is_faceup(duel) && c.is_relate_to_effect(duel, e)) {
        return Ok(());
    }
    let ct = c.get_overlay_count(duel);
    duel.remove(c, POS_FACEUP, REASON_EFFECT | REASON_TEMPORARY)?;
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_PHASE + PHASE_END);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    e1.set_label_object_group(duel, Some(c.into()));
    e1.set_count_limit(duel, 1, 0);
    e1.set_operation(duel, retop);
    duel.register_effect(e1, tp);
    if ct >= 2 && duel.select_yes_no(tp, aux::stringid(ID, 2))? {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
        let g = duel.select_matching_card(tp, edfilter, tp, LOCATION_EXTRA, 0, 1, 1, None)?;
        if !g.is_empty() {
            duel.break_effect()?;
            duel.send_to_grave(g, REASON_EFFECT)?;
        }
    }
    Ok(())
}

fn retop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(c) = a.e.get_label_object_group(duel).and_then(|g| g.get_first()) {
        duel.return_to_field(c)?;
    }
    Ok(())
}
