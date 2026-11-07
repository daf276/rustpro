// Number 38: Hope Harbinger Dragon Titanic Galaxy
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 63767246;
pub const XYZ_NUMBER: u32 = 38;

// In the Extra Deck, Xyz Summoned with Black Luster Soldier and Steam Knight - Hamburk, the opponent's Spell Cards to
// negate, and Number 39: Utopia to destroy with Raigeki and Dark Hole.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (10000100, MZONE),
    (23923758, MZONE),
    (84013237, MZONE),
    (84013237, MZONE),
    (5318639, SZONE | OPPONENT),
    (5318639, SZONE | OPPONENT),
    (12580477, HAND | OPPONENT),
    (53129443, HAND | OPPONENT),
    (12580477, DECK | OPPONENT),
];
// The destroyed Xyz Monster beside it is too rare to assert.
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 2))),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(8);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DISABLE);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_count_limit(duel, 1, 0);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, discon);
    e1.set_target(duel, distg);
    e1.set_operation(duel, disop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, |duel, a| {
        duel.get_attacker().is_some_and(|at| at.is_controler(duel, 1 - a.tp)) && duel.get_attack_target() != Some(a.e.get_handler(duel))
    });
    e2.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e2.set_operation(duel, chngtgop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_ATKCHANGE);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_DESTROYED);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_condition(duel, atkcon);
    e3.set_target(duel, atktg);
    e3.set_operation(duel, atkop);
    c.register_effect(duel, e3);
}

fn discon(duel: &Duel, a: &Args) -> bool {
    let loc: u32 = duel.get_chain_info(a.ev as u8, CHAININFO_TRIGGERING_LOCATION);
    loc & LOCATION_SZONE > 0 && a.re.is_some_and(|re| re.is_spell_effect(duel)) && duel.is_chain_disablable(a.ev as u8)
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_DISABLE, a.eg, 1, 0, 0);
    Ok(false)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp, c) = (a.e, a.tp, a.e.get_handler(duel));
    let Some(re) = a.re else { return Ok(()) };
    let rc = re.get_handler(duel);
    if duel.negate_effect(a.ev as u8)
        && c.is_relate_to_effect(duel, e)
        && rc.is_relate_to_effect(duel, re)
        && c.is_type(duel, TYPE_XYZ)
        && rc.is_can_be_xyz_material(duel, Some(c), tp, REASON_EFFECT)
    {
        rc.cancel_to_grave(duel);
        duel.overlay_full(c, rc, true)?;
    }
    Ok(())
}

fn chngtgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c) = (a.e, a.e.get_handler(duel));
    if !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    let Some(at) = duel.get_attacker() else { return Ok(()) };
    if at.can_attack(duel) && !at.is_immune_to_effect(duel, e) {
        duel.calculate_damage(at, Some(c))?;
    }
    Ok(())
}

fn atkconfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_reason(duel, REASON_BATTLE | REASON_EFFECT)
        && c.is_type(duel, TYPE_XYZ)
        && c.get_base_attack(duel) > 0
        && c.is_previous_position(duel, POS_FACEUP)
        && c.is_previous_location(duel, LOCATION_MZONE)
        && c.is_previous_controler(duel, tp)
}

fn atkcon(duel: &Duel, a: &Args) -> bool {
    a.eg.is_some_and(|eg| eg.is_exists(duel, |c, duel| atkconfilter(c, duel, a.tp), 1, None))
}

fn atktg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let xyz_filter = aux::faceup_filter(|c, duel| c.is_type(duel, TYPE_XYZ));
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE)
            && chkc.is_controler(duel, tp)
            && chkc.is_type(duel, TYPE_XYZ)
            && chkc.is_faceup(duel));
    }
    if chk == 0 {
        return Ok(atkcon(duel, a) && duel.is_existing_target(xyz_filter, tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATKDEF);
    duel.select_target(tp, xyz_filter, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    let mg = a.eg.unwrap_or_default().filter(duel, |c, duel| atkconfilter(c, duel, tp), None);
    a.e.set_label_object_group(duel, Some(mg));
    Ok(false)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !(tc.is_faceup(duel) && tc.is_relate_to_effect(duel, e)) {
        return Ok(());
    }
    let mut g = e.get_label_object_group(duel).unwrap_or_default();
    if g.len() >= 2 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
        g = g.select(duel, tp, 1, 1, None)?;
    }
    let e1 = duel.create_effect(e.get_handler(duel));
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_value(duel, g.get_first().map_or(0, |first| first.get_base_attack(duel)));
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
    tc.register_effect(duel, e1);
    Ok(())
}
