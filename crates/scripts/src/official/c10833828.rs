// Forbidden Dark Contract with the Swamp King
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, SZONE};

const ID: u32 = 10833828;

pub const LISTED_SERIES: &[u16] = &[SET_DDD];

struct Params;

impl FusionSummonParams for Params {
    fn fusfilter(c: CardId, duel: &Duel) -> bool {
        c.is_race(duel, RACE_FIEND)
    }

    fn extrafil(duel: &Duel, e: EffectId, _: u8, _: CardSet) -> (Option<CardSet>, Option<FusionCheck>) {
        (None, Some((extrafil, e.get_handler(duel))))
    }
}

// Face-up, with D/D/D monsters in the hand to Special Summon and D/D monsters in the hand and on the field to fuse them with into
// D/D/D Alfred the Divine Sage King.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, SZONE),
    (83303851, HAND),
    (25857977, HAND),
    (45206713, HAND),
    (93317313, HAND),
    (72291412, MZONE),
    (45206713, MZONE),
    (11852093, EXTRA),
    (11852093, EXTRA),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_FUSION_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_SZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_target(duel, Fusion::summon_eff_tg::<Params>);
    e3.set_operation(duel, Fusion::summon_eff_op::<Params>);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 2));
    e4.set_category(duel, CATEGORY_DAMAGE);
    e4.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e4.set_code(duel, EVENT_PHASE | PHASE_STANDBY);
    e4.set_range(duel, LOCATION_SZONE);
    e4.set_count_limit(duel, 1, 0);
    e4.set_condition(duel, |duel, a| duel.is_turn_player(a.tp));
    e4.set_target(duel, super::c46372010::damtg::<2000>);
    e4.set_operation(duel, super::c46372010::damop);
    c.register_effect(duel, e4);
}

// s.extrafil(g) applied, with g the handler's card targets.
fn extrafil(duel: &Duel, c: CardId, _: u8, sg: CardSet, _: CardId) -> bool {
    !c.get_card_target(duel).intersection(sg).is_empty()
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_DDD]) && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_HAND, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_HAND as i32);
    duel.hint(HINT_OPSELECTED, 1 - tp, e.get_description(duel));
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) || duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_HAND, 0, 1, 1, None)?;
    if let Some(tc) = g.get_first()
        && duel.special_summon_step(tc, 0, tp, tp, false, false, POS_FACEUP_DEFENSE)?
    {
        for code in [EFFECT_DISABLE, EFFECT_DISABLE_EFFECT] {
            let e1 = duel.create_effect(c);
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_code(duel, code);
            e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
            tc.register_effect_full(duel, e1, true);
        }
        c.set_card_target(duel, tc);
    }
    duel.special_summon_complete()?;
    Ok(())
}
