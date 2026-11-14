// D/D Count Surveyor
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT, PZONE};

const ID: u32 = 5997110;

pub const LISTED_SERIES: &[u16] = &[SET_DD, SET_DDD];
// In the hand with D/D monsters there to discard, and D/D monsters with 0 ATK or DEF in the Deck to search; in a Pendulum
// Zone with D/D Scale Surveyor to Pendulum Summon Level 8 D/D/D monsters, and the opponent's Fiends to Tribute.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, PZONE),
    (42382265, PZONE),
    (74069667, HAND),
    (20715411, HAND),
    (72181263, HAND),
    (46796664, DECK),
    (11609969, DECK),
    (67322708, MZONE | OPPONENT),
    (67322708, MZONE | OPPONENT),
    (67322708, MZONE | OPPONENT),
    (33334269, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Pendulum::add_procedure(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_RELEASE + CATEGORY_ATKCHANGE + CATEGORY_DEFCHANGE);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_range(duel, LOCATION_PZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_condition(duel, |duel, a| {
        a.eg.is_some_and(|eg| eg.is_exists(duel, |c, duel| tributeconfilter(c, duel, a.tp), 1, None))
    });
    e1.set_target(duel, tributetg);
    e1.set_operation(duel, tributeop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_HAND);
    e2.set_count_limit(duel, 1, ID);
    e2.set_cost(duel, |duel, a, chk| {
        Cost::discard_full(
            duel,
            a,
            chk,
            |c, duel| c.is_set_card(duel, &[SET_DD]) && c.is_monster(duel),
            true,
            (1, 1),
        )
    });
    e2.set_target(duel, stock::special_summon_self_target);
    e2.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_SUMMON_SUCCESS);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_target(duel, thtg);
    e3.set_operation(duel, thop);
    c.register_effect(duel, e3);
    let e4 = e3.clone_effect(duel);
    e4.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e4);
}

fn tributeconfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_pendulum_summoned(duel) && c.is_summon_player(duel, tp) && c.is_set_card(duel, &[SET_DDD])
}

fn rescon(duel: &Duel, sg: CardSet) -> (bool, bool) {
    let same = sg.get_class_count(duel, |c, duel| i64::from(c.get_attribute(duel))) == 1
        || sg.get_class_count(duel, |c, duel| c.get_race(duel) as i64) == 1;
    (same && sg.is_exists(duel, CardId::is_releasable_by_effect, 2, None), false)
}

fn tributetg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chkc.is_some() {
        return Ok(false);
    }
    let filter = aux::faceup_filter(|c, duel| c.is_can_be_effect_target(duel, e));
    let g = duel.get_matching_group(filter, tp, 0, LOCATION_MZONE, None);
    let rescon = |duel: &Duel, sg, _, _| rescon(duel, sg);
    if chk == 0 {
        return Ok(g.len() >= 3 && aux::select_unselect_group_check(duel, g, 3, 3, &rescon));
    }
    let tg = aux::select_unselect_group(duel, g, 3, 3, &rescon, tp, HINTMSG_TARGET)?;
    duel.set_target_card(tg);
    duel.set_operation_info(0, CATEGORY_RELEASE, Some(tg), 2, tp, 0);
    duel.set_operation_info(0, CATEGORY_ATKCHANGE, Some(tg), 1, tp, 0);
    duel.set_operation_info(0, CATEGORY_DEFCHANGE, Some(tg), 1, tp, 0);
    Ok(false)
}

fn tributeop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let mut tg = duel.get_target_cards(e).unwrap_or_default().filter(duel, CardId::is_faceup, None);
    if tg.len() != 3 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RELEASE);
    let sg = tg.filter_select(duel, tp, CardId::is_releasable_by_effect, 2, 2, None)?;
    if sg.len() != 2 {
        return Ok(());
    }
    duel.hint_selection(sg);
    tg = tg.difference(sg);
    let atk = sg.get_sum(duel, |c, duel| i64::from(c.get_attack(duel)));
    let def = sg.get_sum(duel, |c, duel| i64::from(c.get_defense(duel)));
    if duel.release(sg, REASON_EFFECT)? == 2 {
        let c = e.get_handler(duel);
        let Some(atkdef_c) = tg.get_first() else { return Ok(()) };
        if atk > 0 {
            atkdef_c.update_attack(duel, atk as i32, Some(RESET_EVENT | RESETS_STANDARD), Some(c), None);
        }
        if def > 0 {
            atkdef_c.update_defense(duel, def as i32, Some(RESET_EVENT | RESETS_STANDARD), Some(c), None);
        }
    }
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_DD]) && c.is_monster(duel) && c.is_able_to_hand(duel) && (c.is_attack(duel, &[0]) || c.is_defense(duel, &[0]))
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}
