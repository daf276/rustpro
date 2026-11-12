// Psychic Nightmare
use crate::prelude::*;
use crate::testing::{EXTRA, MZONE};

const ID: u32 = 7582066;

// In the Extra Deck, Synchro Summoned from Stinger the Electric Expeller (Level 3 Tuner) and Chosen by the World Chalice (Level 3
// Psychic).
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (57649113, MZONE),
    (57649113, MZONE),
    (22916281, MZONE),
    (22916281, MZONE),
];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, |c, duel| c.is_race_full(duel, RACE_PSYCHIC, Some(sc), sumtype, tp))
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_ATKCHANGE);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_condition(duel, |duel, a| duel.get_field_group_count(a.tp, 0, LOCATION_HAND) != 0);
    e1.set_operation(duel, atkop);
    c.register_effect(duel, e1);
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) || c.is_facedown(duel) || duel.get_field_group_count(tp, 0, LOCATION_HAND) == 0 {
        return Ok(());
    }
    let g = duel.get_field_group(tp, 0, LOCATION_HAND).random_select(duel, tp, 1);
    let Some(tc) = g.get_first() else { return Ok(()) };
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_CARDTYPE);
    let op = duel.select_option(tp, &[70, 71, 72])?;
    duel.confirm_cards(tp, tc)?;
    duel.shuffle_hand(1 - tp)?;
    if (op == 0 && tc.is_monster(duel)) || (op == 1 && tc.is_spell(duel)) || (op == 2 && tc.is_trap(duel)) {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_value(duel, 1000);
        e1.set_reset_full(duel, RESETS_STANDARD_DISABLE_PHASE_END, 2);
        c.register_effect(duel, e1);
    }
    Ok(())
}
