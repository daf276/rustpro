// Number S37: Spider Shark
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 79625003;
pub const XYZ_NUMBER: u32 = 37;
const HOPE_WOVEN_SPIDER_SHARK: u32 = 37279508;

pub const LISTED_NAMES: &[u32] = &[HOPE_WOVEN_SPIDER_SHARK];

// In the Extra Deck, Xyz Summoned with three Level 5 WATER Normal Monsters (Misairuzame), with the opponent's monsters to banish and
// Dark Hole in the opponent's hand to destroy it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (33178416, MZONE),
    (33178416, MZONE),
    (33178416, MZONE),
    (33178416, MZONE),
    (33178416, MZONE),
    (30655537, MZONE | OPPONENT),
    (30655537, MZONE | OPPONENT),
    (30655537, MZONE | OPPONENT),
    (53129443, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> = Some(|c, duel, scard, sumtype, tp| c.is_attribute_full(duel, ATTRIBUTE_WATER, Some(scard), sumtype, tp));
    const LV: Option<u32> = Some(5);
    const CT: usize = 3;
    const ALTERF: Option<AlterFilter> = Some(|c, duel, tp, lc| {
        c.is_summon_code(duel, Some(lc), u64::from(SUMMON_TYPE_XYZ), tp, &[HOPE_WOVEN_SPIDER_SHARK]) && c.is_faceup(duel)
    });
    const DESC: u64 = aux::stringid(ID, 0);
}

struct ReturnOp;

impl aux::DelayedOperation for ReturnOp {
    fn operate(duel: &mut Duel, rg: CardSet, a: &Args) -> Script<()> {
        aux::DefaultFieldReturnOp::operate(duel, rg, a)
    }

    fn condition(duel: &Duel, _: CardSet, a: &Args) -> bool {
        duel.is_turn_player(1 - a.tp)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_category(duel, CATEGORY_REMOVE + CATEGORY_ATKCHANGE);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, rmtg);
    e1.set_operation(duel, rmop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 2));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_PHASE + PHASE_END);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_condition(duel, spcon);
    e2.set_target(duel, stock::special_summon_self_target);
    e2.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e2);
}

fn rmtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, 1 - tp) && chkc.is_able_to_remove(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_able_to_remove, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_target(tp, CardId::is_able_to_remove, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_REMOVE, g, 1, tp, 0);
    Ok(false)
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
        && aux::remove_until::<ReturnOp>(duel, tc, 0, REASON_EFFECT, PHASE_STANDBY, ID, e, tp)?.is_some()
    {
        let g = duel.get_matching_group(CardId::is_faceup, tp, 0, LOCATION_MZONE, None);
        if g.is_empty() {
            return Ok(());
        }
        let c = e.get_handler(duel);
        for atkc in g {
            let e1 = duel.create_effect(c);
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
            e1.set_code(duel, EFFECT_UPDATE_ATTACK);
            e1.set_value(duel, -1000);
            e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
            atkc.register_effect(duel, e1);
        }
    }
    Ok(())
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.get_turn_id(duel) == duel.get_turn_count()
        && c.is_xyz_summoned(duel)
        && c.is_previous_location(duel, LOCATION_MZONE)
        && c.is_reason(duel, REASON_DESTROY)
}
