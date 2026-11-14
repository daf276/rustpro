// Supreme King Z-ARC - Synchro Universe
use super::c70771599::Procedure;
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE, OPPONENT, PZONE};

const ID: u32 = 48654267;

pub const LISTED_SERIES: &[u16] = &[SET_SUPREME_KING_GATE, SET_SUPREME_KING_DRAGON];
pub const LISTED_NAMES: &[u32] = &[CARD_ZARC];
// In the Extra Deck, Synchro Summoned with Performapal Odd-Eyes Synchron (a Pendulum Tuner) and Junoldo the Shadespirit Power Patron
// (a DARK Pendulum Monster); in the Pendulum Zone to be Special Summoned by releasing Supreme King Dragon Darkwurm, with more
// Darkwurm in the Deck to summon, and against the opponent's Sangan and Dark Hole.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, PZONE),
    (82224646, MZONE),
    (82224646, MZONE),
    (10266279, MZONE),
    (10266279, MZONE),
    (69610326, MZONE),
    (69610326, DECK),
    (69610326, DECK),
    (26202165, MZONE | OPPONENT),
    (53129443, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Pendulum::add_procedure_full(duel, c, false, None);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_PZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, selfspcost);
    e1.set_target(duel, selfsptg);
    e1.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_code(duel, EFFECT_CHANGE_CODE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_value(duel, CARD_ZARC as i32);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_BATTLED);
    e3.set_condition(duel, spcon);
    e3.set_target(duel, sptg);
    e3.set_operation(duel, spop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 2));
    e4.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e4.set_property(duel, EFFECT_FLAG_DELAY);
    e4.set_code(duel, EVENT_DESTROYED);
    e4.set_condition(duel, pencon);
    e4.set_target(duel, pentg);
    e4.set_operation(duel, penop);
    c.register_effect(duel, e4);
}

fn cfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_SUPREME_KING_GATE, SET_SUPREME_KING_DRAGON])
        && c.is_type(duel, TYPE_PENDULUM)
        && duel.get_mzone_count(tp, Some(c.into())) > 0
}

fn selfspcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.check_release_group_cost(tp, |c, duel| cfilter(c, duel, tp), (1, 1), false, None, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RELEASE);
    let g = duel.select_release_group_cost(tp, |c, duel| cfilter(c, duel, tp), (1, 1), false, None, None)?;
    duel.release(g, REASON_COST)?;
    Ok(false)
}

fn selfsptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    Ok(false)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    let bc = a.e.get_handler(duel).get_battle_target(duel);
    bc.is_some_and(|bc| bc.is_status(duel, STATUS_BATTLE_DESTROYED) && bc.is_controler(duel, 1 - tp))
        || duel
            .check_event_full(EVENT_BATTLE_DAMAGE)
            .is_some_and(|event| event.event_player == 1 - tp)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    if !(c.is_set_card(duel, &[SET_SUPREME_KING_DRAGON])
        && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff))
    {
        return false;
    }
    if c.is_location(duel, LOCATION_EXTRA) {
        duel.get_location_count_from_ex(tp, tp, None, c) > 0
    } else {
        duel.get_mzone_count(tp, None) > 0
    }
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let location = LOCATION_DECK | LOCATION_EXTRA | LOCATION_GRAVE;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, location, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, location as i32);
    Ok(false)
}

fn exfilter1(c: CardId, duel: &Duel) -> bool {
    c.is_location(duel, LOCATION_EXTRA) && c.is_facedown(duel) && c.is_type(duel, TYPE_FUSION | TYPE_SYNCHRO | TYPE_XYZ)
}

fn exfilter2(c: CardId, duel: &Duel) -> bool {
    c.is_location(duel, LOCATION_EXTRA) && (c.is_type(duel, TYPE_LINK) || (c.is_faceup(duel) && c.is_type(duel, TYPE_PENDULUM)))
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let mut ft1 = duel.get_location_count(tp, LOCATION_MZONE);
    let mut ft2 = duel.get_location_count_from_ex(tp, tp, None, None);
    let mut ft3 = duel.get_location_count_from_ex(tp, tp, None, TYPE_FUSION + TYPE_SYNCHRO + TYPE_XYZ);
    let mut ft4 = duel.get_location_count_from_ex(tp, tp, None, TYPE_PENDULUM + TYPE_LINK);
    let mut ft = duel.get_usable_mzone_count(tp).min(2);
    if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
        for count in [&mut ft1, &mut ft2, &mut ft3, &mut ft4] {
            if *count > 0 {
                *count = 1;
            }
        }
        ft = 1;
    }
    if let Some(ect) = aux::check_summon_gate(duel, tp) {
        for count in [&mut ft1, &mut ft2, &mut ft3, &mut ft4] {
            *count = ect.min(*count);
        }
    }
    let mut location = 0;
    if ft1 > 0 {
        location |= LOCATION_DECK | LOCATION_GRAVE;
    }
    if ft2 > 0 || ft3 > 0 || ft4 > 0 {
        location |= LOCATION_EXTRA;
    }
    if location == 0 {
        return Ok(());
    }
    let sg = duel.get_matching_group(aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp)), tp, location, 0, None);
    if sg.is_empty() {
        return Ok(());
    }
    let rescon = |duel: &Duel, sg: CardSet| {
        let exnpct = sg.filter_count(duel, exfilter1, None) as i32;
        let expct = sg.filter_count(duel, exfilter2, None) as i32;
        let mct = sg.filter_count(duel, |c, duel| !c.is_location(duel, LOCATION_EXTRA), None) as i32;
        let res = ft3 >= exnpct && ft4 >= expct && ft1 >= mct;
        (res, !res)
    };
    let rg = aux::select_unselect_group(
        duel,
        sg,
        1,
        ft.max(0) as usize,
        &|duel, sg, _, _| rescon(duel, sg),
        tp,
        HINTMSG_SPSUMMON,
    )?;
    duel.special_summon(rg, 0, tp, tp, false, false, POS_FACEUP_DEFENSE)?;
    Ok(())
}

fn pencon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_previous_location(duel, LOCATION_MZONE) && c.is_faceup(duel)
}

fn pentg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    Ok(chk == 0 && duel.check_pendulum_zones(a.tp))
}

fn penop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if !duel.check_pendulum_zones(tp) {
        return Ok(());
    }
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) {
        duel.move_to_field(c, tp, tp, LOCATION_PZONE, POS_FACEUP, true)?;
    }
    Ok(())
}
