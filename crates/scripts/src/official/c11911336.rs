// Spellbook of the Grand Circle
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND, MZONE};

const ID: u32 = 11911336;

pub const LISTED_SERIES: &[u16] = &[SET_CHARMER, SET_FAMILIAR_POSSESSED];

const CHARMERS_OF_THE_GRAND_CIRCLE: u32 = 48016074;

struct Params;

impl FusionSummonParams for Params {
    fn matfilter(c: CardId, duel: &Duel, _: EffectId, _: u8) -> bool {
        c.is_set_card(duel, &[SET_CHARMER, SET_FAMILIAR_POSSESSED])
    }

    fn stage2(duel: &mut Duel, e: EffectId, _: Option<CardId>, tp: u8, _: Option<CardSet>, chk: i32) -> Script<()> {
        if chk == 1 {
            let e1 = duel.create_effect(e.get_handler(duel));
            e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
            e1.set_code(duel, EVENT_CHAIN_END);
            e1.set_operation(duel, |duel, a| {
                duel.set_chain_limit_till_chain_end(|_, _, rp, tp, _| rp == tp, 0);
                a.e.reset(duel);
                Ok(())
            });
            duel.register_effect(e1, tp);
        }
        Ok(())
    }
}

// Four Charmers of different Attributes in the Deck to search; Charmers in the hand and field for the Graveyard effect to Fuse.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, GRAVE),
    (CHARMERS_OF_THE_GRAND_CIRCLE, EXTRA),
    (CHARMERS_OF_THE_GRAND_CIRCLE, EXTRA),
    (759393, DECK),
    (19327348, DECK),
    (37744402, DECK),
    (37970940, DECK),
    (73318863, DECK),
    (74364659, DECK),
    (759393, HAND),
    (19327348, MZONE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::SpecialSummonedCode(CHARMERS_OF_THE_GRAND_CIRCLE),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_TODECK);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_FUSION_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, Fusion::summon_eff_tg::<Params>);
    e2.set_operation(duel, Fusion::summon_eff_op::<Params>);
    c.register_effect(duel, e2);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_CHARMER])
        && c.is_monster(duel)
        && (c.is_location(duel, LOCATION_DECK) || c.is_faceup(duel))
        && c.is_able_to_hand(duel)
}

const LOCATIONS: u32 = LOCATION_DECK | LOCATION_GRAVE | LOCATION_REMOVED;

fn attribute_check() -> impl Fn(&Duel, CardSet, CardSet, Option<CardId>) -> (bool, bool) {
    aux::dpcheck(|c, duel| i64::from(c.get_attribute(duel)))
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        let g = duel.get_matching_group(thfilter, tp, LOCATIONS, 0, None);
        return Ok(aux::select_unselect_group_check(duel, g, 4, 4, &attribute_check()));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATIONS as i32);
    duel.set_operation_info(0, CATEGORY_TODECK, None, 2, tp, LOCATION_HAND as i32);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_matching_group(aux::necro_valley_filter(thfilter), tp, LOCATIONS, 0, None);
    if g.len() < 4 {
        return Ok(());
    }
    let sg = aux::select_unselect_group(duel, g, 4, 4, &attribute_check(), tp, HINTMSG_ATOHAND)?;
    if sg.len() == 4 && duel.send_to_hand(sg, PLAYER_NONE, REASON_EFFECT)? == 4 {
        duel.confirm_cards(1 - tp, sg)?;
        duel.shuffle_hand(tp)?;
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
        let tdg = duel.select_matching_card(tp, CardId::is_able_to_deck, tp, LOCATION_HAND, 0, 2, 2, None)?;
        if tdg.len() == 2 {
            duel.break_effect()?;
            duel.send_to_deck(tdg, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)?;
        }
    }
    Ok(())
}
