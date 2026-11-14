// Odd-Eyes Pendulumgraph Dragon
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE, OPPONENT, PZONE};

const ID: u32 = 66425726;
const ODD_EYES_ADVENT: u32 = 16494704;

pub const LISTED_NAMES: &[u32] = &[ODD_EYES_ADVENT];
pub const LISTED_SERIES: &[u16] = &[SET_ODD_EYES];
// In the Pendulum Zone with Odd-Eyes Advent in the Deck, and Ritual Summoned by it with a Pendulum material from the hand,
// against the opponent's Spells (Dark Hole, Pot of Greed) with Odd-Eyes Vortex Dragon in the Extra Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, PZONE),
    (ODD_EYES_ADVENT, DECK),
    (ODD_EYES_ADVENT, DECK),
    (ID, HAND),
    (ODD_EYES_ADVENT, HAND),
    (14105623, HAND),
    (ID, MZONE),
    (53262004, EXTRA),
    (53129443, HAND | OPPONENT),
    (55144522, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Pendulum::add_procedure(duel, c);
    c.enable_unsummonable(duel);
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_SINGLE);
    e0.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e0.set_code(duel, EFFECT_REVIVE_LIMIT);
    e0.set_condition(duel, |duel, a| !a.e.get_handler(duel).is_location(duel, LOCATION_HAND));
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EFFECT_SPSUMMON_CONDITION);
    e1.set_value(duel, Value::SpSummonCondition(splimit));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_PHASE + PHASE_END);
    e2.set_range(duel, LOCATION_PZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e3.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_condition(duel, |duel, a| {
        let tp = a.tp;
        a.eg.unwrap_or_default().is_exists(
            duel,
            |c, duel| c.is_summon_player(duel, 1 - tp) && c.is_summon_location(duel, LOCATION_EXTRA),
            1,
            None,
        )
    });
    e3.set_operation(duel, damop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_category(duel, CATEGORY_DISABLE + CATEGORY_SPECIAL_SUMMON);
    e4.set_type(duel, EFFECT_TYPE_QUICK_O);
    e4.set_code(duel, EVENT_CHAINING);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_count_limit(duel, 1, 0);
    e4.set_condition(duel, |duel, a| {
        a.rp == 1 - a.tp && a.re.is_some_and(|re| re.is_spell_effect(duel)) && duel.is_chain_disablable(a.ev as u8)
    });
    e4.set_target(duel, distg);
    e4.set_operation(duel, disop);
    c.register_effect(duel, e4);
}

fn splimit(duel: &Duel, e: EffectId, _: Option<EffectId>, _: u8, st: u32, _: u8, _: u8) -> bool {
    st & SUMMON_TYPE_RITUAL == SUMMON_TYPE_RITUAL
        || (st & SUMMON_TYPE_PENDULUM == SUMMON_TYPE_PENDULUM && e.get_handler(duel).is_location(duel, LOCATION_HAND))
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_ritual_spell(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_able_to_hand(duel) && duel.is_existing_matching_card(thfilter, a.tp, LOCATION_DECK | LOCATION_GRAVE, 0, 1, None));
    }
    let locations = LOCATION_DECK | LOCATION_GRAVE | LOCATION_PZONE;
    duel.set_operation_info(0, CATEGORY_TOHAND, Some(c.into()), 2, a.tp, locations as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let filter = aux::necro_valley_filter(thfilter);
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_DECK | LOCATION_GRAVE, 0, 1, 1, None)?;
    if !g.is_empty()
        && duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)? > 0
        && g.get_first().is_some_and(|tc| tc.is_location(duel, LOCATION_HAND))
    {
        duel.confirm_cards(1 - tp, g)?;
        duel.break_effect()?;
        if duel.send_to_hand(c, PLAYER_NONE, REASON_EFFECT)? > 0 {
            duel.confirm_cards(1 - tp, c)?;
        }
    }
    Ok(())
}

fn damop(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.hint(HINT_CARD, 0, u64::from(ID));
    duel.damage(1 - a.tp, 300, REASON_EFFECT)?;
    Ok(())
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_type(duel, TYPE_PENDULUM) && duel.check_pendulum_zones(a.tp));
    }
    duel.set_operation_info(0, CATEGORY_DISABLE, a.eg, 1, 0, 0);
    if c.is_ritual_summoned(duel) {
        duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, a.tp, LOCATION_EXTRA as i32);
    }
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, mc: CardId) -> bool {
    // The Lua tests the count itself, a number, which is always true.
    let _ = duel.get_location_count_from_ex(tp, tp, Some(mc.into()), c);
    c.is_set_card(duel, &[SET_ODD_EYES]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) || c.is_immune_to_effect(duel, e) {
        return Ok(());
    }
    if !duel.check_pendulum_zones(tp) {
        duel.send_to_grave_full(c, REASON_RULE, PLAYER_NONE, PLAYER_NONE)?;
        return Ok(());
    }
    let mut sg = CardSet::default();
    if c.is_ritual_summoned(duel) {
        sg = duel.get_matching_group(|sc, duel| spfilter(sc, duel, e, tp, c), tp, LOCATION_EXTRA, 0, None);
    }
    if duel.move_to_field(c, tp, tp, LOCATION_PZONE, POS_FACEUP, true)?
        && duel.negate_effect(a.ev as u8)
        && !sg.is_empty()
        && duel.select_yes_no(tp, aux::stringid(ID, 2))?
    {
        duel.break_effect()?;
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        if let Some(sc) = sg.select(duel, tp, 1, 1, None)?.get_first() {
            duel.special_summon(sc, 0, tp, tp, false, false, POS_FACEUP)?;
        }
    }
    Ok(())
}
