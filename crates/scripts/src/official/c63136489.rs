// Chimera Fusion
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND};

const ID: u32 = 63136489;
const GAZELLE: u32 = 5818798;
const BERFOMET: u32 = 77207191;

pub const LISTED_NAMES: &[u32] = &[CARD_CHIMERA_MYTHICAL_BEAST, GAZELLE, BERFOMET];

struct Params;

impl FusionSummonParams for Params {
    fn extrafil(duel: &Duel, e: EffectId, _: u8, _: CardSet) -> (Option<CardSet>, Option<FusionCheck>) {
        (None, Some((matcheck, e.get_handler(duel))))
    }
}

// Chimera the Flying Mythical Beast in the Extra Deck and the Graveyard, Fusion Summoned with Gazelle the King of Mythical Beasts
// and Berfomet in the hand; a copy in the Graveyard with both in the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (CARD_CHIMERA_MYTHICAL_BEAST, EXTRA),
    (GAZELLE, HAND),
    (BERFOMET, HAND),
    (ID, GRAVE),
    (CARD_CHIMERA_MYTHICAL_BEAST, GRAVE),
    (GAZELLE, DECK),
    (BERFOMET, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummonedCode(CARD_CHIMERA_MYTHICAL_BEAST),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = Fusion::create_summon_eff::<Params>(duel, c);
    e1.set_hint_timing(duel, 0, TIMING_MAIN_END);
    e1.set_condition(duel, |duel, _| duel.is_main_phase());
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, effcon);
    e2.set_target(duel, efftg);
    e2.set_operation(duel, effop);
    c.register_effect(duel, e2);
}

fn matcheck(duel: &Duel, _: CardId, tp: u8, sg: CardSet, fc: CardId) -> bool {
    sg.is_exists(
        duel,
        |c, duel| c.is_race_full(duel, RACE_BEAST | RACE_FIEND, Some(fc), SUMMON_TYPE_FUSION.into(), tp),
        1,
        None,
    )
}

fn effcon(duel: &Duel, a: &Args) -> bool {
    let filter = |c: CardId, duel: &Duel| c.is_faceup(duel) && c.is_code(duel, CARD_CHIMERA_MYTHICAL_BEAST);
    duel.is_existing_matching_card(filter, a.tp, LOCATION_ONFIELD | LOCATION_GRAVE, 0, 1, None)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, code: u32) -> bool {
    c.is_code(duel, code) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let exists = |duel: &Duel, code| {
        duel.is_existing_matching_card(
            |c, duel| spfilter(c, duel, e, tp, code),
            tp,
            LOCATION_DECK | LOCATION_GRAVE,
            0,
            1,
            None,
        )
    };
    let b1 = c.is_able_to_hand(duel);
    let b2 = c.is_able_to_remove(duel)
        && duel.get_location_count(tp, LOCATION_MZONE) > 1
        && exists(duel, GAZELLE)
        && exists(duel, BERFOMET)
        && duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none();
    if chk == 0 {
        return Ok(b1 || b2);
    }
    let op = duel.select_effect(tp, &[(b1, aux::stringid(ID, 2)), (b2, aux::stringid(ID, 3))])?;
    e.set_label(duel, op.map_or(0, |op| op as i64));
    if op == Some(1) {
        e.set_category(duel, CATEGORY_TOHAND);
        duel.set_operation_info(0, CATEGORY_TOHAND, Some(c.into()), 1, tp, 0);
    } else {
        e.set_category(duel, CATEGORY_REMOVE + CATEGORY_SPECIAL_SUMMON);
        duel.set_operation_info(0, CATEGORY_REMOVE, Some(c.into()), 1, tp, 0);
        duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 2, tp, (LOCATION_DECK | LOCATION_GRAVE) as i32);
    }
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    if e.get_label(duel) == 1 {
        duel.send_to_hand(c, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, c)?;
    } else if duel.remove(c, POS_FACEUP, REASON_EFFECT)? > 0
        && c.is_location(duel, LOCATION_REMOVED)
        && duel.get_location_count(tp, LOCATION_MZONE) > 1
        && duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none()
    {
        let filter = aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp, GAZELLE) || spfilter(c, duel, e, tp, BERFOMET));
        let g = duel.get_matching_group(filter, tp, LOCATION_DECK | LOCATION_GRAVE, 0, None);
        if g.get_class_count(duel, |c, duel| i64::from(c.get_code(duel))) == 2 {
            let sg = aux::select_unselect_group(duel, g, 2, 2, &aux::dncheck, tp, HINTMSG_SPSUMMON)?;
            duel.special_summon(sg, 0, tp, tp, false, false, POS_FACEUP)?;
        }
    }
    Ok(())
}
