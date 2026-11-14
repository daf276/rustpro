// Springans Kitt
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND};

const ID: u32 = 45484331;

pub const LISTED_SERIES: &[u16] = &[SET_BRANDED];
pub const LISTED_NAMES: &[u32] = &[CARD_ALBAZ];
// In the hand with Albion the Branded Dragon in the Graveyard, and Branded Spells in the Deck to add.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (87746184, GRAVE),
    (29948294, DECK),
    (36637374, DECK),
    (44362883, DECK),
    (30271097, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, spcon);
    e1.set_target(duel, stock::special_summon_self_target_player_0);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SEARCH + CATEGORY_TOHAND + CATEGORY_TODECK);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_SUMMON_SUCCESS);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e3);
}

fn spconfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_type(duel, TYPE_FUSION) && c.lists_code_as_material(duel, &[CARD_ALBAZ])
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(spconfilter, a.tp, LOCATION_MZONE | LOCATION_GRAVE, 0, 1, None)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, e, tp) = (a.e.get_handler(duel), a.e, a.tp);
    if c.is_relate_to_effect(duel, e) {
        duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_BRANDED])
        && c.is_spell_trap(duel)
        && c.is_able_to_hand(duel)
        && (c.is_faceup(duel) || c.is_location(duel, LOCATION_DECK))
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let locations = LOCATION_DECK | LOCATION_GRAVE | LOCATION_REMOVED;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter, tp, locations, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, locations as i32);
    duel.set_operation_info(0, CATEGORY_TODECK, None, 1, tp, LOCATION_HAND as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(
        tp,
        aux::necro_valley_filter(thfilter),
        tp,
        LOCATION_DECK | LOCATION_GRAVE | LOCATION_REMOVED,
        0,
        1,
        1,
        None,
    )?;
    let Some(tc) = g.first() else { return Ok(()) };
    if duel.send_to_hand(tc, PLAYER_NONE, REASON_EFFECT)? <= 0 || !tc.is_location(duel, LOCATION_HAND) {
        return Ok(());
    }
    duel.confirm_cards(1 - tp, tc)?;
    duel.shuffle_hand(tp)?;
    if tc.is_previous_location(duel, LOCATION_DECK) {
        duel.shuffle_deck(tp)?;
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let td = duel.select_matching_card(tp, CardId::is_able_to_deck, tp, LOCATION_HAND, 0, 1, 1, None)?;
    if !td.is_empty() {
        duel.disable_shuffle_check(true);
        duel.break_effect()?;
        duel.send_to_deck(td, PLAYER_NONE, SEQ_DECKBOTTOM, REASON_EFFECT)?;
    }
    Ok(())
}
