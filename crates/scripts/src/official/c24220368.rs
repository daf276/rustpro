// Gem-Knight Dispersion
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND, REMOVED};

const ID: u32 = 24220368;

pub const LISTED_SERIES: &[u16] = &[SET_GEM_KNIGHT, SET_GEM];
pub const LISTED_NAMES: &[u32] = &[GEM_KNIGHT_FUSION];

const GEM_KNIGHT_FUSION: u32 = 1264319;
const RUBY: u32 = 76614340;

struct Params;

impl FusionSummonParams for Params {
    fn fusfilter(c: CardId, duel: &Duel) -> bool {
        c.is_set_card(duel, &[SET_GEM_KNIGHT])
    }

    fn extrafil(duel: &Duel, e: EffectId, tp: u8, _: CardSet) -> (Option<CardSet>, Option<FusionCheck>) {
        fextra(duel, e, tp)
    }

    fn extratg(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
        if chk == 0 {
            return Ok(true);
        }
        duel.set_possible_operation_info(0, CATEGORY_TOGRAVE, None, 0, a.tp, (LOCATION_DECK | LOCATION_EXTRA) as i32);
        Ok(false)
    }
}

// Gem-Knight Ruby in the Extra Deck, Fusion Summoned with Gem-Knight Garnet from the hand, or from the Deck with Gem-Knight Fusion
// in the GY; Garnet in the Deck and banished to add.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (GEM_KNIGHT_FUSION, GRAVE),
    (RUBY, EXTRA),
    (RUBY, EXTRA),
    (91731841, HAND),
    (91731841, DECK),
    (91731841, DECK),
    (91731841, REMOVED),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::SpecialSummonedCode(RUBY)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_target(duel, efftg);
    e1.set_operation(duel, effop);
    c.register_effect(duel, e1);
}

fn fcheck(duel: &Duel, _: CardId, _: u8, sg: CardSet, _: CardId) -> bool {
    sg.filter_count(duel, |c, duel| c.is_location(duel, LOCATION_DECK | LOCATION_EXTRA), None) <= 2
}

fn fextrafilter(c: CardId, duel: &Duel) -> bool {
    !c.is_race(duel, RACE_ROCK) && c.is_set_card(duel, &[SET_GEM_KNIGHT]) && c.is_monster(duel) && c.is_able_to_grave(duel)
}

fn fextra(duel: &Duel, e: EffectId, tp: u8) -> (Option<CardSet>, Option<FusionCheck>) {
    if duel.is_existing_matching_card(|c, duel| c.is_code(duel, GEM_KNIGHT_FUSION), tp, LOCATION_GRAVE, 0, 1, None) {
        let eg = duel.get_matching_group(fextrafilter, tp, LOCATION_DECK | LOCATION_EXTRA, 0, None);
        if !eg.is_empty() {
            return (Some(eg), Some((fcheck, e.get_handler(duel))));
        }
    }
    (None, None)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_GEM])
        && c.is_monster(duel)
        && (c.is_location(duel, LOCATION_DECK) || c.is_faceup(duel))
        && c.is_able_to_hand(duel)
}

fn fusion_target(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    Fusion::new_summon_eff_closure(duel, a.e, false);
    Fusion::summon_eff_tg::<Params>(duel, a, chk, None)
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let b1 = duel.get_flag_effect(tp, ID) == 0 && fusion_target(duel, a, 0)?;
    let b2 =
        duel.get_flag_effect(tp, ID + 1) == 0 && duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK | LOCATION_REMOVED, 0, 1, None);
    if chk == 0 {
        return Ok(b1 || b2);
    }
    let op = duel.select_effect(tp, &[(b1, aux::stringid(ID, 1)), (b2, aux::stringid(ID, 2))])?;
    e.set_label(duel, op.map_or(0, |op| op as i64));
    match op {
        Some(1) => {
            e.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_FUSION_SUMMON);
            duel.register_flag_effect(tp, ID, RESET_PHASE | PHASE_END, 0, 1);
            fusion_target(duel, a, 1)?;
        }
        Some(2) => {
            e.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
            duel.register_flag_effect(tp, ID + 1, RESET_PHASE | PHASE_END, 0, 1);
            duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, (LOCATION_DECK | LOCATION_REMOVED) as i32);
        }
        _ => {}
    }
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    match a.e.get_label(duel) {
        1 => {
            Fusion::new_summon_eff_closure(duel, a.e, true);
            Fusion::summon_eff_op::<Params>(duel, a)?;
        }
        2 => {
            stock::search(duel, a, thfilter, LOCATION_DECK | LOCATION_REMOVED)?;
            let e1 = duel.create_effect(a.e.get_handler(duel));
            e1.set_description(duel, aux::stringid(ID, 3));
            e1.set_type(duel, EFFECT_TYPE_FIELD);
            e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
            e1.set_code(duel, EFFECT_CHANGE_DAMAGE);
            e1.set_target_range(duel, 0, 1);
            e1.set_condition(duel, |duel, _| duel.is_main_phase());
            e1.set_value(
                duel,
                Value::ChangeDamage(|_, _, _, val, reason, _, _| if reason & REASON_EFFECT > 0 { val / 2 } else { val }),
            );
            e1.set_reset(duel, RESET_PHASE | PHASE_END);
            duel.register_effect(e1, a.tp);
        }
        _ => {}
    }
    Ok(())
}
