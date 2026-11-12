// Number 39: Utopia Beyond
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE, OPPONENT};

const ID: u32 = 21521304;
pub const XYZ_NUMBER: u32 = 39;

// In the Extra Deck, Xyz Summoned with two Level 6 monsters, with an opponent's monster to zero and Number 39: Utopia in the
// Graveyard to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (6631034, MZONE),
    (8353769, MZONE),
    (9540040, MZONE),
    (26202165, MZONE | OPPONENT),
    (84013237, GRAVE),
    (84013237, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(6);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.add_setcodes_rule(duel, ID, true, &[SET_UTOPIA]);
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_ATKCHANGE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_xyz_summoned(duel));
    e1.set_operation(duel, atkop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_REMOVE + CATEGORY_SPECIAL_SUMMON + CATEGORY_RECOVER);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_count_limit(duel, 1, 0);
    e2.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (tp, c) = (a.tp, a.e.get_handler(duel));
    for tc in duel.get_matching_group(CardId::is_faceup, tp, 0, LOCATION_MZONE, None) {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_SET_ATTACK_FINAL);
        e1.set_value(duel, 0);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        tc.register_effect(duel, e1);
    }
    Ok(())
}

fn rmfilter(c: CardId, duel: &Duel, ft: i32) -> bool {
    c.is_faceup(duel) && c.is_type(duel, TYPE_XYZ) && c.is_able_to_remove(duel) && (ft > 0 || c.get_sequence(duel) < 5)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_UTOPIA]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chkc.is_some() {
        return Ok(false);
    }
    let ft = duel.get_location_count(tp, LOCATION_MZONE);
    if chk == 0 {
        return Ok(ft > -1
            && duel.is_existing_target(|c, duel| rmfilter(c, duel, ft), tp, LOCATION_MZONE, 0, 1, None)
            && duel.is_existing_target(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g1 = duel.select_target(tp, |c, duel| rmfilter(c, duel, ft), tp, LOCATION_MZONE, 0, 1, 1, None)?;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g2 = duel.select_target(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_REMOVE, g1, 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g2, 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_RECOVER, None, 0, tp, 1250);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let first = |duel: &Duel, category| {
        duel.get_operation_info(0, category)
            .and_then(|info| info.op_cards)
            .and_then(|g| g.get_first())
    };
    let (tc1, tc2) = (first(duel, CATEGORY_REMOVE), first(duel, CATEGORY_SPECIAL_SUMMON));
    unported!(duel, tc1.is_none() || tc2.is_none());
    let (Some(tc1), Some(tc2)) = (tc1, tc2) else { return Ok(()) };
    if !tc1.is_relate_to_effect(duel, e) || duel.remove(tc1, POS_FACEUP, REASON_EFFECT)? == 0 {
        return Ok(());
    }
    if !tc2.is_relate_to_effect(duel, e) || duel.special_summon(tc2, 0, tp, tp, false, false, POS_FACEUP)? == 0 {
        return Ok(());
    }
    duel.break_effect()?;
    duel.recover(tp, 1250, REASON_EFFECT)?;
    Ok(())
}
