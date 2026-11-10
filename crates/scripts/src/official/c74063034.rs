// Invocation
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, REMOVED};

const ID: u32 = 74063034;
const ALEISTER_THE_INVOKER: u32 = 86120751;
const FUSION_SUMMON: u64 = 1170;

pub const LISTED_NAMES: &[u32] = &[ALEISTER_THE_INVOKER];

struct Params;

impl FusionSummonParams for Params {
    fn matfilter(c: CardId, duel: &Duel, _: EffectId, _: u8) -> bool {
        (c.is_location(duel, LOCATION_HAND) && c.is_able_to_grave(duel)) || (c.is_on_field(duel) && c.is_able_to_remove(duel))
    }

    fn extrafil(duel: &Duel, e: EffectId, tp: u8, _: CardSet) -> (Option<CardSet>, Option<FusionCheck>) {
        let check: Option<FusionCheck> = Some((checkmat, e.get_handler(duel)));
        if duel.is_player_affected_by_effect(tp, CARD_SPIRIT_ELIMINATION).is_some() {
            return (None, check);
        }
        let filter = |c, duel: &Duel| Fusion::is_monster_filter(c, duel, CardId::is_able_to_remove);
        (
            Some(duel.get_matching_group(filter, tp, LOCATION_GRAVE, LOCATION_GRAVE, None)),
            check,
        )
    }

    const EXTRAOP: Option<FusionMaterialFn> = Some(extraop);

    fn extratg(duel: &mut Duel, _: &Args, chk: i32) -> Script<bool> {
        if chk == 0 {
            return Ok(true);
        }
        let locations = LOCATION_MZONE | LOCATION_GRAVE;
        duel.set_possible_operation_info(0, CATEGORY_REMOVE, None, 0, PLAYER_EITHER, locations as i32);
        Ok(false)
    }
}

fn extraop(duel: &mut Duel, _: EffectId, _: CardId, _: u8, sg: &mut CardSet) -> Script<()> {
    let rg = sg.filter(duel, |c, duel| c.is_location(duel, LOCATION_GRAVE | LOCATION_ONFIELD), None);
    if !rg.is_empty() {
        duel.remove(rg, POS_FACEUP, REASON_EFFECT + REASON_MATERIAL + REASON_FUSION)?;
        *sg = sg.difference(rg);
    }
    Ok(())
}

// Invoked Magellanica to Fusion Summon from Aleister the Invoker and EARTH Tao the Chanter in hand, and a copy in the
// Graveyard with the banished Aleister the Invoker for the Graveyard effect.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (48791583, EXTRA),
    (ALEISTER_THE_INVOKER, HAND),
    (46247516, HAND),
    (ID, GRAVE),
    (ALEISTER_THE_INVOKER, REMOVED),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(FUSION_SUMMON), Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = Fusion::create_summon_eff::<Params>(duel, c);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TODECK + CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_target(duel, tdtg);
    e2.set_operation(duel, tdop);
    c.register_effect(duel, e2);
}

fn checkmat(duel: &Duel, _: CardId, _: u8, sg: CardSet, fc: CardId) -> bool {
    fc.is_set_card(duel, &[SET_INVOKED]) || !sg.is_exists(duel, |c, duel| c.is_location(duel, LOCATION_GRAVE | LOCATION_ONFIELD), 1, None)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_code(duel, ALEISTER_THE_INVOKER) && c.is_able_to_hand(duel)
}

fn tdtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_REMOVED) && chkc.is_controler(duel, tp) && thfilter(chkc, duel));
    }
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_able_to_deck(duel) && duel.is_existing_target(thfilter, tp, LOCATION_REMOVED, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let g = duel.select_target(tp, thfilter, tp, LOCATION_REMOVED, 0, 1, 1, None)?;
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
