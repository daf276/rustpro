// Gimmick Puppet Fiendish Knight
use super::c63825486::{register_self_to_hand, thcost};
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 4145915;

pub const LISTED_SERIES: &[u16] = &[SET_GIMMICK_PUPPET];

// In the hand with Scissor Arms in the Graveyard to summon, and on the field for the opponent's Raigeki.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, MZONE),
    (29216967, GRAVE),
    (29216967, GRAVE),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, thcost::<ID>);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    register_self_to_hand::<ID>(duel, c);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    (c.is_set_card(duel, &[SET_GIMMICK_PUPPET]) || c.is_controler(duel, 1 - tp))
        && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, c.get_owner(duel), 0xff)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let filter = |tc, duel: &Duel| spfilter(tc, duel, e, tp);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) >= 2
            && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
            && duel.is_existing_target(filter, tp, LOCATION_GRAVE, LOCATION_GRAVE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let mut g = duel
        .select_target(tp, filter, tp, LOCATION_GRAVE, LOCATION_GRAVE, 1, 1, None)?
        .unwrap_or_default();
    g.insert(c);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(g), 2, tp, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, e) && duel.special_summon_step(tc, 0, tp, tc.get_owner(duel), false, false, POS_FACEUP_DEFENSE)? {
        tc.negate_effects(duel, c, 0, false);
        if duel.special_summon_complete()? == 0 || !c.is_relate_to_effect(duel, e) {
            return Ok(());
        }
        duel.break_effect()?;
        duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
