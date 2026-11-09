// Patissciel Couverture
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 26435595;

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] =
        &[FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_type_full(duel, TYPE_PENDULUM, Some(fc), sumtype, tp)); 2];
}

// In the Extra Deck, Fusion Summoned by Polymerization from Junoldo and Jupredo, which the opponent's Dark Hole destroys into the
// Extra Deck for it to place, and which places itself in the Pendulum Zone when destroyed.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, LOCATION_PZONE as u16),
    (24094653, HAND),
    (10266279, MZONE),
    (43871165, MZONE),
    (17390179, MZONE),
    (53129443, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Pendulum::add_procedure_full(duel, c, false, None);
    Fusion::add_proc_mix_n::<Materials>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_PZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, pctg);
    e1.set_operation(duel, pcop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_DESTROYED);
    e2.set_condition(duel, |duel, a| {
        let c = a.e.get_handler(duel);
        c.is_faceup(duel) && c.is_previous_location(duel, LOCATION_MZONE)
    });
    e2.set_target(duel, |duel, a, chk, _| Ok(chk == 0 && duel.check_pendulum_zones(a.tp)));
    e2.set_operation(duel, penop);
    c.register_effect(duel, e2);
}

fn pcfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_type(duel, TYPE_PENDULUM) && !c.is_forbidden(duel)
}

fn pctg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    Ok(chk == 0 && duel.check_pendulum_zones(tp) && duel.is_existing_matching_card(pcfilter, tp, LOCATION_EXTRA, 0, 1, None))
}

fn pcop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if !a.e.get_handler(duel).is_relate_to_effect(duel, a.e) || !duel.check_pendulum_zones(tp) {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOFIELD);
    let g = duel.select_matching_card(tp, pcfilter, tp, LOCATION_EXTRA, 0, 1, 1, None)?;
    if let Some(tc) = g.get_first() {
        duel.move_to_field(tc, tp, tp, LOCATION_PZONE, POS_FACEUP, true)?;
    }
    Ok(())
}

fn penop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if duel.check_pendulum_zones(tp) && c.is_relate_to_effect(duel, a.e) {
        duel.move_to_field(c, tp, tp, LOCATION_PZONE, POS_FACEUP, true)?;
    }
    Ok(())
}
