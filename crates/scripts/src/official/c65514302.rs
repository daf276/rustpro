// Magnet Bonding
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND, MZONE};

const ID: u32 = 65514302;

pub const LISTED_NAMES: &[u32] = &[44839512];
pub const LISTED_SERIES: &[u16] = &[SET_MAGNET_WARRIOR, SET_MAGNA_WARRIOR];

struct Params;

impl FusionSummonParams for Params {
    const EXTRAOP: Option<FusionMaterialFn> = Some(Fusion::shuffle_material);

    fn fusfilter(c: CardId, duel: &Duel) -> bool {
        c.is_race(duel, RACE_ROCK)
    }

    fn matfilter(c: CardId, duel: &Duel, _: EffectId, _: u8) -> bool {
        c.is_race(duel, RACE_ROCK) && c.is_able_to_deck(duel)
    }

    fn extrafil(duel: &Duel, _: EffectId, tp: u8, _: CardSet) -> (Option<CardSet>, Option<FusionCheck>) {
        (
            Some(duel.get_matching_group(fextrafilter, tp, LOCATION_GRAVE | LOCATION_REMOVED, 0, None)),
            None,
        )
    }
}

// In the hand with Linear Magnum, Alpha The Magnet Warrior and Valkyrion in the Deck to add, and Plasma Magnum in the Extra Deck
// to Fusion Summon with Magnet Warriors from the hand, field and GY.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (44839512, DECK),
    (99785935, DECK),
    (75347539, DECK),
    (75347539, DECK),
    (47247792, EXTRA),
    (39256679, HAND),
    (11549357, MZONE),
    (99785935, GRAVE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::SpecialSummonedCode(47247792)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_condition(duel, |duel, _| duel.is_main_phase());
    e1.set_target(duel, efftg);
    e1.set_operation(duel, effop);
    e1.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER);
    c.register_effect(duel, e1);
}

fn magnetthfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_code(duel, 44839512) || (c.is_level_below(duel, 4) && c.is_set_card(duel, &[SET_MAGNET_WARRIOR]))) && c.is_able_to_hand(duel)
}

fn magnathfilter(c: CardId, duel: &Duel) -> bool {
    c.is_level(duel, &[8]) && c.is_set_card(duel, &[SET_MAGNA_WARRIOR]) && c.is_able_to_hand(duel)
}

fn fextrafilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_ROCK) && c.is_faceup(duel) && c.is_able_to_deck(duel)
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let b1 = !duel.has_flag_effect(tp, ID) && duel.is_existing_matching_card(magnetthfilter, tp, LOCATION_DECK, 0, 1, None);
    let b2 = !duel.has_flag_effect(tp, ID + 1) && duel.is_existing_matching_card(magnathfilter, tp, LOCATION_DECK, 0, 1, None);
    let b3 = !duel.has_flag_effect(tp, ID + 2) && {
        Fusion::new_summon_eff_closure(duel, e, false);
        Fusion::summon_eff_tg::<Params>(duel, a, 0, None)?
    };
    if chk == 0 {
        return Ok(b1 || b2 || b3);
    }
    let op = duel.select_effect(
        tp,
        &[(b1, aux::stringid(ID, 1)), (b2, aux::stringid(ID, 2)), (b3, aux::stringid(ID, 3))],
    )?;
    e.set_label(duel, op.map_or(0, |op| op as i64));
    match op {
        Some(op @ (1 | 2)) => {
            e.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
            duel.register_flag_effect(tp, ID + op as u32 - 1, RESET_PHASE | PHASE_END, 0, 1);
            duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
        }
        Some(3) => {
            e.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_FUSION_SUMMON + CATEGORY_TODECK);
            duel.register_flag_effect(tp, ID + 2, RESET_PHASE | PHASE_END, 0, 1);
            let locations = LOCATION_HAND | LOCATION_MZONE | LOCATION_GRAVE | LOCATION_REMOVED;
            duel.set_operation_info(0, CATEGORY_TODECK, None, 1, tp, locations as i32);
            duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
        }
        _ => {}
    }
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    match a.e.get_label(duel) {
        1 => stock::search(duel, a, magnetthfilter, LOCATION_DECK),
        2 => stock::search(duel, a, magnathfilter, LOCATION_DECK),
        3 => {
            Fusion::new_summon_eff_closure(duel, a.e, true);
            Fusion::summon_eff_op::<Params>(duel, a)
        }
        _ => Ok(()),
    }
}
