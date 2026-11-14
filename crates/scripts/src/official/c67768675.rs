// Kaibaman the Legend
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND};

const ID: u32 = 67768675;

pub const LISTED_SERIES: &[u16] = &[SET_BLUE_EYES];
pub const LISTED_NAMES: &[u32] = &[CARD_BLUEEYES_W_DRAGON];

// In the hand with Blue-Eyes White Dragon in the Deck to show and Special Summon, and copies in the Graveyard to search.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, GRAVE),
    (ID, GRAVE),
    (CARD_BLUEEYES_W_DRAGON, DECK),
    (CARD_BLUEEYES_W_DRAGON, DECK),
    (CARD_BLUEEYES_W_DRAGON, DECK),
    (CARD_BLUEEYES_W_DRAGON, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

const SHOW_LOCATION: u32 = LOCATION_HAND | LOCATION_DECK | LOCATION_ONFIELD | LOCATION_GRAVE;
const SPSUMMON_LOCATION: u32 = LOCATION_HAND | LOCATION_DECK | LOCATION_GRAVE;

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_property_full(duel, EFFECT_FLAG_DELAY, EFFECT_FLAG2_CHECK_SIMULTANEOUS);
    e3.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e3.set_range(duel, LOCATION_GRAVE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_condition(duel, thcon);
    e3.set_cost(duel, Cost::self_banish);
    e3.set_target(duel, thtg);
    e3.set_operation(duel, thop);
    c.register_effect(duel, e3);
}

fn showfilter(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, CARD_BLUEEYES_W_DRAGON) && (c.is_faceup(duel) || !c.is_on_field(duel))
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_code(duel, CARD_BLUEEYES_W_DRAGON) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(showfilter, tp, SHOW_LOCATION, 0, 3, None)
            && duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, SPSUMMON_LOCATION, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, SPSUMMON_LOCATION as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let mut rg = duel.get_matching_group(showfilter, tp, SHOW_LOCATION, 0, None);
    if rg.len() > 3 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONFIRM);
        rg = duel.select_matching_card(tp, showfilter, tp, SHOW_LOCATION, 0, 3, 3, None)?;
    }
    if rg.len() != 3 {
        return Ok(());
    }
    let (fieldgraveg, handdeckg) = rg.split(duel, |c, duel| c.is_location(duel, LOCATION_ONFIELD | LOCATION_GRAVE), None);
    if !fieldgraveg.is_empty() {
        duel.hint_selection(fieldgraveg);
    }
    if !handdeckg.is_empty() {
        duel.confirm_cards(1 - tp, handdeckg)?;
    }
    if rg.is_exists(duel, |c, duel| c.is_location(duel, LOCATION_HAND), 1, None) {
        duel.shuffle_hand(tp)?;
    }
    if rg.is_exists(duel, |c, duel| c.is_location(duel, LOCATION_DECK), 1, None) {
        duel.shuffle_deck(tp)?;
    }
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let filter = aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp));
    let g = duel.select_matching_card(tp, filter, tp, SPSUMMON_LOCATION, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.break_effect()?;
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}

fn thconfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_code(duel, CARD_BLUEEYES_W_DRAGON) && c.is_faceup(duel) && c.is_summon_player(duel, tp)
}

fn thcon(duel: &Duel, a: &Args) -> bool {
    a.eg.unwrap_or_default()
        .is_exists(duel, |c, duel| thconfilter(c, duel, a.tp), 1, None)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_BLUE_EYES]) && c.is_monster(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}
