// Epurrely Plump
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 24434049;

pub const LISTED_NAMES: &[u32] = &[55584558];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(2);
    const CT: usize = 2;
}

// In the Extra Deck, Xyz Summoned with two Level 2 monsters, with Spells in the Graveyard to attach and Purrely Quick-Play Spells in the
// hand to attach as they are activated.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (756652, MZONE),
    (2830619, MZONE),
    (12482652, MZONE),
    (10598400, MZONE),
    (55584558, HAND),
    (55584558, HAND),
    (82105704, HAND),
    (83764718, GRAVE),
    (55584558, GRAVE),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_LEAVE_GRAVE);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit_full(duel, 1, 0, EFFECT_COUNT_CODE_SINGLE);
    e1.set_condition(duel, gyovcon);
    e1.set_target(duel, gyovtg);
    e1.set_operation(duel, gyovop);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_hint_timing(duel, 0, TIMING_END_PHASE);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_REMOVE);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_code(duel, EVENT_CHAINING);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 3, 0);
    e3.set_condition(duel, qpovcon);
    e3.set_target(duel, qpovtg);
    e3.set_operation(duel, qpovop);
    c.register_effect(duel, e3);
}

fn gyovcon(duel: &Duel, a: &Args) -> bool {
    let ov = a.e.get_handler(duel).get_overlay_group(duel);
    a.e.is_has_type(duel, EFFECT_TYPE_QUICK_O) == ov.is_exists(duel, |c, duel| c.is_code(duel, 55584558), 1, None)
}

fn gyovfilter(c: CardId, duel: &Duel, xc: CardId, tp: u8) -> bool {
    c.is_spell_trap(duel) && c.is_can_be_xyz_material(duel, Some(xc), tp, REASON_EFFECT)
}

fn gyovtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && gyovfilter(chkc, duel, c, tp));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(|c2, duel| gyovfilter(c2, duel, c, tp), tp, LOCATION_GRAVE, LOCATION_GRAVE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_XMATERIAL);
    let g = duel.select_target(
        tp,
        |c2, duel| gyovfilter(c2, duel, c, tp),
        tp,
        LOCATION_GRAVE,
        LOCATION_GRAVE,
        1,
        2,
        None,
    )?;
    unported!(duel, g.is_none());
    duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, g, g.map_or(0, |g| g.len() as u8), 0, 0);
    Ok(false)
}

fn gyovop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) || c.is_immune_to_effect(duel, e) {
        return Ok(());
    }
    let g = duel.get_target_cards(e).unwrap_or_default().filter(
        duel,
        |tc, duel| gyovfilter(tc, duel, c, tp) && !tc.is_immune_to_effect(duel, e),
        None,
    );
    if !g.is_empty() {
        duel.overlay(c, g)?;
    }
    Ok(())
}

pub fn qpovcon(duel: &Duel, a: &Args) -> bool {
    let Some(re) = a.re else { return false };
    if a.rp == 1 - a.tp || !re.is_has_type(duel, EFFECT_TYPE_ACTIVATE) {
        return false;
    }
    let rc = re.get_handler(duel);
    rc.is_set_card(duel, &[SET_PURRELY])
        && rc.is_quick_play_spell(duel)
        && rc.is_on_field(duel)
        && rc.is_can_be_xyz_material(duel, Some(a.e.get_handler(duel)), a.tp, REASON_EFFECT)
}

fn qpovtg(duel: &mut Duel, _: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_possible_operation_info(0, CATEGORY_REMOVE, None, 1, PLAYER_ALL, LOCATION_MZONE as i32);
    Ok(false)
}

fn qpovop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let Some(re) = a.re else { return Ok(()) };
    let rc = re.get_handler(duel);
    if c.is_relate_to_effect(duel, e)
        && rc.is_relate_to_effect(duel, re)
        && !c.is_immune_to_effect(duel, e)
        && !rc.is_immune_to_effect(duel, e)
        && rc.is_can_be_xyz_material(duel, Some(c), tp, REASON_EFFECT)
    {
        duel.overlay(c, rc)?;
        if !c.get_overlay_group(duel).contains(rc) {
            return Ok(());
        }
        rc.cancel_to_grave(duel);
        if duel.is_existing_matching_card(CardId::is_able_to_remove, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None)
            && duel.select_yes_no(tp, aux::stringid(ID, 2))?
        {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
            let g = duel.select_matching_card(tp, CardId::is_able_to_remove, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
            if g.is_empty() {
                return Ok(());
            }
            duel.hint_selection_full(g, true);
            duel.break_effect()?;
            aux::remove_until::<aux::DefaultFieldReturnOp>(duel, g, 0, REASON_EFFECT, PHASE_END, ID, e, tp)?;
        }
    }
    Ok(())
}
