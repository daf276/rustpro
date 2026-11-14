// Grapha, Dragon Lord of Dark World
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 34230233;

pub const LISTED_SERIES: &[u16] = &[SET_DARK_WORLD];
pub const LISTED_NAMES: &[u32] = &[ID];

// In the Graveyard to return Renge from the field, in the hand to be discarded by Dark World Dealings, and the opponent's
// Dealings to discard it from their side.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, GRAVE),
    (ID, HAND),
    (60606759, MZONE),
    (74117290, HAND),
    (74117290, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e1.set_range(duel, LOCATION_GRAVE);
    e1.set_condition(duel, spcon);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_condition_mut(duel, descon);
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
}

// Shared by the Dark World monsters that return one from the field to Special Summon themselves from the Graveyard.
pub fn spcon_with(duel: &Duel, a: &Args, filter: fn(CardId, &Duel) -> bool) -> bool {
    let Some(c) = a.c else { return true };
    let rg = duel.get_matching_group(filter, a.e.get_handler_player(duel), LOCATION_MZONE, 0, None);
    for te in c.get_card_effect(duel, EFFECT_NECRO_VALLEY).iter() {
        if te.get_operation(duel).is_none() {
            return false;
        }
        unported!(duel, true);
    }
    let tp = c.get_controler(duel);
    aux::select_unselect_group_check(duel, rg, 1, 1, &|duel, sg, _, _| chkf_mmz(duel, tp, sg))
}

pub fn sptg_with(duel: &mut Duel, a: &Args, filter: fn(CardId, &Duel) -> bool) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let rg = duel.get_matching_group(filter, tp, LOCATION_MZONE, 0, None);
    let g = aux::select_unselect_group_full(
        duel,
        rg,
        1,
        1,
        &|duel, sg, _, _| chkf_mmz(duel, tp, sg),
        tp,
        HINTMSG_RTOHAND,
        None,
        true,
    )?;
    if g.is_empty() {
        return Ok(false);
    }
    e.set_label_object_group(duel, Some(g));
    Ok(true)
}

pub fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(g) = a.e.get_label_object_group(duel) else { return Ok(()) };
    duel.send_to_hand(g, PLAYER_NONE, REASON_COST)?;
    Ok(())
}

fn chkf_mmz(duel: &Duel, tp: u8, sg: CardSet) -> (bool, bool) {
    (duel.get_mzone_count(tp, Some(sg)) >= 1, false)
}

// Shared by the Dark World monsters that react to being discarded by an effect.
pub fn descon(duel: &mut Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    let previous_controler = c.get_previous_controler(duel);
    a.e.set_label(duel, i64::from(previous_controler));
    c.is_previous_location(duel, LOCATION_HAND) && a.r & (REASON_DISCARD | REASON_EFFECT) == REASON_DISCARD | REASON_EFFECT
}

fn spfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_DARK_WORLD]) && !c.is_code(duel, ID) && c.is_able_to_hand_as_cost(duel)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    spcon_with(duel, a, spfilter)
}

fn sptg(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
    sptg_with(duel, a, spfilter)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_controler(duel, 1 - tp));
    }
    if chk == 0 {
        return Ok(true);
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, aux::TRUE, tp, 0, LOCATION_ONFIELD, 1, 1, None)?;
    let count = g.map_or(0, |g| g.len() as u8);
    duel.set_operation_info(0, CATEGORY_DESTROY, g, count, 0, 0);
    if a.rp == tp {
        e.set_category(duel, CATEGORY_DESTROY);
    } else {
        e.set_category(duel, CATEGORY_DESTROY + CATEGORY_SPECIAL_SUMMON);
        duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_HAND as i32);
    }
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, e) && duel.destroy(tc, REASON_EFFECT)? != 0 && a.rp == 1 - tp && i64::from(tp) == e.get_label(duel) {
        duel.break_effect()?;
        let hg = duel.get_field_group(tp, 0, LOCATION_HAND);
        if !hg.is_empty() {
            let cg = hg.random_select(duel, tp, 1);
            let Some(cc) = cg.get_first() else { return Ok(()) };
            duel.confirm_cards(tp, cc)?;
            if duel.get_location_count(tp, LOCATION_MZONE) > 0
                && cc.is_can_be_special_summoned(duel, e, 0, tp, false, false)
                && duel.select_yes_no(tp, aux::stringid(ID, 1))?
            {
                duel.special_summon(cc, 0, tp, tp, false, false, POS_FACEUP)?;
            } else {
                duel.shuffle_hand(1 - tp)?;
            }
        }
    }
    Ok(())
}
