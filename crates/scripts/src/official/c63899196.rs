// The Monarchs Masterplan
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND};

const ID: u32 = 63899196;
const MONARCHS: [u32; 6] = [51945556, 4929256, 26205777, 60229110, 73125233, 9748752];

pub const LISTED_SERIES: &[u16] = &[SET_MONARCH];
pub const LISTED_NAMES: &[u32] = &[ID, 51945556, 4929256, 26205777, 60229110, 73125233, 9748752];

// Copies in the Deck with Gold Sarcophagus to banish them, Monarch Spells to send, and a Monarch of each Attribute to add.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, DECK),
    (ID, DECK),
    (75500286, HAND),
    (75500286, HAND),
    (9283801, DECK),
    (22842126, DECK),
    (51945556, DECK),
    (4929256, DECK),
    (26205777, DECK),
    (60229110, DECK),
    (73125233, DECK),
    (9748752, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOGRAVE);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_target(duel, tgtg);
    e1.set_operation(duel, tgop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_REMOVE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_MONARCH]) && c.is_spell_trap(duel) && !c.is_code(duel, ID) && c.is_able_to_grave(duel)
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, tgfilter, LOCATION_DECK, CATEGORY_TOGRAVE)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::send_to_grave(duel, a, tgfilter, LOCATION_DECK)
}

fn attrfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    let attr = c.get_attribute(duel);
    c.is_faceup(duel) && duel.is_existing_matching_card(|sc, duel| thfilter(sc, duel, attr), tp, LOCATION_DECK | LOCATION_GRAVE, 0, 1, None)
}

fn thfilter(c: CardId, duel: &Duel, attr: u32) -> bool {
    c.is_attribute(duel, attr) && c.is_attack(duel, &[2400]) && c.is_defense(duel, &[1000]) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, 1 - tp) && attrfilter(chkc, duel, tp));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(|c, duel| attrfilter(c, duel, tp), tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    duel.select_target(tp, |c, duel| attrfilter(c, duel, tp), tp, 0, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, (LOCATION_DECK | LOCATION_GRAVE) as i32);
    duel.set_possible_operation_info(0, CATEGORY_SUMMON, None, 1, tp, LOCATION_HAND as i32);
    Ok(false)
}

fn sumfilter(c: CardId, duel: &Duel) -> bool {
    c.is_summonable(duel, true, None) && MONARCHS.iter().any(|&code| c.is_code(duel, code))
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !(tc.is_relate_to_effect(duel, a.e) && tc.is_faceup(duel)) {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let attr = tc.get_attribute(duel);
    let filter = aux::necro_valley_filter(|c, duel| thfilter(c, duel, attr));
    let Some(sc) = duel
        .select_matching_card(tp, filter, tp, LOCATION_DECK | LOCATION_GRAVE, 0, 1, 1, None)?
        .get_first()
    else {
        return Ok(());
    };
    if duel.send_to_hand(sc, PLAYER_NONE, REASON_EFFECT)? > 0 && sc.is_location(duel, LOCATION_HAND) {
        duel.confirm_cards(1 - tp, sc)?;
        duel.shuffle_hand(tp)?;
        duel.shuffle_deck(tp)?;
        let sg = duel.get_matching_group(sumfilter, tp, LOCATION_HAND | LOCATION_MZONE, 0, None);
        if !sg.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 2))? {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_SUMMON);
            if let Some(sumc) = sg.select(duel, tp, 1, 1, None)?.get_first() {
                duel.break_effect()?;
                duel.summon(tp, sumc, true, None)?;
            }
        }
    }
    Ok(())
}
