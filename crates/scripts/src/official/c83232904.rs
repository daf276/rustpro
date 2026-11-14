// Clown Crew Malabarisme
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND, MZONE};

const ID: u32 = 83232904;

pub const LISTED_SERIES: &[u16] = &[SET_CLOWN_CREW];
// In the hand and the Graveyard, with Clown Crew monsters in the Deck and the Extra Deck to Special Summon and a monster on the
// field to Tribute for the Graveyard copy: Mystical Elf.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (82159583, DECK),
    (42759961, DECK),
    (31533473, EXTRA),
    (15025844, MZONE),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummonedCode(82159583),
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, aux::exccon);
    e2.set_cost(duel, |duel, a, chk| Cost::and(duel, a, chk, &[Cost::self_banish, rthcost]));
    e2.set_target(duel, rthtg);
    e2.set_operation(duel, stock::first_target_to_hand);
    c.register_effect(duel, e2);
}

pub fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, mmz_chk: bool) -> bool {
    if !(c.is_set_card(duel, &[SET_CLOWN_CREW]) && c.is_monster(duel) && c.is_can_be_special_summoned(duel, e, 0, tp, true, false)) {
        return false;
    }
    if c.is_location(duel, LOCATION_DECK) {
        mmz_chk
    } else {
        c.is_location(duel, LOCATION_EXTRA) && duel.get_location_count_from_ex(tp, tp, None, c) > 0
    }
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let mmz_chk = duel.get_location_count(tp, LOCATION_MZONE) > 0;
        return Ok(duel.is_existing_matching_card(
            |c, duel| spfilter(c, duel, e, tp, mmz_chk),
            tp,
            LOCATION_DECK | LOCATION_EXTRA,
            0,
            1,
            None,
        ));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, (LOCATION_DECK | LOCATION_EXTRA) as i32);
    Ok(false)
}

fn pendlinkfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_location(duel, LOCATION_EXTRA) && c.is_faceup(duel)) || c.is_link_monster(duel)
}

// The "cannot activate the effects of monsters Special Summoned from the Deck or Extra Deck until the end of the next turn" effect.
pub fn register_activation_limit(duel: &mut Duel, e: EffectId, tp: u8, description: u64) {
    let e1 = duel.create_effect(e.get_handler(duel));
    e1.set_description(duel, description);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e1.set_target_range(duel, 1, 0);
    e1.set_value(duel, Value::EffectPlayer(aclimit));
    e1.set_reset_full(duel, RESET_PHASE | PHASE_END, 2);
    duel.register_effect(e1, tp);
}

// Special Summons up to 2 "Clown Crew" monsters from the Deck and/or Extra Deck, ignoring their Summoning conditions.
pub fn special_summon_clown_crew(duel: &mut Duel, e: EffectId, tp: u8) -> Script<()> {
    let mmz_ct = duel.get_location_count(tp, LOCATION_MZONE);
    let emz_ct = duel.get_location_count_from_ex_full(tp, tp, None, None, ZONES_EMZ);
    let linkmz_ct = duel.get_location_count_from_ex(tp, tp, None, None);
    let mut ft = (mmz_ct + emz_ct).min(2);
    if ft <= 0 {
        return Ok(());
    }
    if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
        ft = 1;
    }
    let g = duel.get_matching_group(
        |c, duel| spfilter(c, duel, e, tp, mmz_ct > 0),
        tp,
        LOCATION_DECK | LOCATION_EXTRA,
        0,
        None,
    );
    if g.is_empty() {
        return Ok(());
    }
    let rescon = |duel: &Duel, sg: CardSet, _: CardSet, _: Option<CardId>| {
        let in_deck = sg.filter_count(duel, |c, duel| c.is_location(duel, LOCATION_DECK), None);
        (
            in_deck as i32 <= mmz_ct && sg.filter_count(duel, pendlinkfilter, None) as i32 <= linkmz_ct,
            false,
        )
    };
    let sg = aux::select_unselect_group(duel, g, 1, ft as usize, &rescon, tp, HINTMSG_SPSUMMON)?;
    if sg.is_empty() {
        return Ok(());
    }
    let (fup, fdown) = sg.split(duel, pendlinkfilter, None);
    let (fdown_main, fdown_ex) = fdown.split(duel, |c, duel| c.is_location(duel, LOCATION_DECK), None);
    let (priority_0, priority_1) = if linkmz_ct < mmz_ct { (fup, fdown_main) } else { (fdown_main, fup) };
    for c in priority_0.into_iter().chain(priority_1).chain(fdown_ex) {
        duel.special_summon_step(c, 0, tp, tp, true, false, POS_FACEUP)?;
    }
    duel.special_summon_complete()?;
    Ok(())
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if e.is_has_type(duel, EFFECT_TYPE_ACTIVATE) {
        register_activation_limit(duel, e, tp, aux::stringid(ID, 2));
    }
    special_summon_clown_crew(duel, e, tp)
}

pub fn aclimit(duel: &Duel, _: EffectId, re: EffectId, _: u8) -> bool {
    let rc = re.get_handler(duel);
    re.is_monster_effect(duel) && rc.is_on_field(duel) && rc.is_summon_location(duel, LOCATION_DECK | LOCATION_EXTRA)
}

fn rthcostfilter(c: CardId, duel: &Duel) -> bool {
    duel.is_existing_target(CardId::is_able_to_hand, 0, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, Some(c.into()))
}

fn rthcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.check_release_group_cost(tp, rthcostfilter, (1, 1), true, None, None));
    }
    let g = duel.select_release_group_cost(tp, rthcostfilter, (1, 1), true, None, None)?;
    duel.release(g, REASON_COST)?;
    Ok(false)
}

fn rthtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_able_to_hand(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_able_to_hand, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
    let g = duel.select_target(tp, CardId::is_able_to_hand, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOHAND, g, 1, tp, 0);
    Ok(false)
}
