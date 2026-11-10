// Red-Eyes Fusion
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND};

const ID: u32 = 6172122;

pub const LISTED_NAMES: &[u32] = &[CARD_REDEYES_B_DRAGON];

struct Params;

impl FusionSummonParams for Params {
    fn fusfilter(c: CardId, duel: &Duel) -> bool {
        c.lists_archetype_as_material(duel, &[SET_RED_EYES])
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
            duel.set_possible_operation_info(0, CATEGORY_TOGRAVE, None, 0, a.tp, LOCATION_DECK as i32);
        }
        Ok(true)
    }

    fn stage2(duel: &mut Duel, e: EffectId, tc: Option<CardId>, _: u8, _: Option<CardSet>, chk: i32) -> Script<()> {
        if let (1, Some(tc)) = (chk, tc) {
            let e1 = duel.create_effect(e.get_handler(duel));
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_code(duel, EFFECT_CHANGE_CODE);
            e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
            e1.set_value(duel, CARD_REDEYES_B_DRAGON as i32);
            e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
            tc.register_effect(duel, e1);
        }
        Ok(())
    }
}

// Red-Eyes Dark Dragoon in the Extra Deck, Fusion Summoned with Dark Magician in the hand and Red-Eyes Black Dragon in the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (37818794, EXTRA),
    (CARD_DARK_MAGICIAN, HAND),
    (CARD_REDEYES_B_DRAGON, DECK),
    (CARD_REDEYES_B_DRAGON, DECK),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummonedCode(37818794)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = Fusion::create_summon_eff::<Params>(duel, c);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_cost(duel, cost);
    c.register_effect(duel, e1);
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_activity_count(tp, ACTIVITY_SUMMON) == 0 && duel.get_activity_count(tp, ACTIVITY_SPSUMMON) == 0);
    }
    let c = a.e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CANNOT_SUMMON);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_OATH + EFFECT_FLAG_CLIENT_HINT);
    e1.set_target_range(duel, 1, 0);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e2.set_label_object(duel, Some(a.e));
    e2.set_target_spsummon(duel, |duel, e, _, _, _, _, _, se| e.get_label_object(duel) != se);
    duel.register_effect(e2, tp);
    Ok(false)
}

fn fcheck(duel: &Duel, _: CardId, tp: u8, sg: CardSet, fc: CardId) -> bool {
    sg.is_exists(duel, |c, duel| ffilter(c, duel, fc, tp), 1, None)
}

fn ffilter(c: CardId, duel: &Duel, fc: CardId, tp: u8) -> bool {
    let material = Fusion::material(duel, fc.get_original_code(duel));
    c.is_set_card_full(duel, &[SET_RED_EYES], Some(fc), SUMMON_TYPE_FUSION.into(), tp)
        && (material.is_empty() || c.is_summon_code(duel, Some(fc), SUMMON_TYPE_FUSION.into(), tp, &material))
}
