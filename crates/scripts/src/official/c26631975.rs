// Dinomorphia Domain
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND};

const ID: u32 = 26631975;

pub const LISTED_SERIES: &[u16] = &[SET_DINOMORPHIA];

struct Params;

impl FusionSummonParams for Params {
    fn fusfilter(c: CardId, duel: &Duel) -> bool {
        c.is_set_card(duel, &[SET_DINOMORPHIA])
    }

    fn extrafil(duel: &Duel, _: EffectId, tp: u8, _: CardSet) -> (Option<CardSet>, Option<FusionCheck>) {
        let g = duel.get_matching_group(
            |c, duel| Fusion::is_monster_filter(c, duel, CardId::is_able_to_grave),
            tp,
            LOCATION_DECK,
            0,
            None,
        );
        (Some(g), None)
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
            (LOCATION_HAND | LOCATION_DECK | LOCATION_MZONE) as i32,
        );
        Ok(false)
    }
}

// Copies to activate and in the GY, Therizia and Diplos in the Deck as materials for Kentregina or Stealthbergia.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, GRAVE),
    (92133240, DECK),
    (92133240, DECK),
    (38628859, DECK),
    (38628859, DECK),
    (48832775, EXTRA),
    (74936480, EXTRA),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(1170), Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = Fusion::create_summon_eff::<Params>(duel, c);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_hint_timing(duel, 0, TIMING_MAIN_END);
    e1.set_condition(duel, |duel, _| duel.is_main_phase());
    e1.set_cost(duel, super::c78420796::cost);
    c.register_effect(duel, e1);
    super::c78420796::register_no_damage::<ID>(duel, c);
}
