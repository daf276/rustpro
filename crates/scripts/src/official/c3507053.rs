// Appliancer Dryer Drake
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 3507053;

// Link Summoned from an Appliancer monster, with Appliancer Link monsters to co-link with and opponent's attackers.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (78225596, EXTRA),
    (78225596, EXTRA),
    (78447174, MZONE),
    (78447174, MZONE),
    (78447174, MZONE),
    (4729591, MZONE),
    (81769387, MZONE),
    (78447174, HAND),
    (78447174, DECK),
    (30118200, EXTRA),
    (13597785, MZONE | OPPONENT),
    (89631139, MZONE | OPPONENT),
];
// A co-link needs Celtopus Link Summoned into the Extra Monster Zone with another Appliancer Link Monster in the zone it points to; random play rarely gets it.
pub const CHECKS: &[Check] = &[
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 0))),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 1;

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_set_card_full(duel, &[SET_APPLIANCER], Some(lc), sumtype, tp)
    }
}

// The "cannot be Link Material" effect the Link-1 Appliancer monsters share.
pub fn register_cannot_be_link_material(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EFFECT_CANNOT_BE_LINK_MATERIAL);
    e1.set_condition(duel, |duel, a| {
        let c = a.e.get_handler(duel);
        c.is_status(duel, STATUS_SPSUMMON_TURN) && c.is_link_summoned(duel)
    });
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
}

pub fn colinkcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).get_mutual_linked_group_count(duel) > 0
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    register_cannot_be_link_material(duel, c);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).is_linked(duel));
    e2.set_value(duel, 1000);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_code(duel, EVENT_FREE_CHAIN);
    e3.set_range(duel, LOCATION_MMZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_condition(duel, |duel, a| mvcon(duel, a) && colinkcon(duel, a));
    e3.set_target(duel, mvtg);
    e3.set_operation(duel, mvop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e4.set_code(duel, EVENT_BE_BATTLE_TARGET);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_count_limit(duel, 1, 0);
    e4.set_condition(duel, |duel, a| !colinkcon(duel, a));
    e4.set_operation(duel, |duel, _| duel.negate_attack().map(|_| ()));
    c.register_effect(duel, e4);
}

fn mvcon(duel: &Duel, a: &Args) -> bool {
    duel.is_turn_player(a.tp) && duel.is_battle_phase()
}

fn appliancer_faceup(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_APPLIANCER])
}

fn mvtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(appliancer_faceup, a.tp, LOCATION_MMZONE, 0, 1, Some(a.e.get_handler(duel).into())));
    }
    Ok(false)
}

fn mvop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    let g = duel.select_matching_card(tp, appliancer_faceup, tp, LOCATION_MMZONE, 0, 1, 1, Some(c.into()))?;
    if let Some(tc) = g.get_first() {
        duel.swap_sequence(c, tc);
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_code(duel, EFFECT_EXTRA_ATTACK);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        e1.set_value(duel, 1);
        tc.register_effect(duel, e1);
    }
    Ok(())
}
