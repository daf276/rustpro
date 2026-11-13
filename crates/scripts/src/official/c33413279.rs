// Explosive Magician
use crate::prelude::*;
use crate::testing::{EXTRA, GRAVE, MZONE, OPPONENT, SZONE};

const ID: u32 = 33413279;

pub const COUNTER_LIST: &[u16] = &[COUNTER_SPELL];

// In the Extra Deck, Synchro Summoned from Lunar Rabbit Omajinai (Level 1 Tuner) and Leo Wizard (Level 5 Spellcaster), with
// Selene (Link Summoned from Lucky Trinket and Leo Wizard) putting a Spell Counter on herself for each Spell in the Graveyards.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (45819647, EXTRA),
    (45819647, EXTRA),
    (47643326, MZONE),
    (47643326, MZONE),
    (47643326, MZONE),
    (4392470, MZONE),
    (4392470, MZONE),
    (4392470, MZONE),
    (3985011, MZONE),
    (3985011, MZONE),
    (46130346, GRAVE),
    (46130346, GRAVE),
    (46130346, GRAVE),
    (46130346, GRAVE),
    (46130346, GRAVE),
    (46130346, GRAVE),
    (5318639, SZONE | OPPONENT),
    (5318639, SZONE | OPPONENT),
];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, |c, duel| {
            c.is_race_full(duel, RACE_SPELLCASTER, Some(sc), sumtype, tp)
        })
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_cost(duel, Cost::remove_counter_from_field::<COUNTER_SPELL, 2>);
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    c.register_effect(duel, e1);
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_controler(duel, 1 - tp) && chkc.is_spell_trap(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_spell_trap, tp, 0, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, CardId::is_spell_trap, tp, 0, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, tp, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
        && tc.is_controler(duel, 1 - a.tp)
    {
        duel.destroy(tc, REASON_EFFECT)?;
    }
    Ok(())
}
