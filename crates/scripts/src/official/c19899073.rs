// Ame no Murakumo no Mitsurugi
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 19899073;
const MITSURUGI_RITUAL: u32 = 81560239;

pub const LISTED_NAMES: &[u32] = &[ID, MITSURUGI_RITUAL];

// In the hand to Ritual Summon with Mitsurugi Ritual from Aramasa on the field, and on the field with Aramasa in the
// Deck to search. The opponent's Raigeki destroys it to Tribute it and Mystical Space Typhoon gives it a chain to negate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (MITSURUGI_RITUAL, HAND),
    (40543231, MZONE),
    (40543231, MZONE),
    (40543231, DECK),
    (CARD_POLYMERIZATION, GRAVE),
    (12580477, HAND | OPPONENT),
    (5318639, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DISABLE + CATEGORY_HANDES);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, discon);
    e2.set_target(duel, distg);
    e2.set_operation(duel, disop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_RELEASE);
    e3.set_count_limit(duel, 1, (ID, 2));
    e3.set_target(duel, thtg::<ID>);
    e3.set_operation(duel, thop::<ID, 5>);
    c.register_effect(duel, e3);
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let g = duel.get_matching_group(aux::TRUE, tp, 0, LOCATION_MZONE, None);
    if chk == 0 {
        return Ok(!g.is_empty());
    }
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), g.len() as u8, tp, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_matching_group(aux::TRUE, a.tp, 0, LOCATION_MZONE, None);
    if !g.is_empty() {
        duel.destroy(g, REASON_EFFECT)?;
    }
    Ok(())
}

fn discon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    a.rp == 1 - tp
        && (duel.is_existing_matching_card(
            |c, duel| c.is_discardable_full(duel, REASON_EFFECT, Some(1 - tp)),
            tp,
            0,
            LOCATION_HAND,
            1,
            None,
        ) || duel.is_chain_disablable(a.ev as u8))
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_possible_operation_info(0, CATEGORY_DISABLE, a.eg, 1, 0, 0);
    duel.set_possible_operation_info(0, CATEGORY_HANDES, None, 1, 1 - a.tp, 1);
    Ok(false)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let op = if duel.is_existing_matching_card(
        |c, duel| c.is_discardable_full(duel, REASON_EFFECT, None),
        tp,
        0,
        LOCATION_HAND,
        1,
        None,
    ) {
        duel.select_effect(1 - tp, &[(true, aux::stringid(ID, 3)), (true, aux::stringid(ID, 4))])?
    } else {
        Some(2)
    };
    match op {
        Some(1) => {
            duel.discard_hand(1 - tp, aux::TRUE, 1, 1, REASON_EFFECT | REASON_DISCARD)?;
        }
        Some(2) => {
            duel.negate_effect(a.ev as u8);
        }
        _ => {}
    }
    Ok(())
}

pub fn thfilter<const CODE: u32>(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_MITSURUGI]) && c.is_able_to_hand(duel) && !c.is_code(duel, CODE)
}

// The search of this card and of Ame no Habakiri no Mitsurugi, which differ in their code and the hint of the Special
// Summon.
pub fn thtg<const CODE: u32>(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter::<CODE>, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(a.e.get_handler(duel).into()), 1, tp, 0);
    Ok(false)
}

pub fn thop<const CODE: u32, const HINT: u64>(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, thfilter::<CODE>, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if g.is_empty() || duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)? == 0 {
        return Ok(());
    }
    duel.confirm_cards(1 - tp, g)?;
    let c = e.get_handler(duel);
    if duel.get_location_count(tp, LOCATION_MZONE) > 0
        && c.is_relate_to_effect(duel, e)
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
        && aux::nvfilter(c, duel)
        && duel.select_yes_no(tp, aux::stringid(CODE, HINT))?
    {
        duel.break_effect()?;
        duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
