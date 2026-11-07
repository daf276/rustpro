// Fusion Destiny
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND};

const ID: u32 = 52947044;

struct Params;

impl FusionSummonParams for Params {
    fn fusfilter(c: CardId, duel: &Duel) -> bool {
        c.lists_archetype_as_material(duel, &[SET_DESTINY_HERO])
    }

    fn matfilter(c: CardId, duel: &Duel, _: EffectId, _: u8) -> bool {
        c.is_location(duel, LOCATION_HAND) && c.is_able_to_grave(duel)
    }

    fn extrafil(duel: &Duel, e: EffectId, tp: u8, _: CardSet) -> (Option<CardSet>, Option<FusionCheck>) {
        let filter = |c, duel: &Duel| Fusion::is_monster_filter(c, duel, CardId::is_able_to_grave);
        (
            Some(duel.get_matching_group(filter, tp, LOCATION_DECK, 0, None)),
            Some((fcheck, e.get_handler(duel))),
        )
    }

    fn extratg(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
        if chk != 0 {
            duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 0, a.tp, (LOCATION_HAND | LOCATION_DECK) as i32);
        }
        Ok(true)
    }

    fn stage2(duel: &mut Duel, e: EffectId, tc: Option<CardId>, tp: u8, _: Option<CardSet>, chk: i32) -> Script<()> {
        let c = e.get_handler(duel);
        match (chk, tc) {
            (0, Some(tc)) => {
                tc.register_flag_effect(duel, ID, RESETS_STANDARD_PHASE_END, 0, 2);
            }
            (1, Some(tc)) => {
                let turn = i64::from(duel.get_turn_count());
                let e2 = duel.create_effect(c);
                e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
                e2.set_code(duel, EVENT_PHASE + PHASE_END);
                e2.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
                e2.set_condition(duel, descon);
                e2.set_operation(duel, desop);
                e2.set_reset_full(duel, RESET_PHASE | PHASE_END, 2);
                e2.set_count_limit(duel, 1, 0);
                e2.set_label(duel, turn);
                e2.set_label_object_group(duel, Some(tc.into()));
                duel.register_effect(e2, tp);
            }
            (2, _) if e.is_has_type(duel, EFFECT_TYPE_ACTIVATE) => {
                let e1 = duel.create_effect(c);
                e1.set_description(duel, aux::stringid(ID, 0));
                e1.set_type(duel, EFFECT_TYPE_FIELD);
                e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
                e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
                e1.set_target_range(duel, 1, 0);
                e1.set_target_spsummon(duel, |duel, _, c, _, _, _, _, _| {
                    !(c.is_set_card(duel, &[SET_HERO]) && c.is_attribute(duel, ATTRIBUTE_DARK))
                });
                e1.set_reset(duel, RESET_PHASE | PHASE_END);
                duel.register_effect(e1, tp);
            }
            _ => {}
        }
        Ok(())
    }
}

// Destiny HERO - Dystopia in the Extra Deck, Fusion Summoned from the hand and the Deck with Destiny HERO - Malicious and
// Dreadmaster.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (90579153, EXTRA), (9411399, HAND), (40591390, DECK), (40591390, DECK)];
pub const CHECKS: &[Check] = &[Check::SpecialSummonedCode(90579153)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = Fusion::create_summon_eff::<Params>(duel, c);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    c.register_effect(duel, e1);
}

fn fcheck(duel: &Duel, _: CardId, tp: u8, sg: CardSet, fc: CardId) -> bool {
    sg.is_exists(
        duel,
        |c, duel| c.is_set_card_full(duel, &[SET_DESTINY_HERO], Some(fc), SUMMON_TYPE_FUSION.into(), tp),
        1,
        None,
    )
}

fn descon(duel: &Duel, a: &Args) -> bool {
    let tc = a.e.get_label_object_group(duel).and_then(|g| g.get_first());
    duel.get_turn_count() != a.e.get_label(duel) as i32 && tc.is_some_and(|tc| tc.get_flag_effect(duel, ID) != 0)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = a.e.get_label_object_group(duel).and_then(|g| g.get_first()) {
        duel.destroy(tc, REASON_EFFECT)?;
    }
    Ok(())
}
