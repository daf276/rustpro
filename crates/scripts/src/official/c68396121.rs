// Number C107: Neo Galaxy-Eyes Tachyon Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 68396121;
const TACHYON_DRAGON: u32 = 88177324;

pub const LISTED_NAMES: &[u32] = &[TACHYON_DRAGON];

// In the Extra Deck, Xyz Summoned with three Level 9 monsters.
pub const SUPPORT: &[(u32, u16)] = &[(ID, EXTRA), (ID, EXTRA), (12197223, MZONE), (53927851, MZONE), (85315450, MZONE)];
// The attack effect needs Tachyon Dragon as material, which only Rank-Up-Magic cards can attach.
pub const CHECKS: &[Check] = &[Check::Resolves, Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 1)))];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(9);
    const CT: usize = 3;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_operation(duel, negop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, atkcon);
    e2.set_cost(duel, atkcost);
    e2.set_target(duel, atktg);
    e2.set_operation(duel, atkop);
    c.register_effect(duel, e2);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && (c.is_location(duel, LOCATION_SZONE) || c.is_type(duel, TYPE_EFFECT)) && !c.is_disabled(duel)
}

fn negop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let g = duel.get_matching_group(filter, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, Some(c.into()));
    for tc in g {
        for code in [EFFECT_DISABLE, EFFECT_DISABLE_EFFECT] {
            let e1 = duel.create_effect(c);
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_code(duel, code);
            e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
            tc.register_effect(duel, e1);
        }
    }
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_target_range(duel, 0, 1);
    e1.set_value(duel, Value::EffectPlayer(|duel, _, re, _| re.get_handler(duel).is_on_field(duel)));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    let e2 = duel.create_effect(c);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT + EFFECT_FLAG_OATH);
    e2.set_description(duel, aux::stringid(ID, 2));
    e2.set_reset(duel, RESET_PHASE | PHASE_END);
    e2.set_target_range(duel, 0, 1);
    duel.register_effect(e2, tp);
    Ok(())
}

fn atkcon(duel: &Duel, a: &Args) -> bool {
    duel.is_able_to_enter_bp()
        && a.e
            .get_handler(duel)
            .get_overlay_group(duel)
            .is_exists(duel, |c, duel| c.is_code(duel, TACHYON_DRAGON), 1, None)
}

fn atkcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.check_release_group_cost(tp, aux::TRUE, (2, 2), false, None, Some(c.into())));
    }
    let g = duel.select_release_group_cost(tp, aux::TRUE, (2, 2), false, None, Some(c.into()))?;
    duel.release(g, REASON_COST)?;
    Ok(false)
}

fn atktg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    Ok(chk != 0 || a.e.get_handler(duel).get_effect_count(duel, EFFECT_EXTRA_ATTACK_MONSTER) == 0)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_EXTRA_ATTACK_MONSTER);
        e1.set_value(duel, 2);
        e1.set_reset(duel, RESETS_STANDARD_DISABLE_PHASE_END);
        c.register_effect(duel, e1);
    }
    Ok(())
}
