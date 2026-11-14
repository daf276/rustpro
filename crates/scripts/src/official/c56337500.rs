// Cyberse Reminder
use crate::prelude::*;
use crate::testing::{DECK, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 56337500;

pub const LISTED_SERIES: &[u16] = &[SET_CYNET];
// In the Extra Deck, Xyz Summoned with Level 3 Normal Monsters, with Cynet Mining in the Graveyard, Level 4 Cyberse monsters in the
// Deck, and the opponent's Raigeki to destroy it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (13039848, MZONE),
    (40826495, MZONE),
    (52121290, MZONE),
    (57160136, GRAVE),
    (47226949, DECK),
    (47226949, DECK),
    (53577438, DECK),
    (27182739, DECK),
    (12580477, HAND | OPPONENT),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(3);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, |duel, a, chk| {
        Cost::and(duel, a, chk, &[Cost::detach_from_self::<1, 1>, cost])
    });
    e1.set_target(duel, thtg);
    e1.set_operation(duel, stock::first_target_to_hand);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_DISABLE);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_DESTROYED);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, spcon);
    e2.set_cost(duel, cost);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
    duel.add_custom_activity_counter(
        ID,
        ACTIVITY_SPSUMMON,
        CounterFilter::Card(|duel, c| !c.is_summon_location(duel, LOCATION_EXTRA) || c.is_race(duel, RACE_CYBERSE)),
    );
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_custom_activity_count(ID, tp, ACTIVITY_SPSUMMON) == 0);
    }
    let c = a.e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_OATH + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_spsummon(duel, |duel, _, c, _, _, _, _, _| {
        c.is_location(duel, LOCATION_EXTRA) && !c.is_race(duel, RACE_CYBERSE)
    });
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    aux::add_temp_lizard_check_full(duel, c, tp, |duel, _, c| !c.is_original_race(duel, RACE_CYBERSE));
    Ok(false)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_CYNET]) && c.is_spell_trap(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && thfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(thfilter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_target(tp, thfilter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOHAND, g, 1, tp, 0);
    Ok(false)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let (tp, c) = (a.tp, a.e.get_handler(duel));
    c.is_xyz_summoned(duel)
        && (c.is_reason(duel, REASON_BATTLE) || (a.rp == 1 - tp && c.is_reason(duel, REASON_EFFECT) && c.is_previous_controler(duel, tp)))
        && c.is_previous_location(duel, LOCATION_MZONE)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_level(duel, &[4]) && c.is_race(duel, RACE_CYBERSE) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let g = duel.get_matching_group(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, None);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) >= 2
            && duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none()
            && aux::select_unselect_group_check(duel, g, 2, 2, &aux::dncheck));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 2, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) < 2 || duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
        return Ok(());
    }
    let g = duel.get_matching_group(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, None);
    let sg = aux::select_unselect_group(duel, g, 2, 2, &aux::dncheck, tp, HINTMSG_SPSUMMON)?;
    if sg.len() != 2 {
        return Ok(());
    }
    let c = e.get_handler(duel);
    for sc in sg.iter() {
        if duel.special_summon_step(sc, 0, tp, tp, false, false, POS_FACEUP)? {
            sc.negate_effects(duel, c, 0, false);
        }
    }
    duel.special_summon_complete()?;
    Ok(())
}
