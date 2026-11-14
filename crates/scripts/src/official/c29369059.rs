// Yummy☆Surprise
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 29369059;

pub const LISTED_SERIES: &[u16] = &[SET_YUMMY];
// Yummy monsters (LIGHT Beasts) on the field and in the hand and Graveyard, Yummyusment☆Mignon in the hand and Graveyard as a Field
// Spell to return and place, and opponent's cards to return.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (31425736, MZONE),
    (68810435, MZONE),
    (4215180, HAND),
    (10966439, GRAVE),
    (66975205, GRAVE),
    (66975205, HAND),
    (53129443, SZONE | OPPONENT),
    (53129443, SZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(
        duel,
        0,
        TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMING_BATTLE_START | TIMINGS_CHECK_MONSTER_E,
    );
    e1.set_cost(duel, effcost);
    e1.set_target(duel, efftg);
    e1.set_operation(duel, effop);
    c.register_effect(duel, e1);
}

fn monoppthfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    ((c.is_attribute(duel, ATTRIBUTE_LIGHT) && c.is_race(duel, RACE_BEAST) && c.is_controler(duel, tp) && c.is_faceup(duel))
        || c.is_controler(duel, 1 - tp))
        && c.is_can_be_effect_target(duel, e)
        && c.is_able_to_hand(duel)
}

fn rescon(tp: u8) -> impl Fn(&Duel, CardSet, CardSet, Option<CardId>) -> (bool, bool) {
    move |duel, sg, _, _| (sg.filter_count(duel, |c, duel| c.is_controler(duel, tp), None) == 2, false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_YUMMY]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn fieldspellthfilter(c: CardId, duel: &Duel) -> bool {
    c.is_field_spell(duel) && c.is_faceup(duel) && c.is_able_to_hand(duel)
}

// The returnable cards and which of the three effects can be chosen, as the cost and the target both compute them.
fn options(duel: &Duel, e: EffectId, tp: u8) -> (CardSet, [bool; 3]) {
    let g = duel.get_matching_group(|c, duel| monoppthfilter(c, duel, e, tp), tp, LOCATION_MZONE, LOCATION_ONFIELD, None);
    let b1 = !duel.has_flag_effect(tp, ID) && g.len() >= 4 && aux::select_unselect_group_check(duel, g, 4, 4, &rescon(tp));
    let b2 = !duel.has_flag_effect(tp, ID + 1)
        && duel.get_location_count(tp, LOCATION_MZONE) > 0
        && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_HAND | LOCATION_GRAVE, 0, 1, None);
    let b3 = !duel.has_flag_effect(tp, ID + 2)
        && duel.is_existing_matching_card(fieldspellthfilter, tp, LOCATION_ONFIELD | LOCATION_GRAVE, 0, 1, None);
    (g, [b1, b2, b3])
}

fn effcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    a.e.set_label(duel, -100);
    let (_, [b1, b2, b3]) = options(duel, a.e, a.tp);
    Ok(chk == 0 && (b1 || b2 || b3))
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chkc.is_some() {
        return Ok(false);
    }
    let cost_skip = e.get_label(duel) != -100;
    let (g, [b1, b2, b3]) = options(duel, e, tp);
    if chk == 0 {
        e.set_label(duel, 0);
        return Ok(b1 || b2 || b3);
    }
    let op = duel.select_effect(
        tp,
        &[(b1, aux::stringid(ID, 1)), (b2, aux::stringid(ID, 2)), (b3, aux::stringid(ID, 3))],
    )?;
    e.set_label(duel, op.map_or(0, |op| op as i64));
    match op {
        Some(1) => {
            e.set_category(duel, CATEGORY_TOHAND);
            e.set_property(duel, EFFECT_FLAG_CARD_TARGET);
            if !cost_skip {
                duel.register_flag_effect(tp, ID, RESET_PHASE | PHASE_END, 0, 1);
            }
            let tg = aux::select_unselect_group(duel, g, 4, 4, &rescon(tp), tp, HINTMSG_RTOHAND)?;
            duel.set_target_card(tg);
            duel.set_operation_info(0, CATEGORY_TOHAND, Some(tg), 4, tp, 0);
        }
        Some(2) => {
            e.set_category(duel, CATEGORY_SPECIAL_SUMMON);
            e.set_property(duel, 0);
            if !cost_skip {
                duel.register_flag_effect(tp, ID + 1, RESET_PHASE | PHASE_END, 0, 1);
            }
            duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, (LOCATION_HAND | LOCATION_GRAVE) as i32);
        }
        Some(3) => {
            e.set_category(duel, CATEGORY_TOHAND);
            e.set_property(duel, 0);
            if !cost_skip {
                duel.register_flag_effect(tp, ID + 2, RESET_PHASE | PHASE_END, 0, 1);
            }
            duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, (LOCATION_ONFIELD | LOCATION_GRAVE) as i32);
        }
        _ => {}
    }
    Ok(false)
}

fn plfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_YUMMY]) && c.is_field_spell(duel) && !c.is_forbidden(duel)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    match e.get_label(duel) {
        1 => {
            let tg = duel.get_target_cards(e).unwrap_or_default();
            if !tg.is_empty() {
                duel.send_to_hand(tg, PLAYER_NONE, REASON_EFFECT)?;
            }
        }
        2 => {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
            let filter = aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp));
            let sc = duel
                .select_matching_card(tp, filter, tp, LOCATION_HAND | LOCATION_GRAVE, 0, 1, 1, None)?
                .get_first();
            if let Some(sc) = sc
                && duel.special_summon(sc, 0, tp, tp, false, false, POS_FACEUP)? > 0
            {
                let e1 = duel.create_effect(e.get_handler(duel));
                e1.set_description(duel, 3207);
                e1.set_type(duel, EFFECT_TYPE_SINGLE);
                e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
                e1.set_code(duel, EFFECT_CANNOT_DIRECT_ATTACK);
                e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
                sc.register_effect(duel, e1);
            }
        }
        3 => {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
            let filter = aux::necro_valley_filter(fieldspellthfilter);
            let sc = duel
                .select_matching_card(tp, filter, tp, LOCATION_ONFIELD | LOCATION_GRAVE, 0, 1, 1, None)?
                .get_first();
            let Some(sc) = sc else { return Ok(()) };
            if duel.send_to_hand(sc, PLAYER_NONE, REASON_EFFECT)? > 0 && sc.is_location(duel, LOCATION_HAND) {
                duel.confirm_cards(1 - tp, sc)?;
                duel.shuffle_hand(tp)?;
                if duel.is_existing_matching_card(plfilter, tp, LOCATION_HAND, 0, 1, None)
                    && duel.select_yes_no(tp, aux::stringid(ID, 4))?
                {
                    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOFIELD);
                    let plc = duel
                        .select_matching_card(tp, plfilter, tp, LOCATION_HAND, 0, 1, 1, None)?
                        .get_first();
                    if let Some(plc) = plc {
                        duel.break_effect()?;
                        duel.place_field_spell(plc, tp)?;
                    }
                }
            }
        }
        _ => {}
    }
    Ok(())
}
