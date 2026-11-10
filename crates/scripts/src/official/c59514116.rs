// Secrets of Dark Magic
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND};

const ID: u32 = 59514116;
const DARK_CAVALRY: u32 = 73452089;
const GAIA: u32 = 6368038;
const MAGICIAN_OF_BLACK_CHAOS: u32 = 30208479;
const SANGAN: u32 = 26202165;

pub const LISTED_NAMES: &[u32] = &[CARD_DARK_MAGICIAN, CARD_DARK_MAGICIAN_GIRL];

struct FusionEffect;

impl FusionSummonParams for FusionEffect {
    const DESC: u64 = aux::stringid(ID, 0);

    fn extrafil(duel: &Duel, e: EffectId, _: u8, _: CardSet) -> (Option<CardSet>, Option<FusionCheck>) {
        (None, Some((matcheck, e.get_handler(duel))))
    }
}

struct RitualEffect;

impl RitualParams for RitualEffect {
    const DESC: u64 = aux::stringid(ID, 1);
    const FORCEDSELECTION: Option<ForcedSelection> = Some(|duel, _, _, sg, _| (sg.is_exists(duel, is_dark_magician, 1, None), false));
}

// In the hand with Dark Magician, Gaia The Fierce Knight and Sangan, to Fusion Summon Dark Cavalry and to Ritual Summon
// Magician of Black Chaos.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (DARK_CAVALRY, EXTRA),
    (MAGICIAN_OF_BLACK_CHAOS, HAND),
    (CARD_DARK_MAGICIAN, HAND),
    (CARD_DARK_MAGICIAN, HAND),
    (GAIA, HAND),
    (SANGAN, HAND),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummonedCode(DARK_CAVALRY),
    Check::SpecialSummonedCode(MAGICIAN_OF_BLACK_CHAOS),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = Fusion::create_summon_eff::<FusionEffect>(duel, c);
    e1.set_cost(duel, costhint);
    c.register_effect(duel, e1);
    let e2 = Ritual::create_proc::<RitualEffect, RITPROC_GREATER>(duel, c);
    e2.set_cost(duel, costhint);
    c.register_effect(duel, e2);
}

fn costhint(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.hint(HINT_OPSELECTED, 1 - a.tp, a.e.get_description(duel));
    Ok(false)
}

fn matcheck(duel: &Duel, _: CardId, tp: u8, sg: CardSet, fc: CardId) -> bool {
    sg.is_exists(
        duel,
        |c, duel| {
            c.is_summon_code(
                duel,
                Some(fc),
                SUMMON_TYPE_FUSION as u64,
                tp,
                &[CARD_DARK_MAGICIAN, CARD_DARK_MAGICIAN_GIRL],
            )
        },
        1,
        None,
    )
}

fn is_dark_magician(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, CARD_DARK_MAGICIAN) || c.is_code(duel, CARD_DARK_MAGICIAN_GIRL)
}
