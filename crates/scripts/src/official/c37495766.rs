// Rescue-ACE Turbulence
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 37495766;

pub const LISTED_SERIES: &[u16] = &[SET_RESCUE_ACE];

const RESCUE_ACE_IMPULSE: u32 = 38339996;
const EXTINGUISH: u32 = 99162522;
const CONTAIN: u32 = 62777823;
const RESCUE: u32 = 99984170;

// In the hand and on the field, with Rescue-ACE Impulse in the GY and on the field to banish, Rescue-ACE Spells and Traps
// in the Deck to Set, and the opponent's Dark Hole to destroy it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (RESCUE_ACE_IMPULSE, GRAVE),
    (RESCUE_ACE_IMPULSE, GRAVE),
    (RESCUE_ACE_IMPULSE, MZONE),
    (EXTINGUISH, DECK),
    (CONTAIN, DECK),
    (RESCUE, DECK),
    (53129443, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_cost(duel, spcost);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SET);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, settg);
    e2.set_operation(duel, setop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_DESTROY);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e3.set_code(duel, EVENT_LEAVE_FIELD);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, (ID, 2));
    e3.set_condition(duel, descon);
    e3.set_target(duel, destg);
    e3.set_operation(duel, desop);
    c.register_effect(duel, e3);
}

fn spcostfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_RESCUE_ACE]) && c.is_able_to_remove_as_cost(duel) && aux::sp_elim_filter(c, duel, true, false)
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    let rg = duel.get_matching_group(spcostfilter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, None);
    if chk == 0 {
        return Ok(rg.len() > 1 && aux::select_unselect_group_check(duel, rg, 2, 2, &aux::chkf_mmz(tp, 1)));
    }
    let g = aux::select_unselect_group(duel, rg, 2, 2, &aux::chkf_mmz(tp, 1), tp, HINTMSG_REMOVE)?;
    duel.remove(g, POS_FACEUP, REASON_COST)?;
    Ok(false)
}

pub fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    Ok(false)
}

fn setfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_RESCUE_ACE]) && (c.is_quick_play_spell(duel) || c.is_normal_trap(duel)) && c.is_ssetable(duel)
}

fn settg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    Ok(chk == 0 && duel.is_existing_matching_card(setfilter, a.tp, LOCATION_DECK, 0, 1, None))
}

fn setop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_matching_group(setfilter, tp, LOCATION_DECK, 0, None);
    if g.is_empty() {
        return Ok(());
    }
    let ft = duel.get_location_count(tp, LOCATION_SZONE).min(4);
    let sg = aux::select_unselect_group(duel, g, 1, ft as usize, &aux::dncheck, tp, HINTMSG_SET)?;
    if !sg.is_empty() {
        duel.sset(tp, sg)?;
    }
    Ok(())
}

fn desconfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.get_reason_player(duel) == 1 - tp && c.is_reason(duel, REASON_EFFECT) && c.is_previous_controler(duel, tp)
}

fn descon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    a.eg.is_some_and(|eg| eg.is_exists(duel, |c, duel| desconfilter(c, duel, tp), 1, Some(c.into())))
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, tp, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, a.e) {
        duel.destroy(tc, REASON_EFFECT)?;
    }
    Ok(())
}
