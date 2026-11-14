// A.I.'s Ritual
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE};

const WATER_LEVIATHAN: u32 = 37061511;

pub const LISTED_SERIES: &[u16] = &[SET_IGNISTER];

struct Params;

impl RitualParams for Params {
    fn filter(c: CardId, duel: &Duel) -> bool {
        c.is_race(duel, RACE_CYBERSE)
    }

    fn extrafil(duel: &Duel, e: EffectId, tp: u8) -> Option<CardSet> {
        (e.get_label(duel) == 1).then(|| duel.get_matching_group(mfilter, tp, LOCATION_GRAVE, 0, None))
    }

    fn matfilter(c: CardId, duel: &Duel, _: EffectId, _: u8) -> bool {
        c.is_set_card(duel, &[SET_IGNISTER])
    }
}

// Water Leviathan @Ignister in the hand, Tributing Wizard @Ignister or banishing Backup @Ignister from the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (WATER_LEVIATHAN, HAND),
    (WATER_LEVIATHAN, HAND),
    (3723262, MZONE),
    (3723262, MZONE),
    (30118811, GRAVE),
    (30118811, GRAVE),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummonedCode(WATER_LEVIATHAN)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = Ritual::create_proc::<Params, RITPROC_GREATER>(duel, c);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let faceup_ignister = |c: CardId, duel: &Duel| c.is_faceup(duel) && c.is_set_card(duel, &[SET_IGNISTER]);
        let label = duel.is_existing_matching_card(faceup_ignister, tp, LOCATION_MZONE, 0, 1, None);
        e.set_label(duel, i64::from(label));
    }
    if chk == 1 && e.get_label(duel) == 1 {
        duel.set_operation_info(0, CATEGORY_REMOVE, None, 1, tp, LOCATION_GRAVE as i32);
    }
    Ritual::target::<Params, RITPROC_GREATER>(duel, a, chk, chkc)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    Ritual::operation::<Params, RITPROC_GREATER>(duel, a)?;
    if a.e.get_label(duel) == 1 {
        a.e.set_label(duel, 0);
    }
    Ok(())
}

fn mfilter(c: CardId, duel: &Duel) -> bool {
    duel.is_player_affected_by_effect(c.get_controler(duel), CARD_SPIRIT_ELIMINATION)
        .is_none()
        && c.is_set_card(duel, &[SET_IGNISTER])
        && c.is_level_above(duel, 1)
        && c.is_able_to_remove(duel)
}
