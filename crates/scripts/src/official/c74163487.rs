// Restoration Point Guard
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 74163487;
const HONEYBOT: u32 = 34472920;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_race_full(duel, RACE_CYBERSE, Some(lc), sumtype, tp)
    }
}

// On the field with Honeybots to Link Summon from it, and Raigeki of the opponent to destroy the Link Monster.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, EXTRA),
    (HONEYBOT, EXTRA),
    (HONEYBOT, EXTRA),
    (24154052, MZONE),
    (24154052, MZONE),
    (24154052, MZONE),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_condition(duel, incon);
    e1.set_operation(duel, inop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EVENT_BE_MATERIAL);
    e2.set_condition(duel, |_, a| a.r == REASON_LINK);
    e2.set_operation(duel, regop);
    c.register_effect(duel, e2);
}

fn incfilter(c: CardId, duel: &Duel) -> bool {
    c.is_link_summoned(duel)
}

fn incon(duel: &Duel, a: &Args) -> bool {
    let Some(eg) = a.eg else { return false };
    eg.is_exists(duel, |c, duel| c.is_type(duel, TYPE_LINK), 1, None)
        && eg.is_exists(duel, incfilter, 1, None)
        && !eg.is_contains(a.e.get_handler(duel))
}

fn inop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if !c.is_relate_to_effect(duel, a.e) {
        return Ok(());
    }
    let e1 = duel.create_effect(c);
    e1.set_description(duel, 3008);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE + EFFECT_FLAG_CLIENT_HINT);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(duel, 1);
    e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    c.register_effect(duel, e2);
    Ok(())
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let rc = c.get_reason_card(duel);
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_DESTROYED);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_count_limit(duel, 1, ID);
    e1.set_label_object_group(duel, rc.map(Into::into));
    e1.set_range(duel, LOCATION_GRAVE);
    e1.set_condition(duel, spcon);
    e1.set_target(duel, stock::special_summon_self_target_player_0);
    e1.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e1);
    Ok(())
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let rc = a.e.get_label_object_group(duel).and_then(|g| g.get_first());
    rc.is_some_and(|rc| a.eg.is_some_and(|eg| eg.is_contains(rc))) && a.r & REASON_EFFECT != 0 && a.rp != a.tp
}
