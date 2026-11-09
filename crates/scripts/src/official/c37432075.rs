// Invocation - "Sword"
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, REMOVED};

const ID: u32 = 37432075;
const ALEISTER_THE_INVOKER: u32 = 86120751;
const FUSION_SUMMON: u64 = 1170;

pub const LISTED_NAMES: &[u32] = &[CARD_INVOCATION];

struct Params;

impl FusionSummonParams for Params {
    fn matfilter(c: CardId, duel: &Duel, _: EffectId, _: u8) -> bool {
        c.is_on_field(duel) && c.is_able_to_grave(duel)
    }

    fn extrafil(duel: &Duel, e: EffectId, tp: u8, _: CardSet) -> (Option<CardSet>, Option<FusionCheck>) {
        let filter = |c, duel: &Duel| Fusion::is_monster_filter(c, duel, CardId::is_faceup);
        let g = duel.get_matching_group(filter, tp, LOCATION_REMOVED, LOCATION_REMOVED, None);
        (Some(g), Some((checkmat, e.get_handler(duel))))
    }
}

// Invoked Magellanica to Fusion Summon from Aleister the Invoker and EARTH Tao the Chanter, the latter banished, and
// Aleister the Invoker in the Graveyard for the Graveyard effect.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (48791583, EXTRA),
    (ALEISTER_THE_INVOKER, MZONE),
    (46247516, REMOVED),
    (ID, GRAVE),
    (ALEISTER_THE_INVOKER, GRAVE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(FUSION_SUMMON), Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = Fusion::create_summon_eff::<Params>(duel, c);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TODECK + CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, tdtg);
    e2.set_operation(duel, tdop);
    c.register_effect(duel, e2);
}

fn checkmat(duel: &Duel, _: CardId, _: u8, sg: CardSet, fc: CardId) -> bool {
    fc.is_set_card(duel, &[SET_INVOKED]) || !sg.is_exists(duel, |c, duel| c.is_location(duel, LOCATION_REMOVED), 1, None)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    ((c.is_set_card(duel, &[SET_ALEISTER]) && c.is_monster(duel)) || c.is_code(duel, CARD_INVOCATION)) && c.is_able_to_hand(duel)
}

fn tdtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && thfilter(chkc, duel));
    }
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_able_to_deck(duel) && duel.is_existing_target(thfilter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_target(tp, thfilter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TODECK, Some(c.into()), 1, tp, 0);
    duel.set_operation_info(0, CATEGORY_TOHAND, g, 1, tp, 0);
    Ok(false)
}

fn tdop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let c = e.get_handler(duel);
    let tc = duel.get_first_target();
    if c.is_relate_to_effect(duel, e)
        && duel.send_to_deck(c, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)? > 0
        && c.is_location(duel, LOCATION_DECK)
        && let Some(tc) = tc
        && tc.is_relate_to_effect(duel, e)
    {
        duel.send_to_hand(tc, PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}
