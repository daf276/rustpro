// Wandering Gryphon Rider
use crate::prelude::*;
use crate::testing::{Check, HAND};

const ID: u32 = 2563463;

pub const LISTED_NAMES: &[u32] = &[TOKEN_ADVENTURER];
// Copies in the hand to summon while the field is empty; the Adventurer Token's maker, Rite of Aramesir, is not ported.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, spcon);
    e1.set_target(duel, stock::special_summon_self_target);
    e1.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TODECK + CATEGORY_NEGATE + CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| bravecon(duel, a.e));
    e2.set_target(duel, ngtg);
    e2.set_operation(duel, ngop);
    c.register_effect(duel, e2);
}

fn bravecon(duel: &Duel, e: EffectId) -> bool {
    duel.is_existing_matching_card(
        |c, duel| c.is_faceup(duel) && c.is_code(duel, TOKEN_ADVENTURER),
        e.get_handler_player(duel),
        LOCATION_ONFIELD,
        0,
        1,
        None,
    )
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    duel.is_main_phase() && (duel.get_field_group_count(a.tp, LOCATION_MZONE, 0) == 0 || bravecon(duel, a.e))
}

fn ngtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(!c.is_status(duel, STATUS_BATTLE_DESTROYED) && c.is_able_to_deck(duel) && duel.is_chain_negatable(a.ev as u8));
    }
    duel.set_operation_info(0, CATEGORY_TODECK, Some(c.into()), 1, tp, 0);
    duel.set_operation_info(0, CATEGORY_NEGATE, a.eg, 1, tp, 0);
    if let Some(re) = a.re {
        let rc = re.get_handler(duel);
        if rc.is_destructable(duel) && rc.is_relate_to_effect(duel, re) {
            duel.set_operation_info(0, CATEGORY_DESTROY, a.eg, 1, tp, 0);
        }
    }
    Ok(false)
}

fn ngop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let Some(re) = a.re else { return Ok(()) };
    if c.is_relate_to_effect(duel, a.e)
        && duel.send_to_deck(c, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)? > 0
        && c.is_location(duel, LOCATION_DECK | LOCATION_EXTRA)
        && duel.negate_activation(a.ev as u8)
        && re.get_handler(duel).is_relate_to_effect(duel, re)
    {
        duel.destroy(a.eg.unwrap_or_default(), REASON_EFFECT)?;
    }
    Ok(())
}
