// Radiance of the Voiceless Voice
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, SZONE};

const ID: u32 = 86310763;

pub const LISTED_SERIES: &[u16] = &[SET_VOICELESS_VOICE];

// Set, with Prayers of the Voiceless Voice in the Graveyard to shuffle and Voiceless Voice monsters in the Deck to take,
// and Saffira, Queen of Dragons on the field to destroy as many of the opponent's cards.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, SZONE),
    (52472775, GRAVE),
    (52472775, HAND),
    (25801745, DECK),
    (51296484, DECK),
    (56350972, MZONE),
    (10774240, MZONE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e0.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_SZONE);
    e1.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, _| duel.is_main_phase());
    e1.set_target(duel, efftg);
    e1.set_operation(duel, effop);
    c.register_effect(duel, e1);
}

fn todeckfilter(c: CardId, duel: &Duel) -> bool {
    c.is_able_to_deck(duel)
        && (c.is_ritual_spell(duel)
            || (c.is_attribute(duel, ATTRIBUTE_LIGHT) && c.is_race(duel, RACE_WARRIOR | RACE_DRAGON) && c.is_ritual_monster(duel)))
}

fn silenforcfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, ft: i32) -> bool {
    c.is_set_card(duel, &[SET_VOICELESS_VOICE])
        && c.is_monster(duel)
        && (c.is_able_to_hand(duel) || (ft > 0 && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)))
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_attribute(duel, ATTRIBUTE_LIGHT) && c.is_race(duel, RACE_WARRIOR | RACE_DRAGON) && c.is_ritual_monster(duel)
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_controler(duel, 1 - tp));
    }
    let ft = duel.get_location_count(tp, LOCATION_MZONE);
    let b1 = duel.is_existing_matching_card(todeckfilter, tp, LOCATION_HAND | LOCATION_GRAVE, 0, 1, None)
        && duel.is_existing_matching_card(|c, duel| silenforcfilter(c, duel, e, tp, ft), tp, LOCATION_DECK, 0, 1, None);
    let c = e.get_handler(duel);
    let ct = duel.get_matching_group_count(cfilter, tp, LOCATION_MZONE, 0, None);
    let b2 = ct > 0 && duel.is_existing_target(aux::TRUE, tp, 0, LOCATION_ONFIELD, 1, None);
    if chk == 0 {
        return Ok(b1 || b2);
    }
    let op = duel.select_effect(tp, &[(b1, aux::stringid(ID, 1)), (b2, aux::stringid(ID, 2))])?;
    e.set_label(duel, op.map_or(0, |op| op as i64));
    match op {
        Some(1) => {
            e.set_category(duel, CATEGORY_TODECK + CATEGORY_SEARCH + CATEGORY_TOHAND + CATEGORY_SPECIAL_SUMMON);
            e.set_property(duel, 0);
            duel.set_operation_info(0, CATEGORY_TODECK, None, 1, tp, (LOCATION_HAND | LOCATION_GRAVE) as i32);
            duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
            duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
        }
        Some(2) => {
            e.set_category(duel, CATEGORY_DESTROY);
            e.set_property(duel, EFFECT_FLAG_CARD_TARGET);
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
            let max = u8::try_from(ct).unwrap_or(u8::MAX);
            let mut g = duel
                .select_target(tp, aux::TRUE, tp, 0, LOCATION_ONFIELD, 1, max, None)?
                .unwrap_or_default();
            g.insert(c);
            duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), g.len() as u8, tp, 0);
        }
        _ => {}
    }
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    match e.get_label(duel) {
        1 => {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
            let filter = aux::necro_valley_filter(todeckfilter);
            let Some(tdc) = duel
                .select_matching_card(tp, filter, tp, LOCATION_HAND | LOCATION_GRAVE, 0, 1, 1, None)?
                .get_first()
            else {
                return Ok(());
            };
            if tdc.is_location(duel, LOCATION_GRAVE) {
                duel.hint_selection_full(tdc, true);
            } else {
                duel.confirm_cards(1 - tp, tdc)?;
            }
            if duel.send_to_deck(tdc, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)? > 0 && tdc.is_location(duel, LOCATION_DECK) {
                let ft = duel.get_location_count(tp, LOCATION_MZONE);
                duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 3));
                let filter = |c, duel: &Duel| silenforcfilter(c, duel, e, tp, ft);
                if let Some(sc) = duel.select_matching_card(tp, filter, tp, LOCATION_DECK, 0, 1, 1, None)?.get_first() {
                    aux::to_hand_or_else(
                        duel,
                        sc,
                        tp,
                        |sc, duel| ft > 0 && sc.is_can_be_special_summoned(duel, e, 0, tp, false, false),
                        |duel| duel.special_summon(sc, 0, tp, tp, false, false, POS_FACEUP).map(drop),
                        aux::stringid(ID, 4),
                    )?;
                }
            }
        }
        2 => {
            let c = e.get_handler(duel);
            if !c.is_relate_to_effect(duel, e) {
                return Ok(());
            }
            let mut g = duel.get_target_cards(e).unwrap_or_default();
            if g.is_empty() {
                return Ok(());
            }
            g.insert(c);
            duel.destroy(g, REASON_EFFECT)?;
        }
        _ => {}
    }
    Ok(())
}
