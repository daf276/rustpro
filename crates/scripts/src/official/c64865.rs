// Cyberse Code Magician
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 64865;

pub const LISTED_NAMES: &[u32] = &[34767865];
// In the hand as extra material for Code Talker Inverted with Link Spider on the field, sent to the Graveyard by the opponent's Dark Hole
// with a Cyberse monster in the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, MZONE),
    (98978921, MZONE),
    (98978921, MZONE),
    (45462149, EXTRA),
    (45462149, EXTRA),
    (24154052, DECK),
    (24154052, DECK),
    (53129443, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::MaterialFrom(HAND)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_code(duel, EFFECT_EXTRA_MATERIAL);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_target_range(duel, 1, 0);
    e1.set_extra_material_operation(duel, extracon);
    e1.set_value(duel, Value::ExtraMaterial(extraval));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TOGRAVE + CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, |duel, a| {
        a.e.get_handler(duel).is_previous_location(duel, LOCATION_HAND | LOCATION_ONFIELD)
    });
    e2.set_target(duel, tgsptg);
    e2.set_operation(duel, tgspop);
    c.register_effect(duel, e2);
}

fn extracon(duel: &mut Duel, _: CardId, e: EffectId, _: u8, sg: CardSet, mg: CardSet, _: CardId, og: CardSet, _: i32) -> bool {
    let player = e.get_handler_player(duel);
    let extrafilter = |c: CardId, duel: &Duel| c.is_location(duel, LOCATION_MZONE) && c.is_controler(duel, player);
    sg.union(mg)
        .filter(duel, extrafilter, None)
        .is_exists(duel, CardId::is_link_monster, 1, Some(og))
        && sg.filter_count(duel, |c, duel| c.has_flag_effect(duel, ID), None) < 2
}

// The flags chk 0 registers (s.flagmap1) are the card's only flags of this code, so chk 2 resets them by the code.
fn extraval(duel: &mut Duel, e: EffectId, call: ExtraMaterialCall) -> CardSet {
    let c = e.get_handler(duel);
    match call {
        ExtraMaterialCall::Get { summon_type, tp, sc } => {
            if summon_type != SUMMON_TYPE_LINK || !sc.is_race(duel, RACE_CYBERSE) || duel.has_flag_effect(tp, ID) {
                return CardSet::default();
            }
            c.register_flag_effect(duel, ID, 0, 0, 1);
            c.into()
        }
        ExtraMaterialCall::Use { summon_type, sg, tp, .. } => {
            if summon_type & SUMMON_TYPE_LINK == SUMMON_TYPE_LINK && !sg.is_empty() {
                duel.hint(HINT_CARD, tp, u64::from(ID));
                duel.register_flag_effect(tp, ID, RESET_PHASE | PHASE_END, 0, 1);
            }
            CardSet::default()
        }
        ExtraMaterialCall::Delete { .. } => {
            c.reset_flag_effect(duel, ID);
            CardSet::default()
        }
    }
}

fn tgspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, mmz_ritsum_chk: bool) -> bool {
    c.is_race(duel, RACE_CYBERSE)
        && (c.is_able_to_grave(duel) || (mmz_ritsum_chk && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)))
}

fn tgsptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let ritual_summoned_chk = e.get_handler(duel).is_ritual_summoned(duel);
    if chk == 0 {
        let mmz_ritsum_chk = ritual_summoned_chk && duel.get_location_count(tp, LOCATION_MZONE) > 0;
        return Ok(duel.is_existing_matching_card(|c, duel| tgspfilter(c, duel, e, tp, mmz_ritsum_chk), tp, LOCATION_DECK, 0, 1, None));
    }
    if !ritual_summoned_chk {
        e.set_label(duel, 0);
        e.set_category(duel, CATEGORY_TOGRAVE);
        duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, LOCATION_DECK as i32);
    } else {
        e.set_label(duel, 1);
        e.set_category(duel, CATEGORY_TOGRAVE + CATEGORY_SPECIAL_SUMMON);
        duel.set_possible_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, LOCATION_DECK as i32);
        duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
    }
    Ok(false)
}

fn tgspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let mmz_ritsum_chk = e.get_label(duel) == 1 && duel.get_location_count(tp, LOCATION_MZONE) > 0;
    let hint = if mmz_ritsum_chk { aux::stringid(ID, 1) } else { HINTMSG_TOGRAVE };
    duel.hint(HINT_SELECTMSG, tp, hint);
    let g = duel.select_matching_card(
        tp,
        |c, duel| tgspfilter(c, duel, e, tp, mmz_ritsum_chk),
        tp,
        LOCATION_DECK,
        0,
        1,
        1,
        None,
    )?;
    if let Some(sc) = g.get_first() {
        let op = if mmz_ritsum_chk {
            let b1 = sc.is_able_to_grave(duel);
            let b2 = sc.is_can_be_special_summoned(duel, e, 0, tp, false, false);
            duel.select_effect(tp, &[(b1, aux::stringid(ID, 2)), (b2, aux::stringid(ID, 3))])?
        } else {
            Some(1)
        };
        match op {
            Some(1) => {
                duel.send_to_grave(sc, REASON_EFFECT)?;
            }
            Some(2) => {
                duel.special_summon(sc, 0, tp, tp, false, false, POS_FACEUP)?;
            }
            _ => {}
        }
    }
    let c = e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 4));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_card(duel, |duel, _, c| c.is_race_except(duel, RACE_CYBERSE));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    Ok(())
}
