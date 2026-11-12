// Marincess Great Bubble Reef
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, REMOVED};

const ID: u32 = 47910940;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_attribute_full(duel, ATTRIBUTE_WATER, Some(lc), sumtype, tp)
    }
}

// Link Summoned from WATER Normal Monsters (Psychic Kappa, Slime Toad), with a WATER monster in hand and the Graveyard to pay and Marincess Sea Star banished.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (7892180, MZONE),
    (7892180, MZONE),
    (68638985, MZONE),
    (68638985, MZONE),
    (7892180, HAND),
    (68638985, GRAVE),
    (62886670, REMOVED),
    (62886670, REMOVED),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DRAW);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_code(duel, EVENT_PHASE + PHASE_STANDBY);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_cost(duel, drcost);
    e1.set_target(duel, stock::draw_target::<1>);
    e1.set_operation(duel, stock::draw);
    c.register_effect(duel, e1);
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
    e2b.set_code(duel, EVENT_REMOVE);
    e2b.set_range(duel, LOCATION_MZONE);
    e2b.set_operation(duel, regop);
    c.register_effect(duel, e2b);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, ID);
    e3.set_cost(duel, spcost);
    e3.set_target(duel, sptg);
    e3.set_operation(duel, spop);
    c.register_effect(duel, e3);
}

fn drcostfilter(c: CardId, duel: &Duel) -> bool {
    c.is_attribute(duel, ATTRIBUTE_WATER) && c.is_faceup(duel) && c.is_able_to_remove_as_cost(duel)
}

fn drcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    stock::banish_cost(duel, a, chk, drcostfilter, LOCATION_MZONE | LOCATION_GRAVE)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) && c.is_faceup(duel) {
        c.update_attack(duel, a.ev as i32 * 600, Some(RESETS_STANDARD_DISABLE_PHASE_END), None, None);
    }
    Ok(())
}

fn atkconfilter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel)
        && c.is_faceup(duel)
        && (c.is_previous_location(duel, LOCATION_MZONE) || !c.is_previous_location(duel, LOCATION_ONFIELD))
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    let ct = a.eg.map_or(0, |eg| eg.filter_count(duel, atkconfilter, None));
    if ct > 0 {
        duel.raise_single_event(a.e.get_handler(duel), EVENT_CUSTOM + ID, a.re, a.r, a.rp, a.ep, ct as u32)?;
    }
    Ok(())
}

fn spcostfilter(c: CardId, duel: &Duel) -> bool {
    c.is_attribute(duel, ATTRIBUTE_WATER) && c.is_able_to_grave_as_cost(duel)
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    stock::send_to_grave_cost(duel, a, chk, spcostfilter, LOCATION_HAND)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_MARINCESS]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false) && c.is_faceup(duel)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_REMOVED)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::special_summon(duel, a, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_REMOVED, POS_FACEUP)
}
