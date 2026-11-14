// Number C32: Shark Drake LeVeiss
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE};

const ID: u32 = 7628844;

pub const LISTED_SERIES: &[u16] = &[SET_SHARK_DRAKE];
pub const XYZ_NUMBER: u32 = 32;

// In the Extra Deck, Xyz Summoned with four Level 5 monsters, or over Number C32: Shark Drake Veiss by discarding a Spell.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (33178416, MZONE),
    (43793530, MZONE),
    (55998462, MZONE),
    (74637266, MZONE),
    (49221191, MZONE),
    (12580477, HAND),
    (53129443, HAND),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 1))];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(5);
    const CT: usize = 4;
    const ALTERF: Option<AlterFilter> = Some(|c, duel, tp, xyzc| {
        c.is_rank(duel, &[4])
            && c.is_set_card_full(duel, &[SET_SHARK_DRAKE], Some(xyzc), u64::from(SUMMON_TYPE_XYZ), tp)
            && c.is_type_full(duel, TYPE_XYZ, Some(xyzc), u64::from(SUMMON_TYPE_XYZ), tp)
            && c.is_faceup(duel)
    });
    const DESC: u64 = aux::stringid(ID, 0);

    fn op_check(duel: &Duel, _: EffectId, tp: u8, _: CardId) -> bool {
        duel.is_existing_matching_card(altproccostfilter, tp, LOCATION_HAND, 0, 1, None)
    }

    fn op(duel: &mut Duel, _: EffectId, tp: u8, _: CardId) -> Script<bool> {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_DISCARD);
        let g = duel.get_matching_group(altproccostfilter, tp, LOCATION_HAND, 0, None);
        let cancelable = Xyz::proc_cancellable(duel);
        let Some(sc) = g.select_unselect(duel, Some(CardSet::default()), tp, false, cancelable, 1, 1)? else {
            return Ok(false);
        };
        Ok(duel.send_to_grave(sc, REASON_DISCARD | REASON_COST)? > 0)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_category(duel, CATEGORY_DISABLE + CATEGORY_ATKCHANGE + CATEGORY_DEFCHANGE);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DAMAGE_STEP);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_hint_timing(duel, TIMING_DAMAGE_STEP, TIMING_DAMAGE_STEP | TIMINGS_CHECK_MONSTER_E);
    e1.set_condition(duel, |duel, _| !(duel.is_phase(PHASE_DAMAGE) && duel.is_damage_calculated()));
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, distg);
    e1.set_operation(duel, disop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_EXTRA_ATTACK);
    e2.set_value(duel, 1);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_code(duel, EFFECT_PIERCE);
    c.register_effect(duel, e3);
}

fn altproccostfilter(c: CardId, duel: &Duel) -> bool {
    c.is_spell(duel) && c.is_discardable(duel)
}

fn disfilter(c: CardId, duel: &Duel) -> bool {
    c.is_negatable_monster(duel) && c.is_type(duel, TYPE_EFFECT)
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, 1 - tp) && disfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(disfilter, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_NEGATE);
    let Some(tc) = duel
        .select_target(tp, disfilter, tp, 0, LOCATION_MZONE, 1, 1, None)?
        .and_then(|g| g.get_first())
    else {
        return Ok(false);
    };
    duel.set_operation_info(0, CATEGORY_DISABLE, Some(tc.into()), 1, tp, 0);
    duel.set_operation_info(0, CATEGORY_ATKCHANGE, Some(tc.into()), 1, tp, -tc.get_attack(duel));
    duel.set_operation_info(0, CATEGORY_DEFCHANGE, Some(tc.into()), 1, tp, -tc.get_defense(duel));
    Ok(false)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_faceup(duel) && tc.is_relate_to_effect(duel, e) && tc.is_can_be_disabled_by_effect(duel, e, true) {
        let c = e.get_handler(duel);
        tc.negate_effects(duel, c, 0, false);
        duel.adjust_instantly(Some(tc));
        if !tc.is_disabled(duel) {
            return Ok(());
        }
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_SET_ATTACK_FINAL);
        e1.set_value(duel, 0);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        tc.register_effect(duel, e1);
        let e2 = e1.clone_effect(duel);
        e2.set_code(duel, EFFECT_SET_DEFENSE_FINAL);
        tc.register_effect(duel, e2);
    }
    Ok(())
}
