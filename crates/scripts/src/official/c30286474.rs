// Gouki The Master Ogre
use super::c22510667::Procedure;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 30286474;

// Link Summoned from Gouki Guts and Gouki Machine Suprex, with the opponent's monsters to negate and attack.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (7540107, MZONE),
    (29099860, MZONE),
    (7540107, MZONE),
    (7540107, MZONE | OPPONENT),
    (29099860, MZONE | OPPONENT),
    (7540107, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Link::add_procedure::<Procedure<2, 0>>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DISABLE);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER);
    e1.set_count_limit(duel, 1, 0);
    e1.set_cost(duel, discost);
    e1.set_target(duel, distg);
    e1.set_operation(duel, disop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_ATTACK_ALL);
    e2.set_value(duel, 1);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_code(duel, EFFECT_CANNOT_SELECT_BATTLE_TARGET);
    e3.set_condition(duel, atcon);
    e3.set_value(duel, Value::Card(|duel, e, c| i32::from(atlimit(duel, e, c))));
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_SINGLE);
    e4.set_code(duel, EFFECT_CANNOT_DIRECT_ATTACK);
    e4.set_condition(duel, atcon);
    c.register_effect(duel, e4);
}

fn discost(duel: &mut Duel, a: &Args, _: i32) -> Script<bool> {
    a.e.set_label(duel, 1);
    Ok(true)
}

fn costfilter(c: CardId, duel: &Duel, g: CardSet) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_GOUKI]) && g.is_contains(c) && c.is_able_to_hand_as_cost(duel)
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let lg = e.get_handler(duel).get_linked_group(duel);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_controler(duel, 1 - tp) && chkc.is_negatable(duel));
    }
    if chk == 0 {
        if e.get_label(duel) != 1 {
            return Ok(false);
        }
        e.set_label(duel, 0);
        return Ok(
            duel.is_existing_matching_card(|c, duel| costfilter(c, duel, lg), tp, LOCATION_MZONE, 0, 1, None)
                && duel.is_existing_target(CardId::is_negatable, tp, 0, LOCATION_ONFIELD, 1, None),
        );
    }
    e.set_label(duel, 0);
    let rt = duel.get_target_count(CardId::is_negatable, tp, 0, LOCATION_ONFIELD, None);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
    let cg = duel.select_matching_card(tp, |c, duel| costfilter(c, duel, lg), tp, LOCATION_MZONE, 0, 1, rt as u8, None)?;
    let ct = cg.len() as u8;
    duel.send_to_hand(cg, PLAYER_NONE, REASON_COST)?;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    let g = duel.select_target(tp, CardId::is_negatable, tp, 0, LOCATION_ONFIELD, ct, ct, None)?;
    duel.set_operation_info(0, CATEGORY_DISABLE, g, g.map_or(0, |g| g.len() as u8), 0, 0);
    Ok(false)
}

fn disfilter(c: CardId, duel: &Duel, e: EffectId) -> bool {
    ((c.is_faceup(duel) && !c.is_disabled(duel)) || c.is_type(duel, TYPE_TRAPMONSTER)) && c.is_relate_to_effect(duel, e)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c) = (a.e, a.e.get_handler(duel));
    let g = duel
        .get_chain_info::<Option<CardSet>>(0, CHAININFO_TARGET_CARDS)
        .unwrap_or_default();
    for tc in g.filter(duel, |c, duel| disfilter(c, duel, e), None) {
        duel.negate_related_chain(tc, RESET_TURN_SET);
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_code(duel, EFFECT_DISABLE);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        tc.register_effect(duel, e1);
        let e2 = duel.create_effect(c);
        e2.set_type(duel, EFFECT_TYPE_SINGLE);
        e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e2.set_code(duel, EFFECT_DISABLE_EFFECT);
        e2.set_value(duel, RESET_TURN_SET as i32);
        e2.set_reset(duel, RESETS_STANDARD_PHASE_END);
        tc.register_effect(duel, e2);
        if tc.is_type(duel, TYPE_TRAPMONSTER) {
            let e3 = duel.create_effect(c);
            e3.set_type(duel, EFFECT_TYPE_SINGLE);
            e3.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
            e3.set_code(duel, EFFECT_DISABLE_TRAPMONSTER);
            e3.set_reset(duel, RESETS_STANDARD_PHASE_END);
            tc.register_effect(duel, e3);
        }
    }
    Ok(())
}

fn atcon(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(CardId::is_faceup, a.e.get_handler_player(duel), 0, LOCATION_MZONE, 1, None)
}

fn atlimit(duel: &Duel, e: EffectId, c: CardId) -> bool {
    let g = duel.get_matching_group(CardId::is_faceup, e.get_handler_player(duel), 0, LOCATION_MZONE, None);
    let tg = g.get_max_group(duel, |c, duel| i64::from(c.get_attack(duel)));
    tg.is_none_or(|(tg, _)| !tg.is_contains(c)) || c.is_facedown(duel)
}
