// Firewall Dragon Singularity
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, MZONE, OPPONENT};

const ID: u32 = 21637210;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 3;

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_type_full(duel, TYPE_EFFECT, Some(lc), sumtype, tp)
    }
}

// In the Extra Deck, Link Summoned with three Sangans (Effect Monsters), with the opponent's monsters to return, and a Cyberse monster
// (Digitron) in the Graveyard; its Link Summoned monsters in the zones it points to are destroyed by the opponent's Sangans.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (26202165, MZONE),
    (26202165, MZONE),
    (26202165, MZONE),
    (26202165, MZONE),
    (26202165, DECK),
    (32295838, GRAVE),
    (32295838, GRAVE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (12580477, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

const TYPES: u32 = TYPE_RITUAL | TYPE_FUSION | TYPE_SYNCHRO | TYPE_XYZ;

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_ATKCHANGE);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DAMAGE_STEP);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_hint_timing(duel, TIMING_DAMAGE_STEP, TIMING_DAMAGE_STEP | TIMINGS_CHECK_MONSTER_E);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, aux::stat_change_damage_step_condition);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_code(duel, EVENT_BATTLE_DESTROYED);
    e2.set_condition(duel, regcon);
    e2.set_operation(duel, regop);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EVENT_TO_GRAVE);
    e3.set_condition(duel, regcon2);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e4.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e4.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_SINGLE_RANGE + EFFECT_FLAG_DELAY);
    e4.set_code(duel, EVENT_CUSTOM + ID);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_count_limit(duel, 1, (ID, 1));
    e4.set_target(duel, sptg);
    e4.set_operation(duel, stock::special_summon_first_target::<POS_FACEUP>);
    c.register_effect(duel, e4);
}

fn getcount(duel: &Duel, tp: u8) -> usize {
    duel.get_matching_group(aux::faceup_filter(CardId::is_monster), tp, LOCATION_MZONE | LOCATION_GRAVE, 0, None)
        .get_bin_class_count(duel, |c, duel| u64::from(c.get_type(duel) & TYPES))
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let ct = getcount(duel, tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, 1 - tp)
            && chkc.is_location(duel, LOCATION_ONFIELD | LOCATION_GRAVE)
            && chkc.is_able_to_hand(duel));
    }
    if chk == 0 {
        return Ok(ct > 0 && duel.is_existing_target(CardId::is_able_to_hand, tp, 0, LOCATION_ONFIELD | LOCATION_GRAVE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
    let g = duel.select_target(
        tp,
        CardId::is_able_to_hand,
        tp,
        0,
        LOCATION_ONFIELD | LOCATION_GRAVE,
        1,
        ct as u8,
        None,
    )?;
    let count = g.map_or(0, |g| g.len() as u8);
    duel.set_operation_info(0, CATEGORY_TOHAND, g, count, 0, 0);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_target_cards(a.e).unwrap_or_default();
    if g.is_empty() || duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)? == 0 {
        return Ok(());
    }
    let ct = duel
        .get_operated_group()
        .filter_count(duel, |c, duel| c.is_location(duel, LOCATION_HAND), None);
    if ct > 0 {
        let c = a.e.get_handler(duel);
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_value(duel, ct as i32 * 500);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD_DISABLE);
        c.register_effect(duel, e1);
    }
    Ok(())
}

fn cfilter(c: CardId, duel: &Duel, tp: u8, zone: u32) -> bool {
    let mut seq = c.get_previous_sequence(duel);
    if !c.is_previous_controler(duel, tp) {
        seq += 16;
    }
    c.is_previous_location(duel, LOCATION_MZONE) && zone >> seq & 1 != 0
}

fn regcon(duel: &Duel, a: &Args) -> bool {
    let (tp, zone) = (a.tp, a.e.get_handler(duel).get_linked_zone(duel));
    a.eg.unwrap_or_default()
        .is_exists(duel, |c, duel| cfilter(c, duel, tp, zone), 1, None)
}

fn regcon2(duel: &Duel, a: &Args) -> bool {
    let (tp, zone) = (a.tp, a.e.get_handler(duel).get_linked_zone(duel));
    a.eg.unwrap_or_default().is_exists(
        duel,
        |c, duel| !c.is_reason(duel, REASON_BATTLE) && cfilter(c, duel, tp, zone),
        1,
        None,
    )
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    duel.raise_single_event(c, EVENT_CUSTOM + ID, Some(a.e), 0, a.tp, 0, 0)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_race(duel, RACE_CYBERSE) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && spfilter(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_target(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, tp, 0);
    Ok(false)
}
