// Destruction Sword Memories
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND, MZONE};

const ID: u32 = 32104431;
const DRAGON_DESTROYER: u32 = 86240887;

pub const LISTED_SERIES: &[u16] = &[SET_DESTRUCTION_SWORD, SET_BUSTER_BLADER];
pub const LISTED_NAMES: &[u32] = &[DRAGON_DESTROYER];

struct Params;

impl FusionSummonParams for Params {
    const EXTRAOP: Option<FusionMaterialFn> = Some(Fusion::banish_material);

    fn fusfilter(c: CardId, duel: &Duel) -> bool {
        c.is_code(duel, DRAGON_DESTROYER)
    }

    fn matfilter(c: CardId, duel: &Duel, _: EffectId, _: u8) -> bool {
        aux::sp_elim_filter(c, duel, false, false) && c.is_able_to_remove(duel)
    }

    fn extrafil(duel: &Duel, _: EffectId, tp: u8, _: CardSet) -> (Option<CardSet>, Option<FusionCheck>) {
        if duel.is_player_affected_by_effect(tp, CARD_SPIRIT_ELIMINATION).is_none() {
            let filter = |c: CardId, duel: &Duel| c.is_monster(duel) && c.is_able_to_remove(duel);
            return (Some(duel.get_matching_group(filter, tp, LOCATION_GRAVE, 0, None)), None);
        }
        (None, None)
    }

    fn extratg(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
        if chk == 0 {
            return Ok(true);
        }
        duel.set_operation_info(0, CATEGORY_REMOVE, None, 0, a.tp, LOCATION_GRAVE as i32);
        Ok(false)
    }
}

// In hand with Buster Whelp to discard and Buster Blader monsters in the Deck, and in the Graveyard with Buster Blader and a
// Dragon to banish for the Dragon Destroyer Swordsman.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, GRAVE),
    (49823708, HAND),
    (49823708, HAND),
    (CARD_BUSTER_BLADER, DECK),
    (CARD_BUSTER_BLADER, DECK),
    (3428069, DECK),
    (CARD_BUSTER_BLADER, GRAVE),
    (CARD_BUSTER_BLADER, MZONE),
    (17658803, GRAVE),
    (17658803, MZONE),
    (DRAGON_DESTROYER, EXTRA),
    (DRAGON_DESTROYER, EXTRA),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_cost(duel, cost);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_FUSION_SUMMON + CATEGORY_REMOVE);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER);
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, Fusion::summon_eff_tg::<Params>);
    e2.set_operation(duel, Fusion::summon_eff_op::<Params>);
    c.register_effect(duel, e2);
}

fn costfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_DESTRUCTION_SWORD]) && c.is_discardable(duel)
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(costfilter, tp, LOCATION_HAND, 0, 1, None));
    }
    duel.discard_hand(tp, costfilter, 1, 1, REASON_COST | REASON_DISCARD)?;
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_BUSTER_BLADER]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_DECK)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::special_summon(duel, a, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_DECK, POS_FACEUP)
}
