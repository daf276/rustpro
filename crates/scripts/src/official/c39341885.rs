// Radiant Typhoon Varuroon, the Marine Eidolon
use super::c16922142::quickplaycon;
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE};

const ID: u32 = 39341885;

pub const LISTED_NAMES: &[u32] = &[CARD_MYSTICAL_SPACE_TYPHOON];
// On the field with a copy in the Extra Deck, two Radiant Typhoon monsters to Link Summon it with, a Quick-Play Spell
// (Mystical Space Typhoon) in the hand, Mystical Space Typhoon in the Deck to search and Radiant Typhoon Mandate to place.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (80538047, MZONE),
    (54143349, MZONE),
    (CARD_MYSTICAL_SPACE_TYPHOON, HAND),
    (CARD_MYSTICAL_SPACE_TYPHOON, DECK),
    (53813120, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_set_card_full(duel, &[SET_RADIANT_TYPHOON], Some(lc), sumtype, tp)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SEARCH + CATEGORY_TOHAND);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_link_summoned(duel));
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, monpltg);
    e2.set_operation(duel, monplop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_CHAINING);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, (ID, 2));
    e3.set_condition(duel, quickplaycon);
    e3.set_target(duel, trappltg);
    e3.set_operation(duel, trapplop);
    c.register_effect(duel, e3);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, CARD_MYSTICAL_SPACE_TYPHOON) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK | LOCATION_GRAVE, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let filter = aux::necro_valley_filter(thfilter);
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_DECK | LOCATION_GRAVE, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
    }
    Ok(())
}

fn monplfilter(c: CardId, duel: &Duel, e: EffectId) -> bool {
    c.is_faceup(duel) && !c.is_forbidden(duel) && c.is_can_be_effect_target(duel, e)
}

fn rescon(duel: &Duel, sg: CardSet, tp: u8) -> (bool, bool) {
    let (your_g, opp_g) = sg.split(duel, |c, duel| c.is_owner(duel, tp), None);
    (
        sg.len() == 2
            && sg.is_exists(
                duel,
                |c, duel| c.is_set_card(duel, &[SET_RADIANT_TYPHOON]) && c.is_controler(duel, tp),
                1,
                None,
            )
            && duel.get_location_count(tp, LOCATION_SZONE) >= your_g.len() as i32
            && duel.get_location_count(1 - tp, LOCATION_SZONE) >= opp_g.len() as i32,
        false,
    )
}

fn monpltg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chkc.is_some() {
        return Ok(false);
    }
    let g = duel.get_matching_group(|c, duel| monplfilter(c, duel, e), tp, LOCATION_MZONE, LOCATION_MZONE, None);
    let rescon = |duel: &Duel, sg, _, _| rescon(duel, sg, tp);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_SZONE) > 0 && aux::select_unselect_group_check(duel, g, 2, 2, &rescon));
    }
    let tg = aux::select_unselect_group(duel, g, 2, 2, &rescon, tp, HINTMSG_TOFIELD)?;
    duel.set_target_card(tg);
    Ok(false)
}

fn monplop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    for tc in duel.get_target_cards(e).unwrap_or_default() {
        if tc.is_immune_to_effect(duel, e) {
            continue;
        }
        let owner = tc.get_owner(duel);
        if duel.get_location_count(owner, LOCATION_SZONE) <= 0 {
            duel.send_to_grave_full(tc, REASON_RULE, PLAYER_NONE, PLAYER_NONE)?;
        } else if duel.move_to_field(tc, tp, owner, LOCATION_SZONE, POS_FACEUP, tc.is_monster_card(duel))? {
            let e1 = duel.create_effect(c);
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
            e1.set_code(duel, EFFECT_CHANGE_TYPE);
            e1.set_value(duel, (TYPE_SPELL | TYPE_CONTINUOUS) as i32);
            e1.set_reset(duel, RESET_EVENT | (RESETS_STANDARD & !RESET_TURN_SET));
            tc.register_effect(duel, e1);
        }
    }
    Ok(())
}

fn trapplfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_RADIANT_TYPHOON]) && c.is_continuous_trap(duel) && !c.is_forbidden(duel) && c.check_unique_on_field(duel, tp)
}

fn trappltg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    Ok(chk == 0
        && duel.get_location_count(tp, LOCATION_SZONE) > 0
        && duel.is_existing_matching_card(|c, duel| trapplfilter(c, duel, tp), tp, LOCATION_DECK | LOCATION_GRAVE, 0, 1, None))
}

fn trapplop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if duel.get_location_count(tp, LOCATION_SZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOFIELD);
    let filter = aux::necro_valley_filter(move |c, duel| trapplfilter(c, duel, tp));
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_DECK | LOCATION_GRAVE, 0, 1, 1, None)?;
    if let Some(sc) = g.get_first() {
        duel.move_to_field(sc, tp, tp, LOCATION_SZONE, POS_FACEUP, true)?;
    }
    Ok(())
}
