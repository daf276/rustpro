// Downerd Magician
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 72167543;

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> = Some(|c, duel, xyz, sumtype, tp| c.is_race_full(duel, RACE_SPELLCASTER, Some(xyz), sumtype, tp));
    const LV: Option<u32> = Some(4);
    const CT: usize = 2;
    const ALTERF: Option<AlterFilter> = Some(ovfilter);
    const DESC: u64 = aux::stringid(ID, 0);
}

// In the Extra Deck, Xyz Summoned with Level 4 Spellcasters (Mystical Elf, Ancient Sorcerer) or on Tantrum Toddler, against
// Sangan to battle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (15025844, MZONE),
    (15025844, MZONE),
    (36821538, MZONE),
    (7020743, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_PIERCE);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_value(duel, Value::Card(|duel, _, c| c.get_overlay_count(duel) as i32 * 200));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e3.set_code(duel, EVENT_BATTLED);
    e3.set_operation(duel, rmop);
    c.register_effect(duel, e3);
}

fn ovfilter(c: CardId, duel: &Duel, _: u8, _: CardId) -> bool {
    c.is_faceup(duel) && c.is_rank_below(duel, 3) && duel.is_phase(PHASE_MAIN2)
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    a.e.get_handler(duel).remove_overlay_card(duel, a.tp, 1, 1, REASON_EFFECT)?;
    Ok(())
}
