// Teardrop the Rikka Queen
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 33779875;

// In the Extra Deck, Xyz Summoned with Level 8 monsters (Black Luster Soldier, Anotherverse Stratios, Tytannial, a Plant), and monsters to Tribute.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (10000100, MZONE),
    (10949074, MZONE),
    (11819616, MZONE),
    (13140300, MZONE),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(8);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1a = duel.create_effect(c);
    e1a.set_description(duel, aux::stringid(ID, 0));
    e1a.set_category(duel, CATEGORY_RELEASE);
    e1a.set_type(duel, EFFECT_TYPE_IGNITION);
    e1a.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1a.set_range(duel, LOCATION_MZONE);
    e1a.set_count_limit(duel, 1, ID);
    e1a.set_condition(duel, |duel, a| !quickcon(duel, a));
    e1a.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1a.set_target(duel, tribtg);
    e1a.set_operation(duel, tribop);
    c.register_effect(duel, e1a);
    let e1b = e1a.clone_effect(duel);
    e1b.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1b.set_code(duel, EVENT_FREE_CHAIN);
    e1b.set_condition(duel, quickcon);
    e1b.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e1b);
    let e2a = duel.create_effect(c);
    e2a.set_description(duel, aux::stringid(ID, 1));
    e2a.set_category(duel, CATEGORY_ATKCHANGE);
    e2a.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e2a.set_code(duel, EVENT_CUSTOM + ID);
    e2a.set_range(duel, LOCATION_MZONE);
    e2a.set_operation(duel, atkop);
    c.register_effect(duel, e2a);
    let e2b = duel.create_effect(c);
    e2b.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2b.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e2b.set_code(duel, EVENT_RELEASE);
    e2b.set_range(duel, LOCATION_MZONE);
    e2b.set_operation(duel, regop);
    c.register_effect(duel, e2b);
}

fn tribtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_releasable_by_effect(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_releasable_by_effect, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RELEASE);
    let g = duel.select_target(tp, CardId::is_releasable_by_effect, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_RELEASE, g, 1, tp, 0);
    Ok(false)
}

fn tribop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.release(tc, REASON_EFFECT)?;
    }
    Ok(())
}

fn quickcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel)
        .get_overlay_group(duel)
        .is_exists(duel, |c, duel| c.is_race(duel, RACE_PLANT), 1, None)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) && c.is_faceup(duel) {
        c.update_attack(duel, a.ev as i32 * 200, Some(RESETS_STANDARD_DISABLE_PHASE_END), None, None);
    }
    Ok(())
}

fn atkconfilter(c: CardId, duel: &Duel) -> bool {
    c.is_previous_location(duel, LOCATION_MZONE) || (!c.is_previous_location(duel, LOCATION_ONFIELD) && c.is_monster(duel))
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    let ct = a.eg.unwrap_or_default().filter_count(duel, atkconfilter, None);
    if ct > 0 {
        let c = a.e.get_handler(duel);
        duel.raise_single_event(c, EVENT_CUSTOM + ID, a.re, a.r, a.rp, a.ep, ct as u32)?;
    }
    Ok(())
}
