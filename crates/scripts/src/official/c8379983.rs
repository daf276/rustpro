// Lunalight Gold Leo
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 8379983;

pub const LISTED_SERIES: &[u16] = &[SET_LUNALIGHT];

const LUNALIGHT_BLACK_SHEEP: u32 = 11317977;
const LUNALIGHT_SILVER_HOUND: u32 = 35763582;
const DARK_HOLE: u32 = 53129443;

pub const LISTED_NAMES: &[u32] = &[ID];

// In hand to Normal Summon with Lunalight Black Sheep and Silver Hound in the Deck to search, and a Lunalight on the
// field for the opponent's Dark Hole to send to the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (LUNALIGHT_BLACK_SHEEP, DECK),
    (LUNALIGHT_SILVER_HOUND, DECK),
    (LUNALIGHT_SILVER_HOUND, MZONE),
    (DARK_HOLE, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_HANDES);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, deckthtg);
    e1.set_operation(duel, deckthop);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_TOHAND);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY + EFFECT_FLAG_DAMAGE_STEP);
    e3.set_code(duel, EVENT_CUSTOM + ID);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_target(duel, gythtg);
    e3.set_operation(duel, stock::first_target_to_hand);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e4.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e4.set_code(duel, EVENT_TO_GRAVE);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_operation(duel, regop);
    c.register_effect(duel, e4);
}

// The group e3 and e4 share through e3's label object.
fn group_key(duel: &Duel, e: EffectId) -> u64 {
    u64::from(ID) << 16 | e.get_owner(duel).index() as u64
}

fn label_group(duel: &Duel, e: EffectId) -> CardSet {
    duel.get_script_group(group_key(duel, e)).unwrap_or_default()
}

fn deckthfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_LUNALIGHT]) && c.is_monster(duel) && !c.is_code(duel, ID) && c.is_able_to_hand(duel)
}

fn deckthtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(deckthfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    duel.set_operation_info(0, CATEGORY_HANDES, None, 0, tp, 1);
    Ok(false)
}

fn deckthop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, deckthfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty()
        && duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)? > 0
        && g.get_first().is_some_and(|tc| tc.is_location(duel, LOCATION_HAND))
    {
        duel.confirm_cards(1 - tp, g)?;
        duel.shuffle_hand(tp)?;
        duel.break_effect()?;
        duel.discard_hand(tp, aux::TRUE, 1, 1, REASON_EFFECT + REASON_DISCARD)?;
    }
    Ok(())
}

fn gythfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_LUNALIGHT])
        && c.is_monster(duel)
        && c.is_able_to_hand(duel)
        && c.is_can_be_effect_target(duel, e)
        && c.is_location(duel, LOCATION_GRAVE)
        && c.is_controler(duel, tp)
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let tg = a.eg.unwrap_or_default().filter(duel, |c, duel| gythfilter(c, duel, e, tp), None);
    if !tg.is_empty() {
        for tc in tg {
            tc.register_flag_effect(duel, ID, RESET_CHAIN, 0, 1);
        }
        let mut g = if duel.get_current_chain() == 0 {
            CardSet::default()
        } else {
            label_group(duel, e)
        };
        g.merge(tg);
        let g = g.filter(duel, |c, duel| c.get_flag_effect(duel, ID) != 0, None);
        duel.set_script_group(group_key(duel, e), Some(g));
        duel.raise_single_event(e.get_handler(duel), EVENT_CUSTOM + ID, Some(e), 0, tp, tp, 0)?;
    }
    Ok(())
}

fn gythtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let g = label_group(duel, e).filter(duel, |c, duel| gythfilter(c, duel, e, tp), None);
    if let Some(chkc) = chkc {
        return Ok(g.is_contains(chkc) && gythfilter(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(!g.is_empty());
    }
    let mut tg = g;
    if g.len() > 1 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
        tg = g.select(duel, tp, 1, 1, None)?;
    }
    duel.set_target_card(tg);
    duel.set_operation_info(0, CATEGORY_TOHAND, Some(tg), 1, tp, 0);
    Ok(false)
}
