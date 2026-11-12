// Angelica, Princess of Noble Arms
use super::c37675907::Procedure;
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 36320744;

pub const LISTED_NAMES: &[u32] = &[CARD_INFERNOBLE_CHARLES, 55749927];
// A copy on the field and two in the Extra Deck, Synchro Summoned with Infernoble Knight Ricciardetto (a Level 1 Tuner) and Noble Knight Artorigus
// (Level 4), with Horn of Olifant and Burstinatrix (a FIRE Warrior) in the Deck, Roland in the Extra Deck, and Mind Control in the opponent's hand to target it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (ID, EXTRA),
    (53404966, MZONE),
    (53404966, MZONE),
    (92125819, MZONE),
    (92125819, MZONE),
    (55749927, DECK),
    (55749927, DECK),
    (58932615, DECK),
    (58932615, DECK),
    (40251688, EXTRA),
    (13597785, MZONE | OPPONENT),
    (37520316, HAND | OPPONENT),
    (37520316, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOGRAVE + CATEGORY_REMOVE + CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_BECOME_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| a.eg.unwrap_or_default().contains(a.e.get_handler(duel)));
    e2.set_target(duel, tgtg);
    e2.set_operation(duel, tgop);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EVENT_BE_BATTLE_TARGET);
    c.register_effect(duel, e3);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    (c.lists_code(duel, CARD_INFERNOBLE_CHARLES) || c.is_code(duel, 55749927)) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_WARRIOR) && c.is_attribute(duel, ATTRIBUTE_FIRE) && c.is_able_to_grave(duel)
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_able_to_remove(duel) && duel.is_existing_matching_card(tgfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, LOCATION_DECK as i32);
    duel.set_operation_info(0, CATEGORY_REMOVE, Some(c.into()), 1, 0, 0);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, (LOCATION_DECK | LOCATION_EXTRA) as i32);
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    if !(c.is_set_card(duel, &[SET_ROLAND]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)) {
        return false;
    }
    if c.is_location(duel, LOCATION_DECK) {
        return duel.get_location_count(tp, LOCATION_MZONE) > 0;
    }
    duel.get_location_count_from_ex(tp, tp, None, c) > 0
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, tgfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    let c = e.get_handler(duel);
    if !g.is_empty()
        && duel.send_to_grave(g, REASON_EFFECT)? > 0
        && g.get_first().is_some_and(|gc| gc.is_location(duel, LOCATION_GRAVE))
        && c.is_relate_to_effect(duel, e)
        && aux::remove_until::<aux::DefaultFieldReturnOp>(duel, c, 0, REASON_EFFECT, PHASE_END, ID, e, tp)?.is_some()
        && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK | LOCATION_EXTRA, 0, 1, None)
        && duel.select_yes_no(tp, aux::stringid(ID, 2))?
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let sg = duel.select_matching_card(
            tp,
            |c, duel| spfilter(c, duel, e, tp),
            tp,
            LOCATION_DECK | LOCATION_EXTRA,
            0,
            1,
            1,
            None,
        )?;
        if sg.is_empty() {
            return Ok(());
        }
        duel.break_effect()?;
        duel.special_summon(sg, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
