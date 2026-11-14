// Mitsurugi Magatama
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 76725398;

pub const LISTED_SERIES: &[u16] = &[SET_MITSURUGI];
// Mitsurugi Ritual Monsters in the hand and Reptiles on the field to release, and face-up opponent's monsters to destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (19899073, HAND),
    (55397172, HAND),
    (18176525, MZONE),
    (40543231, MZONE),
    (82782870, MZONE),
    (26202165, MZONE | OPPONENT),
    (53129443, SZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::SpecialSummonedCode(19899073)];

struct RitualEffect;

impl RitualParams for RitualEffect {
    fn filter(c: CardId, duel: &Duel) -> bool {
        c.is_set_card(duel, &[SET_MITSURUGI])
    }

    fn matfilter(c: CardId, duel: &Duel, _: EffectId, _: u8) -> bool {
        c.is_location(duel, LOCATION_MZONE)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER | TIMING_MAIN_END);
    e1.set_condition(duel, |duel, _| duel.is_main_phase());
    e1.set_target(duel, efftg);
    e1.set_operation(duel, effop);
    c.register_effect(duel, e1);
}

fn rescon(duel: &Duel, sg: CardSet, tp: u8, _: CardSet) -> (bool, bool) {
    let equips = sg.get_first().map(|c| c.get_equip_group(duel)).unwrap_or_default();
    (
        duel.is_existing_target(CardId::is_faceup, tp, 0, LOCATION_ONFIELD, 1, Some(sg.union(equips))),
        false,
    )
}

fn reptile(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_REPTILE)
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_controler(duel, 1 - tp) && chkc.is_faceup(duel));
    }
    let b1 = duel.check_release_group_cost(tp, reptile, (1, 1), false, Some(&rescon), None);
    let b2 = Ritual::target::<RitualEffect, RITPROC_GREATER>(duel, a, 0, None)?;
    if chk == 0 {
        return Ok(b1 || b2);
    }
    let op = duel.select_effect(tp, &[(b1, aux::stringid(ID, 1)), (b2, aux::stringid(ID, 2))])?;
    e.set_label(duel, op.map_or(0, |op| op as i64));
    match op {
        Some(1) => {
            e.set_category(duel, CATEGORY_DESTROY);
            e.set_property(duel, EFFECT_FLAG_CARD_TARGET);
            let rg = duel.select_release_group_cost(tp, reptile, (1, 1), false, Some(&rescon), None)?;
            duel.release(rg, REASON_COST)?;
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
            let tg = duel.select_target(tp, CardId::is_faceup, tp, 0, LOCATION_ONFIELD, 1, 1, None)?;
            duel.set_operation_info(0, CATEGORY_DESTROY, tg, 1, tp, 0);
        }
        Some(2) => {
            e.set_category(duel, CATEGORY_RELEASE + CATEGORY_SPECIAL_SUMMON);
            e.set_property(duel, 0);
            duel.set_operation_info(0, CATEGORY_RELEASE, None, 1, tp, LOCATION_MZONE as i32);
            duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_HAND as i32);
        }
        _ => {}
    }
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    match a.e.get_label(duel) {
        1 => stock::destroy_first_target(duel, a),
        2 => Ritual::operation::<RitualEffect, RITPROC_GREATER>(duel, a),
        _ => Ok(()),
    }
}
