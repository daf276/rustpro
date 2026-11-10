// Worldsea Dragon Zealantis
use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT};

const ID: u32 = 45112597;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 1;

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_type_full(duel, TYPE_EFFECT, Some(lc), sumtype, tp)
    }
}

// S:P Little Knight (Link markers left and right, like its own) to be co-linked, Sangans to Link Summon it and the opponent's Sangans
// to banish; no copy, since a second one on the field needs the unported unique-card self destruction.
pub const SUPPORT: &[(u32, u16)] = &[
    (29301450, MZONE),
    (26202165, MZONE),
    (26202165, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    c.set_unique_on_field(duel, 1, 0, |c, duel| c.is_code(duel, ID), LOCATION_ONFIELD);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_REMOVE + CATEGORY_SPECIAL_SUMMON + CATEGORY_SET);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, rmsptg);
    e1.set_operation(duel, rmspop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, _| duel.is_battle_phase());
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
}

fn rmsptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_able_to_remove, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_REMOVE, None, 1, tp, LOCATION_MZONE as i32);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, PLAYER_EITHER, LOCATION_REMOVED as i32);
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    let owner = c.get_owner(duel);
    c.is_faceup(duel)
        && c.is_location(duel, LOCATION_REMOVED)
        && !c.is_reason(duel, REASON_REDIRECT)
        && (c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP, owner, 0xff)
            || c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEDOWN_DEFENSE, owner, 0xff))
}

fn rmspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let g = duel.get_matching_group(CardId::is_able_to_remove, tp, LOCATION_MZONE, LOCATION_MZONE, None);
    if g.is_empty() || duel.remove(g, POS_FACEUP, REASON_EFFECT)? == 0 {
        return Ok(());
    }
    let og = duel.get_operated_group();
    let sg = og.filter(duel, |c, duel| spfilter(c, duel, e, tp), None);
    if sg.is_empty() {
        return Ok(());
    }
    let (mut your_sg, mut opp_sg) = sg.split(duel, |c, duel| c.is_owner(duel, tp), None);
    let (your_ft, opp_ft) = (
        duel.get_location_count(tp, LOCATION_MZONE),
        duel.get_location_count(1 - tp, LOCATION_MZONE),
    );
    if your_sg.len() as i32 > your_ft {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        your_sg = your_sg.select(duel, tp, your_ft as u8, your_ft as u8, None)?;
    }
    if opp_sg.len() as i32 > opp_ft {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        opp_sg = opp_sg.select(duel, tp, opp_ft as u8, opp_ft as u8, None)?;
    }
    let sg = your_sg.union(opp_sg);
    for sc in sg {
        let owner = sc.get_owner(duel);
        let mut sump = 0;
        if sc.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP, owner, 0xff) {
            sump |= POS_FACEUP;
        }
        if sc.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEDOWN_DEFENSE, owner, 0xff) {
            sump |= POS_FACEDOWN_DEFENSE;
        }
        duel.special_summon_step(sc, 0, tp, owner, false, false, sump)?;
    }
    let fdg = sg.filter(duel, CardId::is_facedown, None);
    if !fdg.is_empty() {
        duel.confirm_cards(1 - tp, fdg)?;
    }
    duel.break_effect()?;
    duel.special_summon_complete()?;
    Ok(())
}

fn desctfilter(c: CardId, duel: &Duel) -> bool {
    c.get_mutual_linked_group_count(duel) > 0
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let dg = duel.get_field_group(tp, LOCATION_ONFIELD, LOCATION_ONFIELD);
    if chk == 0 {
        return Ok(!dg.is_empty() && duel.is_existing_matching_card(desctfilter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(dg), 1, tp, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let ct = duel.get_matching_group_count(desctfilter, tp, LOCATION_MZONE, LOCATION_MZONE, None);
    if ct == 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_matching_card(tp, aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, ct as u8, None)?;
    if !g.is_empty() {
        duel.hint_selection(g);
        duel.destroy(g, REASON_EFFECT)?;
    }
    Ok(())
}
