// Jurrac Stego
use crate::prelude::*;
use crate::testing::{Check, DECK, MZONE, OPPONENT};

const ID: u32 = 20568404;

pub const LISTED_NAMES: &[u32] = &[ID];
// Two on the field, Uraby (Level 4 Dinosaur) and Jurrac Megalo (Level 1) in the Deck to send and to summon, and a strong
// attacker on the opponent's side to destroy it in battle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (1784619, DECK),
    (1784619, DECK),
    (93170499, DECK),
    (93170499, DECK),
    (93170499, DECK),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::SpecialSummonedCode(93170499),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY + CATEGORY_TOGRAVE + CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_BATTLE_DESTROYED);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_DINOSAUR) && c.is_able_to_grave(duel)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let g = duel.get_matching_group(aux::TRUE, tp, LOCATION_ONFIELD, 0, None);
    if chk == 0 {
        return Ok(!g.is_empty() && duel.is_existing_matching_card(tgfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), 1, tp, 0);
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, LOCATION_DECK as i32);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, (LOCATION_HAND | LOCATION_DECK) as i32);
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.has_level(duel)
        && c.is_set_card(duel, &[SET_JURRAC])
        && !c.is_code(duel, ID)
        && c.is_can_be_special_summoned(duel, e, 0, tp, true, false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let e1 = duel.create_effect(e.get_handler(duel));
    e1.set_description(duel, aux::stringid(ID, 3));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_spsummon(duel, |duel, _, c, _, _, _, _, _| !c.is_race(duel, RACE_DINOSAUR));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let dg = duel.select_matching_card(tp, aux::TRUE, tp, LOCATION_ONFIELD, 0, 1, 1, None)?;
    if dg.is_empty() {
        return Ok(());
    }
    duel.hint_selection(dg);
    if duel.destroy(dg, REASON_EFFECT)? == 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let Some(tgc) = duel.select_matching_card(tp, tgfilter, tp, LOCATION_DECK, 0, 1, 1, None)?.first() else {
        return Ok(());
    };
    if duel.send_to_grave(tgc, REASON_EFFECT)? > 0 && tgc.is_location(duel, LOCATION_GRAVE) {
        let mut ft = duel.get_location_count(tp, LOCATION_MZONE);
        if ft == 0 {
            return Ok(());
        }
        if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
            ft = 1;
        }
        let lvl = tgc.get_level(duel);
        let g = duel.get_matching_group(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_HAND | LOCATION_DECK, 0, None);
        if !g.is_empty()
            && g.check_with_sum_equal(duel, CardId::get_level, lvl, 1, ft as usize)
            && duel.select_yes_no(tp, aux::stringid(ID, 2))?
        {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
            let sg = g.select_with_sum_equal(duel, tp, CardId::get_level, lvl, 1, ft)?;
            if sg.is_empty() {
                return Ok(());
            }
            duel.break_effect()?;
            duel.special_summon(sg, 0, tp, tp, true, false, POS_FACEUP)?;
        }
    }
    Ok(())
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(
            aux::faceup_filter(CardId::is_able_to_hand),
            tp,
            LOCATION_ONFIELD,
            LOCATION_ONFIELD,
            1,
            None,
        ));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_ONFIELD as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
    let g = duel.select_matching_card(
        tp,
        aux::faceup_filter(CardId::is_able_to_hand),
        tp,
        LOCATION_ONFIELD,
        LOCATION_ONFIELD,
        1,
        1,
        None,
    )?;
    if !g.is_empty() {
        duel.hint_selection(g);
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}
