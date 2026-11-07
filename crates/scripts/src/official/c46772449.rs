// Evilswarm Exciton Knight
use crate::prelude::*;
use crate::testing::{EXTRA, MZONE};

const ID: u32 = 46772449;

// In the Extra Deck, Xyz Summoned with Mystical Elf and Flying Kamakiri #2.
pub const SUPPORT: &[(u32, u16)] = &[(ID, EXTRA), (15025844, MZONE), (3134241, MZONE)];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(4);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_hint_timing(duel, TIMING_SPSUMMON, TIMING_BATTLE_START | TIMING_BATTLE_END);
    e1.set_count_limit_full(duel, 1, 0, EFFECT_COUNT_CODE_CHAIN);
    e1.set_condition(duel, descon);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    c.register_effect(duel, e1);
}

fn descon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    if !((duel.is_turn_player(tp) && duel.is_main_phase()) || (duel.is_turn_player(1 - tp) && duel.is_battle_phase())) {
        return false;
    }
    duel.get_field_group_count(tp, 0, LOCATION_HAND | LOCATION_ONFIELD)
        > duel.get_field_group_count(tp, LOCATION_HAND | LOCATION_ONFIELD, 0)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let g = duel.get_matching_group(
        aux::TRUE,
        tp,
        LOCATION_ONFIELD,
        LOCATION_ONFIELD,
        Some(a.e.get_handler(duel).into()),
    );
    if chk == 0 {
        return Ok(!g.is_empty());
    }
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), g.len() as u8, tp, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let exc = c.is_relate_to_effect(duel, e).then_some(c);
    let g = duel.get_matching_group(aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, exc.map(CardSet::from));
    if !g.is_empty() {
        duel.destroy(g, REASON_EFFECT)?;
    }
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CHANGE_DAMAGE);
    e1.set_target_range(duel, 0, 1);
    e1.set_value(duel, 0);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_NO_EFFECT_DAMAGE);
    duel.register_effect(e2, tp);
    Ok(())
}
