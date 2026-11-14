// Sunavalon Dryanome
use super::c93896655::{spcon, spop, sptg};
use crate::prelude::*;
use crate::testing::{EXTRA, MZONE, OPPONENT};

const ID: u32 = 65285459;

pub const LISTED_SERIES: &[u16] = &[SET_SUNVINE];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_race_full(duel, RACE_PLANT, Some(lc), sumtype, tp)
    }
}

// In the Extra Deck, Link Summoned with three Plants (Sunavalon Dryas and Sunseed Genius Loci), with Sunvine Link Monsters in the
// Extra Deck to summon and monsters of the opponent's to attack the Link Monster it points to.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (65563871, EXTRA),
    (93896655, MZONE),
    (27520594, MZONE),
    (27520594, MZONE),
    (89631139, MZONE | OPPONENT),
    (89631139, MZONE | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_IGNORE_BATTLE_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_DAMAGE);
    e2.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL + EFFECT_FLAG_DELAY);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 3, 0);
    e2.set_condition(duel, spcon);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_BE_BATTLE_TARGET);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_condition(duel, mvcon);
    e3.set_operation(duel, mvop);
    c.register_effect(duel, e3);
}

fn mvcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    let tp = c.get_controler(duel);
    let lg = c.get_linked_group(duel);
    duel.get_attack_target()
        .is_some_and(|d| lg.is_contains(d) && d.is_controler(duel, tp) && duel.get_location_count(tp, LOCATION_MZONE) > 0)
}

fn mvop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let Some(tc) = duel.get_attack_target() else { return Ok(()) };
    if tc.is_relate_to_battle(duel)
        && duel.negate_attack()?
        && duel.get_location_count(tp, LOCATION_MZONE) > 0
        && let Some(seq) = duel.select_disable_field(tp, 1, LOCATION_MZONE as u16, 0, 0)?
    {
        duel.move_sequence(tc, seq.trailing_zeros() as u8)?;
    }
    Ok(())
}
