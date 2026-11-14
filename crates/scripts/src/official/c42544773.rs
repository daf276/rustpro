// Spenta, the Magistus Sealer
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 42544773;

pub const LISTED_SERIES: &[u16] = &[SET_MAGISTUS];
pub const LISTED_NAMES: &[u32] = &[ID];

// In hand and in the Graveyard, a Magistus monster, Crowley, in the Deck to search or in the Graveyard to equip, Magistus
// Chorozo in the Extra Deck to equip, and the opponent's monster to halve.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (875572, DECK),
    (875572, GRAVE),
    (66532962, EXTRA),
    (13597785, MZONE),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, Cost::self_discard);
    e1.set_target(duel, efftg);
    e1.set_operation(duel, effop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_EQUIP);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, eqtg);
    e2.set_operation(duel, eqop);
    c.register_effect(duel, e2);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_MAGISTUS]) && c.is_monster(duel) && !c.is_code(duel, ID) && c.is_able_to_hand(duel)
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let b1 = duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None);
    let b2 = duel.is_existing_matching_card(CardId::is_faceup, tp, 0, LOCATION_MZONE, 1, None);
    if chk == 0 {
        return Ok(b1 || b2);
    }
    let op = duel.select_effect(tp, &[(b1, aux::stringid(ID, 2)), (b2, aux::stringid(ID, 3))])?;
    e.set_label(duel, op.map_or(0, |op| op as i64));
    if op == Some(1) {
        e.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
        duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    } else if op == Some(2) {
        e.set_category(duel, CATEGORY_ATKCHANGE);
    }
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    match e.get_label(duel) {
        1 => {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
            let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
            if !g.is_empty() {
                duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
                duel.confirm_cards(1 - tp, g)?;
            }
        }
        2 => {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATKDEF);
            let g = duel.select_matching_card(tp, CardId::is_faceup, tp, 0, LOCATION_MZONE, 1, 1, None)?;
            if let Some(sc) = g.get_first() {
                duel.hint_selection(sc);
                let e1 = duel.create_effect(e.get_handler(duel));
                e1.set_type(duel, EFFECT_TYPE_SINGLE);
                e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
                e1.set_code(duel, EFFECT_SET_ATTACK_FINAL);
                // Lua halves with a float, which the core rounds.
                e1.set_value(duel, (f64::from(sc.get_attack(duel)) / 2.0).round() as i32);
                e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
                sc.register_effect(duel, e1);
            }
        }
        _ => {}
    }
    Ok(())
}

fn eqfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_MAGISTUS]) && c.is_monster(duel) && c.check_unique_on_field(duel, tp) && !c.is_forbidden(duel)
}

fn eqtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_MZONE) && chkc.is_faceup(duel));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_SZONE) > 0
            && duel.is_existing_target(CardId::is_faceup, tp, LOCATION_MZONE, 0, 1, None)
            && duel.is_existing_matching_card(
                |c, duel| eqfilter(c, duel, tp),
                tp,
                LOCATION_EXTRA | LOCATION_GRAVE,
                0,
                1,
                Some(e.get_handler(duel).into()),
            ));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    duel.select_target(tp, CardId::is_faceup, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_EQUIP, None, 1, tp, (LOCATION_EXTRA | LOCATION_GRAVE) as i32);
    Ok(false)
}

fn eqop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !(tc.is_relate_to_effect(duel, e) && tc.is_faceup(duel) && duel.get_location_count(tp, LOCATION_SZONE) > 0) {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_EQUIP);
    let filter = aux::necro_valley_filter(|c, duel| eqfilter(c, duel, tp));
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_EXTRA | LOCATION_GRAVE, 0, 1, 1, None)?;
    if let Some(ec) = g.get_first()
        && duel.equip(tp, ec, tc)?
    {
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_code(duel, EFFECT_EQUIP_LIMIT);
        e1.set_value(duel, Value::Card(super::c38423248::equip_limit));
        e1.set_label_object_group(duel, Some(tc.into()));
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        ec.register_effect(duel, e1);
    }
    Ok(())
}
