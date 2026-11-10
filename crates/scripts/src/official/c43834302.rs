// Synchro Fellowship
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE};

const ID: u32 = 43834302;
const ASSAULT_SONIC_WARRIOR: u32 = 18711696;
const NAMES: [u32; 2] = [CARD_JUNK_WARRIOR, CARD_STARDUST_DRAGON];

pub const LISTED_NAMES: &[u32] = &[CARD_JUNK_SYNCHRON, CARD_JUNK_WARRIOR, CARD_STARDUST_DRAGON];

// In the hand and the Graveyard, with Junk Synchron and Assault Sonic Warrior (which lists Junk Warrior) in the Deck to
// add, and Junk Warrior and a Synchron monster on the field and in the hand for the Level change and extra Normal Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (CARD_JUNK_SYNCHRON, DECK),
    (CARD_JUNK_SYNCHRON, HAND),
    (ASSAULT_SONIC_WARRIOR, DECK),
    (CARD_JUNK_WARRIOR, MZONE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_HANDES);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_LVCHANGE);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_condition(duel, |duel, a| {
        duel.is_player_can_additional_summon(a.tp) && duel.get_flag_effect(a.tp, ID) == 0
    });
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, lvtg);
    e2.set_operation(duel, lvop);
    c.register_effect(duel, e2);
}

fn lists_names(c: CardId, duel: &Duel) -> bool {
    NAMES.iter().any(|&code| c.lists_code(duel, code))
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_code(duel, CARD_JUNK_SYNCHRON) || (c.is_monster(duel) && lists_names(c, duel))) && c.is_able_to_hand(duel)
}

fn rescon(duel: &Duel, sg: CardSet, _: CardSet, _: Option<CardId>) -> (bool, bool) {
    (
        sg.is_exists(duel, |c, duel| c.is_code(duel, CARD_JUNK_SYNCHRON), 1, None) && sg.is_exists(duel, lists_names, 1, None),
        false,
    )
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let g = duel.get_matching_group(thfilter, tp, LOCATION_DECK, 0, None);
    if chk == 0 {
        return Ok(aux::select_unselect_group_check(duel, g, 2, 2, &rescon));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 2, tp, LOCATION_DECK as i32);
    duel.set_operation_info(0, CATEGORY_HANDES, None, 0, tp, 1);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_matching_group(thfilter, tp, LOCATION_DECK, 0, None);
    if g.is_empty() {
        return Ok(());
    }
    let sg = aux::select_unselect_group(duel, g, 2, 2, &rescon, tp, HINTMSG_ATOHAND)?;
    if !sg.is_empty() && duel.send_to_hand(sg, PLAYER_NONE, REASON_EFFECT)? > 0 {
        duel.confirm_cards(1 - tp, sg)?;
        duel.shuffle_hand(tp)?;
        duel.break_effect()?;
        duel.discard_hand(tp, aux::TRUE, 1, 1, REASON_EFFECT | REASON_DISCARD)?;
    }
    Ok(())
}

fn lvfilter(c: CardId, duel: &Duel) -> bool {
    c.is_synchro_monster(duel) && c.is_level_above(duel, 2) && c.is_faceup(duel)
}

fn lvtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && lvfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_player_can_summon(tp) && duel.is_existing_target(lvfilter, tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    let g = duel.select_target(tp, lvfilter, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_LVCHANGE, g, 1, tp, 1);
    Ok(false)
}

fn lvop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, e) && tc.is_faceup(duel) && tc.is_level_above(duel, 2) && !tc.is_immune_to_effect(duel, e) {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_code(duel, EFFECT_UPDATE_LEVEL);
        e1.set_value(duel, -1);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        tc.register_effect(duel, e1);
        duel.break_effect()?;
        aux::register_client_hint(duel, c, 0, tp, 1, 0, Some(aux::stringid(ID, 2)));
        duel.register_flag_effect(tp, ID, RESET_PHASE | PHASE_END, 0, 1);
        let e2 = duel.create_effect(c);
        e2.set_type(duel, EFFECT_TYPE_FIELD);
        e2.set_code(duel, EFFECT_EXTRA_SUMMON_COUNT);
        e2.set_target_range(duel, LOCATION_HAND | LOCATION_MZONE, 0);
        e2.set_target_card(duel, |duel, _, c| c.is_set_card(duel, &[SET_SYNCHRON]));
        e2.set_reset(duel, RESET_PHASE | PHASE_END);
        duel.register_effect(e2, tp);
    }
    Ok(())
}
