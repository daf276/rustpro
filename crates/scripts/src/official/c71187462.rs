// Blackwing - Vata the Emblem of Wandering
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE};

const ID: u32 = 71187462;

pub const LISTED_NAMES: &[u32] = &[ID, CARD_BLACK_WINGED_DRAGON];

// Black-Winged Dragon in the Extra Deck and Blackwing - Elphin the Raven (Level 6) in the Deck, to sum to Level 8 with this Level 2 Tuner,
// Blackwing - Gale the Whirlwind on the field for the Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (9012916, EXTRA),
    (9012916, EXTRA),
    (11613567, DECK),
    (11613567, DECK),
    (11613567, DECK),
    (2009101, MZONE),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_condition(duel, spcon);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TOGRAVE + CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, tgtg);
    e2.set_operation(duel, tgop);
    c.register_effect(duel, e2);
}

fn spconfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_BLACKWING]) && !c.is_code(duel, ID)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let Some(_) = a.c else { return true };
    let tp = a.e.get_handler_player(duel);
    duel.get_location_count(tp, LOCATION_MZONE) > 0 && duel.is_existing_matching_card(spconfilter, tp, LOCATION_MZONE, 0, 1, None)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, ec: CardId) -> bool {
    c.is_code(duel, CARD_BLACK_WINGED_DRAGON)
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
        && duel.get_location_count_from_ex(tp, tp, Some(ec.into()), c) > 0
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_BLACKWING]) && !c.is_type(duel, TYPE_TUNER) && c.has_level(duel) && c.is_able_to_grave(duel)
}

fn tgrescon(clv: u32) -> impl Fn(&Duel, CardSet, CardSet, Option<CardId>) -> (bool, bool) {
    move |duel, sg, _, _| {
        let sum = clv + sg.into_iter().map(|c| c.get_level(duel)).sum::<u32>();
        (sum == 8, sum > 8)
    }
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let c = e.get_handler(duel);
        if !c.is_able_to_grave(duel)
            || c.is_level_above(duel, 8)
            || !duel.is_existing_matching_card(|sc, duel| spfilter(sc, duel, e, tp, c), tp, LOCATION_EXTRA, 0, 1, None)
        {
            return Ok(false);
        }
        let g = duel.get_matching_group(tgfilter, tp, LOCATION_DECK, 0, None);
        return Ok(aux::select_unselect_group_check(duel, g, 1, g.len(), &tgrescon(c.get_level(duel))));
    }
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, LOCATION_DECK as i32);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

// The Lua's lizfilter negates before comparing, so it never matches.
fn lizfilter(_: &Duel, _: EffectId, _: CardId) -> bool {
    false
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) && c.is_able_to_grave(duel) && c.is_level_below(duel, 7) {
        let g = duel.get_matching_group(tgfilter, tp, LOCATION_DECK, 0, None);
        let rescon = tgrescon(c.get_level(duel));
        let mg = aux::select_unselect_group_full(duel, g, 1, g.len(), &rescon, tp, HINTMSG_TOGRAVE, Some(&rescon), false)?;
        let mut sg = mg;
        sg.insert(c);
        if !mg.is_empty() && duel.send_to_grave(sg, REASON_EFFECT)? > 0 {
            let og = duel.get_operated_group();
            if og.filter_count(duel, |c, duel| c.is_location(duel, LOCATION_GRAVE), None) != og.len() {
                return Ok(());
            }
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
            let sg = duel.select_matching_card(tp, |sc, duel| spfilter(sc, duel, e, tp, c), tp, LOCATION_EXTRA, 0, 1, 1, None)?;
            if !sg.is_empty() {
                duel.special_summon(sg, 0, tp, tp, false, false, POS_FACEUP)?;
            }
        }
    }
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_card(duel, |duel, _, c| {
        c.is_location(duel, LOCATION_EXTRA) && !c.is_attribute(duel, ATTRIBUTE_DARK)
    });
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    aux::add_temp_lizard_check_full(duel, c, tp, lizfilter);
    Ok(())
}
