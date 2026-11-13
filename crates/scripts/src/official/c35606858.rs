// Masterking Archfiend
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 35606858;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_race_full(duel, RACE_FIEND, Some(lc), sumtype, tp)
    }
}

// On the field with Ryu-Kishin (DEF 500, the Standby Phase LP payment) in the Deck to send, Fiends on the field and the opponent's Raigeki
// to send them to the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (15303296, DECK),
    (15303296, DECK),
    (15303296, DECK),
    (24611934, MZONE),
    (24611934, MZONE),
    (12580477, HAND | OPPONENT),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EVENT_PHASE | PHASE_STANDBY);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_condition(duel, |duel, a| duel.is_turn_player(a.tp));
    e1.set_operation(duel, mtop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TOGRAVE);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_PAY_LPCOST);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit_full(duel, 1, 0, EFFECT_COUNT_CODE_CHAIN);
    e2.set_condition(duel, |_, a| a.ep == a.tp);
    e2.set_target(duel, tgtg);
    e2.set_operation(duel, tgop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_DICE + CATEGORY_TOHAND + CATEGORY_TODECK + CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_CUSTOM + ID);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit_full(duel, 1, 0, EFFECT_COUNT_CODE_CHAIN);
    e3.set_target(duel, dctg);
    e3.set_operation(duel, dcop);
    e3.set_label_object_group(duel, Some(CardSet::default()));
    c.register_effect(duel, e3);
    let e3a = duel.create_effect(c);
    e3a.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e3a.set_code(duel, EVENT_TO_GRAVE);
    e3a.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e3a.set_range(duel, LOCATION_MZONE);
    e3a.set_label_object(duel, Some(e3));
    e3a.set_operation(duel, regop);
    c.register_effect(duel, e3a);
}

fn mtop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if duel.check_lp_cost(tp, 500) && duel.select_yes_no(tp, aux::stringid(ID, 2))? {
        duel.pay_lp_cost(tp, 500)?;
    } else {
        duel.destroy(a.e.get_handler(duel), REASON_COST)?;
    }
    Ok(())
}

fn tgfilter(c: CardId, duel: &Duel, val: i32) -> bool {
    c.is_race(duel, RACE_FIEND)
        && c.is_monster(duel)
        && (c.is_attack(duel, &[val]) || c.is_defense(duel, &[val]))
        && c.is_able_to_grave(duel)
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (tp, ev) = (a.tp, a.ev as i32);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| tgfilter(c, duel, ev), tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (tp, ev) = (a.tp, a.ev as i32);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, |c, duel| tgfilter(c, duel, ev), tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_grave(g, REASON_EFFECT)?;
    }
    Ok(())
}

fn cfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, ft: i32) -> bool {
    c.is_race(duel, RACE_FIEND)
        && c.is_controler(duel, tp)
        && (c.is_able_to_hand(duel) || c.is_able_to_deck(duel) || (ft > 0 && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)))
        && c.has_flag_effect(duel, ID + 1)
}

fn regfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_race(duel, RACE_FIEND) && c.is_controler(duel, tp)
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let tg = a.eg.unwrap_or_default().filter(duel, |c, duel| regfilter(c, duel, tp), None);
    if tg.is_empty() {
        return Ok(());
    }
    let reset = RESET_EVENT | RESET_REMOVE | RESET_TEMP_REMOVE | RESET_TOHAND | RESET_TODECK | RESET_LEAVE | RESET_TOFIELD;
    for tc in tg {
        tc.register_flag_effect(duel, ID, RESET_CHAIN, 0, 1);
        tc.register_flag_effect(duel, ID + 1, reset, 0, 1);
    }
    let Some(e3) = e.get_label_object(duel) else { return Ok(()) };
    let mut g = e3.get_label_object_group(duel).unwrap_or_default();
    if duel.get_current_chain() == 0 {
        g.clear();
    }
    g.merge(tg);
    g.remove_matching(duel, |c, duel| c.get_flag_effect(duel, ID) == 0, None);
    e3.set_label_object_group(duel, Some(g));
    duel.raise_single_event(e.get_handler(duel), EVENT_CUSTOM + ID, Some(e), 0, tp, tp, 0)
}

fn dctg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let ft = duel.get_location_count(tp, LOCATION_MZONE);
    let g = e
        .get_label_object_group(duel)
        .unwrap_or_default()
        .filter(duel, |c, duel| cfilter(c, duel, e, tp, ft), None);
    if chk == 0 {
        return Ok(!g.is_empty());
    }
    duel.set_target_card(g);
    duel.set_operation_info(0, CATEGORY_DICE, None, 0, tp, 1);
    duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, Some(g), 1, tp, 0);
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_GRAVE as i32);
    duel.set_possible_operation_info(0, CATEGORY_TODECK, None, 1, tp, LOCATION_GRAVE as i32);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_GRAVE as i32);
    Ok(false)
}

fn dcop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let ft = duel.get_location_count(tp, LOCATION_MZONE);
    let g = e
        .get_label_object_group(duel)
        .unwrap_or_default()
        .filter(duel, |c, duel| cfilter(c, duel, e, tp, ft), None);
    if g.is_empty() {
        return Ok(());
    }
    match duel.toss_dice(tp, 1)?.first() {
        Some(1) => {
            let Some(tc) = mon_select(duel, g, tp, CardId::is_able_to_hand, HINTMSG_ATOHAND)? else {
                return Ok(());
            };
            duel.send_to_hand(tc, PLAYER_NONE, REASON_EFFECT)?;
            duel.confirm_cards(1 - tp, tc)?;
        }
        Some(6) => {
            if duel.get_location_count(tp, LOCATION_MZONE) == 0 {
                return Ok(());
            }
            let filter = |c: CardId, duel: &Duel| c.is_can_be_special_summoned(duel, e, 0, tp, false, false);
            let Some(tc) = mon_select(duel, g, tp, filter, HINTMSG_SPSUMMON)? else {
                return Ok(());
            };
            duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP)?;
        }
        Some(2..=5) => {
            let Some(tc) = mon_select(duel, g, tp, CardId::is_able_to_deck, HINTMSG_TODECK)? else {
                return Ok(());
            };
            duel.send_to_deck(tc, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)?;
        }
        _ => {}
    }
    Ok(())
}

fn mon_select(duel: &mut Duel, g: CardSet, tp: u8, func: impl Fn(CardId, &Duel) -> bool, hint: u64) -> Script<Option<CardId>> {
    let tc = if g.len() > 1 {
        duel.hint(HINT_SELECTMSG, tp, hint);
        g.filter_select(duel, tp, &func, 1, 1, None)?.get_first()
    } else {
        g.get_first().filter(|&tc| func(tc, duel))
    };
    if let Some(tc) = tc {
        duel.hint_selection(tc);
    }
    Ok(tc)
}
