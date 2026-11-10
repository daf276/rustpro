// Terrors of the Overroot
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT, SZONE};
const ID: u32 = 63086455;

// In the hand, with Sangan and a Set Spell on the opponent's field to send, and a Normal Monster (Kojikocy) and
// Mystical Space Typhoon in their Graveyard to Special Summon or Set.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (26202165, MZONE | OPPONENT),
    (5318639, SZONE | OPPONENT),
    (1184620, GRAVE | OPPONENT),
    (5318639, GRAVE | OPPONENT),
];
// The face-down Special Summon is the monster path; the Spell and Trap path names no card of its own.
pub const CHECKS: &[Check] = &[Check::Resolves, Check::SpecialSummonedCode(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOGRAVE + CATEGORY_LEAVE_GRAVE + CATEGORY_SET);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER_E | TIMING_MAIN_END);
    e1.set_target(duel, tgtg);
    e1.set_operation(duel, tgop);
    c.register_effect(duel, e1);
}

fn tgfilter(c: CardId, duel: &Duel, e: EffectId) -> bool {
    c.is_able_to_grave(duel) && c.is_can_be_effect_target(duel, e)
}

fn setfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_can_be_effect_target(duel, e)
        && ((c.is_spell_trap(duel) && c.is_ssetable_full(duel, true, Some(1 - tp)))
            || c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEDOWN_DEFENSE, 1 - tp, 0xff))
}

fn rescon(duel: &Duel, sg: CardSet, tp: u8) -> (bool, bool) {
    let (g1, g2) = sg.split(duel, CardId::is_on_field, None);
    let (Some(c1), Some(c2)) = (g1.get_first(), g2.get_first()) else {
        return (false, false);
    };
    if g1.len() != 1 || g2.len() != 1 {
        return (false, false);
    }
    if c2.is_monster(duel) {
        return (duel.get_mzone_count(1 - tp, Some(c1.into())) > 0, false);
    }
    (
        (c2.is_spell(duel) && c2.is_type(duel, TYPE_FIELD))
            || duel.get_location_count(1 - tp, LOCATION_SZONE) > 0
            || (c1.is_location(duel, LOCATION_SZONE) && c1.get_sequence(duel) < 5),
        false,
    )
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chkc.is_some() {
        return Ok(false);
    }
    let g1 = duel.get_matching_group(|c, duel| tgfilter(c, duel, e), tp, 0, LOCATION_ONFIELD, None);
    let g2 = duel.get_matching_group(|c, duel| setfilter(c, duel, e, tp), tp, 0, LOCATION_GRAVE, None);
    let g = g1.union(g2);
    let rescon = |duel: &Duel, sg, _, _| rescon(duel, sg, tp);
    if chk == 0 {
        return Ok(!g1.is_empty() && !g2.is_empty() && aux::select_unselect_group_check(duel, g, 2, 2, &rescon));
    }
    let tg = aux::select_unselect_group(duel, g, 2, 2, &rescon, tp, HINTMSG_TARGET)?;
    duel.set_target_card(tg);
    let (g1, g2) = tg.split(duel, CardId::is_on_field, None);
    duel.set_operation_info(0, CATEGORY_TOGRAVE, Some(g1), 1, 0, 0);
    let cat = e.get_category(duel);
    if g2.get_first().is_some_and(|c| c.is_monster(duel)) {
        e.set_category(duel, cat | CATEGORY_SPECIAL_SUMMON);
        duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(g2), 1, 0, 0);
    } else {
        e.set_category(duel, cat & !CATEGORY_SPECIAL_SUMMON);
        duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, Some(g2), 1, 0, 0);
    }
    Ok(false)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let tg = duel.get_target_cards(a.e).unwrap_or_default();
    if tg.is_empty() {
        return Ok(());
    }
    let (g1, g2) = tg.split(duel, CardId::is_on_field, None);
    if !g1.is_empty() && duel.send_to_grave(g1, REASON_EFFECT)? > 0 && g1.get_first().is_some_and(|c| c.is_location(duel, LOCATION_GRAVE)) {
        let Some(sc) = g2.get_first() else { return Ok(()) };
        if sc.is_monster(duel)
            && duel.get_location_count(1 - tp, LOCATION_MZONE) > 0
            && duel.special_summon(sc, 0, tp, 1 - tp, false, false, POS_FACEDOWN_DEFENSE)? > 0
        {
            duel.confirm_cards(1 - tp, sc)?;
        } else if sc.is_spell_trap(duel) && sc.is_ssetable_full(duel, false, Some(1 - tp)) {
            duel.sset_full(tp, sc, 1 - tp, true)?;
        }
    }
    Ok(())
}
