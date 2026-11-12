// Locomotion R-Genex
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 38354937;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f1(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_set_card_full(duel, &[SET_GENEX], Some(sc), sumtype, tp)
    }

    fn f2(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_not_tuner(duel, sc, tp) && c.is_attribute_full(duel, ATTRIBUTE_DARK, Some(sc), sumtype, tp)
    }
}

// In the Extra Deck, Synchro Summoned from Genex Controller (a Level 3 Tuner) and Swordstalker (DARK, Level 6), against the
// opponent's Psychic Kappa to take.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (68505803, MZONE),
    (68505803, MZONE),
    (50005633, MZONE),
    (50005633, MZONE),
    (7892180, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_CONTROL);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_synchro_summoned(duel));
    e1.set_operation(duel, ctop);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.has_level(duel)
}

fn ctop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_matching_group(filter, tp, 0, LOCATION_MZONE, None);
    let Some((mut sg, _)) = g.get_max_group(duel, |c, duel| c.get_level(duel).into()) else {
        return Ok(());
    };
    if sg.len() > 1 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONTROL);
        sg = sg.select(duel, tp, 1, 1, None)?;
    }
    if let Some(tc) = sg.get_first() {
        duel.get_control(tc, tp)?;
    }
    Ok(())
}
