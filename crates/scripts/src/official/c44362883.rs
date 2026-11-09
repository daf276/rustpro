// Branded Fusion
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND};

const ID: u32 = 44362883;

pub const LISTED_NAMES: &[u32] = &[CARD_ALBAZ];

struct Params;

impl FusionSummonParams for Params {
    const EXACTCOUNT: Option<usize> = Some(2);

    fn fusfilter(c: CardId, duel: &Duel) -> bool {
        c.lists_code_as_material(duel, &[CARD_ALBAZ])
    }

    fn extrafil(duel: &Duel, e: EffectId, tp: u8, _: CardSet) -> (Option<CardSet>, Option<FusionCheck>) {
        let filter = |c, duel: &Duel| Fusion::is_monster_filter(c, duel, CardId::is_able_to_grave);
        (
            Some(duel.get_matching_group(filter, tp, LOCATION_DECK, 0, None)),
            Some((fcheck, e.get_handler(duel))),
        )
    }

    fn extratg(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
        if chk == 0 {
            return Ok(true);
        }
        duel.set_operation_info(
            0,
            CATEGORY_TOGRAVE,
            None,
            0,
            a.tp,
            (LOCATION_HAND | LOCATION_MZONE | LOCATION_DECK) as i32,
        );
        Ok(false)
    }
}

// Albion the Branded Dragon in the Extra Deck, Fusion Summoned with Fallen of Albaz and Mystical Elf (LIGHT).
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (87746184, EXTRA), (CARD_ALBAZ, HAND), (15025844, HAND)];
pub const CHECKS: &[Check] = &[Check::SpecialSummonedCode(87746184)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = Fusion::create_summon_eff::<Params>(duel, c);
    e1.set_cost(duel, cost);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    c.register_effect(duel, e1);
    duel.add_custom_activity_counter(
        ID,
        ACTIVITY_SPSUMMON,
        CounterFilter::Card(|duel, c| !c.is_summon_location(duel, LOCATION_EXTRA) || c.is_type(duel, TYPE_FUSION)),
    );
}

fn fcheck(duel: &Duel, _: CardId, tp: u8, sg: CardSet, fc: CardId) -> bool {
    sg.is_exists(
        duel,
        |c, duel| c.is_summon_code(duel, Some(fc), SUMMON_TYPE_FUSION.into(), tp, &[CARD_ALBAZ]),
        1,
        None,
    )
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(duel.get_custom_activity_count(ID, tp, ACTIVITY_SPSUMMON) == 0);
    }
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_OATH + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_spsummon(duel, |duel, _, c, _, _, _, _, _| {
        !c.is_type(duel, TYPE_FUSION) && c.is_location(duel, LOCATION_EXTRA)
    });
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    aux::add_temp_lizard_check_full(duel, c, tp, |duel, _, c| !c.is_original_type(duel, TYPE_FUSION));
    Ok(false)
}
