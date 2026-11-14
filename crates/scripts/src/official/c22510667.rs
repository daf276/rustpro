// Gouki The Solid Ogre
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE};

const ID: u32 = 22510667;

pub const LISTED_SERIES: &[u16] = &[SET_GOUKI];
pub const LISTED_NAMES: &[u32] = &[ID];

// A MAX of 0 is Link.AddProcedure's nil maximum.
pub(super) struct Procedure<const MIN: usize, const MAX: usize>;

impl<const MIN: usize, const MAX: usize> LinkProcedure for Procedure<MIN, MAX> {
    const MIN: usize = MIN;
    const MAX: Option<usize> = if MAX == 0 { None } else { Some(MAX) };

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_set_card_full(duel, &[SET_GOUKI], Some(lc), sumtype, tp)
    }
}

// Link Summoned from Gouki Guts and Gouki Machine Suprex, which it then moves.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (7540107, MZONE),
    (29099860, MZONE),
    (7540107, MZONE),
    (7540107, HAND),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Link::add_procedure::<Procedure<2, 0>>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e1.set_condition(duel, |duel, a| incon(duel, a.e));
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_code(duel, EVENT_FREE_CHAIN);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_target(duel, seqtg);
    e3.set_operation(duel, seqop);
    c.register_effect(duel, e3);
}

fn incon(duel: &Duel, e: EffectId) -> bool {
    let c = e.get_handler(duel);
    c.get_linked_group_count(duel) > 0
        && c.get_linked_group(duel)
            .is_exists(duel, aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_GOUKI])), 1, None)
}

fn seqfilter(c: CardId, duel: &Duel, zone: u32) -> bool {
    let controler = c.get_controler(duel);
    c.is_faceup(duel)
        && c.is_set_card(duel, &[SET_GOUKI])
        && !c.is_code(duel, ID)
        && c.get_sequence(duel) < 5
        && duel.get_location_count_full(controler, LOCATION_MZONE, controler, LOCATION_REASON_CONTROL, zone) > 0
}

fn seqtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let zone = a.e.get_handler(duel).get_linked_zone_full(duel, tp) & ZONES_MMZ;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && seqfilter(chkc, duel, zone));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(|c, duel| seqfilter(c, duel, zone), tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    duel.select_target(tp, |c, duel| seqfilter(c, duel, zone), tp, LOCATION_MZONE, 0, 1, 1, None)?;
    Ok(false)
}

fn seqop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let zone = c.get_linked_zone_full(duel, tp) & ZONES_MMZ;
    let Some(tc) = duel.get_first_target() else {
        return Ok(());
    };
    let controler = tc.get_controler(duel);
    if !tc.is_relate_to_effect(duel, e)
        || tc.is_controler(duel, 1 - tp)
        || tc.is_immune_to_effect(duel, e)
        || duel.get_location_count_full(controler, LOCATION_MZONE, controler, LOCATION_REASON_CONTROL, zone) <= 0
    {
        return Ok(());
    }
    let shift = if c.is_controler(duel, tp) { 0 } else { 16 };
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOZONE);
    if let Some(field) = duel.select_disable_field(tp, 1, LOCATION_MZONE as u16, LOCATION_MZONE as u16, !(zone << shift))? {
        duel.move_sequence(tc, (field.trailing_zeros() - shift) as u8)?;
    }
    Ok(())
}
