// Skull Archfiend of Chaos
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND};

const ID: u32 = 24088928;

pub const LISTED_NAMES: &[u32] = &[CARD_LIGHT_AND_DARKNESS_RITUAL];

// In the hand with a copy in the GY, which mentions the Ritual Spell, and Black Magic Ritual and Magician of Black Chaos
// in the Deck for the search.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, GRAVE), (76792184, DECK), (30208479, DECK)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TODECK + CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_HAND | LOCATION_GRAVE);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_target(duel, tdsptg);
    e1.set_operation(duel, tdspop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, thcost);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
}

fn rescon(duel: &Duel, sg: CardSet, _: CardSet, _: Option<CardId>) -> (bool, bool) {
    (
        sg.is_exists(duel, |c, duel| c.lists_code(duel, CARD_LIGHT_AND_DARKNESS_RITUAL), 1, None),
        false,
    )
}

fn tdsptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    if chkc.is_some() {
        return Ok(false);
    }
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let locations = LOCATION_GRAVE | LOCATION_REMOVED;
    let g = duel.get_target_group(
        aux::faceup_filter(|c, duel| c.is_able_to_deck(duel)),
        tp,
        locations,
        locations,
        Some(c.into()),
    );
    if chk == 0 {
        return Ok(g.len() >= 3
            && aux::select_unselect_group_check(duel, g, 3, 3, &rescon)
            && duel.get_location_count(tp, LOCATION_MZONE) > 0
            && c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    let tg = aux::select_unselect_group(duel, g, 3, 3, &rescon, tp, HINTMSG_TODECK)?;
    duel.set_target_card(tg);
    duel.set_operation_info(0, CATEGORY_TODECK, Some(tg), 3, tp, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    Ok(false)
}

fn tdspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let tg = duel.get_target_cards(e).unwrap_or_default();
    if !tg.is_empty()
        && duel.send_to_deck(tg, PLAYER_NONE, SEQ_DECKBOTTOM, REASON_EFFECT)? > 0
        && tg.is_exists(duel, |c, duel| c.is_location(duel, LOCATION_DECK | LOCATION_EXTRA), 1, None)
    {
        let (tg_your, tg_opp) = tg.split(duel, |c, duel| c.is_controler(duel, tp), None);
        let in_deck = |c: CardId, duel: &Duel| c.is_location(duel, LOCATION_DECK);
        let your_count = tg_your.filter_count(duel, in_deck, None);
        let opp_count = tg_opp.filter_count(duel, in_deck, None);
        if your_count >= 2 {
            duel.sort_deckbottom(tp, tp, your_count as u16)?;
        }
        if opp_count >= 2 {
            duel.sort_deckbottom(tp, 1 - tp, opp_count as u16)?;
        }
        let c = e.get_handler(duel);
        if c.is_relate_to_effect(duel, e) {
            duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)?;
        }
    }
    Ok(())
}

fn thcostfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_ritual_spell(duel)
        && c.is_able_to_grave_as_cost(duel)
        && duel.is_existing_matching_card(|mc, duel| thfilter(mc, duel, c), tp, LOCATION_DECK, 0, 1, None)
}

fn thfilter(c: CardId, duel: &Duel, rit_spell: CardId) -> bool {
    c.is_ritual_monster(duel) && rit_spell.lists_code(duel, c.get_code(duel)) && c.is_able_to_hand(duel)
}

fn thcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    let locations = LOCATION_HAND | LOCATION_DECK;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| thcostfilter(c, duel, tp), tp, locations, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, |c, duel| thcostfilter(c, duel, tp), tp, locations, 0, 1, 1, None)?;
    if let Some(sc) = g.get_first() {
        duel.send_to_grave(sc, REASON_COST)?;
        a.e.set_label_object_group(duel, Some(sc.into()));
    }
    Ok(false)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, a.tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let Some(rit_spell) = a.e.get_label_object_group(duel).and_then(|g| g.get_first()) else {
        return Ok(());
    };
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, |c, duel| thfilter(c, duel, rit_spell), tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
    }
    Ok(())
}
