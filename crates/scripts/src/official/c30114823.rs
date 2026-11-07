// Code Generator
use crate::prelude::*;

const ID: u32 = 30114823;

pub fn initial_effect(duel: &mut Duel, c: CardId) {
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
    e2.set_category(duel, CATEGORY_TOGRAVE + CATEGORY_SEARCH + CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_BE_MATERIAL);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, tgcon);
    e2.set_target(duel, tgtg);
    e2.set_operation(duel, tgop);
    c.register_effect(duel, e2);
}

fn extracon(duel: &Duel, _c: CardId, e: EffectId, _tp: u8, sg: CardSet, mg: CardSet, _lc: CardId, og: CardSet, _chk: i32) -> bool {
    let player = e.get_handler_player(duel);
    let extrafilter = |c: CardId, duel: &Duel| c.is_location(duel, LOCATION_MZONE) && c.is_controler(duel, player);
    sg.union(mg)
        .filter(duel, extrafilter, None)
        .is_exists(duel, |c, duel| c.is_race(duel, RACE_CYBERSE), 1, Some(og))
        && sg.filter_count(duel, |c, duel| c.has_flag_effect(duel, ID), None) < 2
}

// The flags chk 0 registers (s.flagmap) are the card's only flags of this code, so chk 2 resets them by the code.
fn extraval(duel: &mut Duel, e: EffectId, call: ExtraMaterialCall) -> CardSet {
    let c = e.get_handler(duel);
    match call {
        ExtraMaterialCall::Get { summon_type, tp, sc } => {
            if summon_type != SUMMON_TYPE_LINK || !sc.is_set_card(duel, &[SET_CODE_TALKER]) || duel.has_flag_effect(tp, ID) {
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

fn tgcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_location(duel, LOCATION_GRAVE)
        && c.is_previous_location(duel, LOCATION_HAND | LOCATION_ONFIELD)
        && a.r == REASON_LINK
        && c.get_reason_card(duel).is_some_and(|rc| rc.is_set_card(duel, &[SET_CODE_TALKER]))
}

fn tgfilter(c: CardId, duel: &Duel, tohand_chk: i64) -> bool {
    c.is_race(duel, RACE_CYBERSE)
        && c.is_attack_below(duel, 1200)
        && (c.is_able_to_grave(duel) || (tohand_chk == 1 && c.is_able_to_hand(duel)))
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let tohand_chk = e.get_handler(duel).is_previous_location(duel, LOCATION_ONFIELD);
    if chk == 0 {
        // The Lua passes the boolean, which is never == 1.
        return Ok(duel.is_existing_matching_card(|c, duel| tgfilter(c, duel, 0), tp, LOCATION_DECK, 0, 1, None));
    }
    e.set_label(duel, i64::from(tohand_chk));
    if !tohand_chk {
        duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, LOCATION_DECK as i32);
    } else {
        duel.set_possible_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, LOCATION_DECK as i32);
        duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    }
    Ok(false)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let label = a.e.get_label(duel);
    let tohand_chk = label == 1;
    duel.hint(HINT_SELECTMSG, tp, if tohand_chk { aux::stringid(ID, 1) } else { HINTMSG_TOGRAVE });
    let filter = |c, duel: &Duel| tgfilter(c, duel, label);
    let Some(sc) = duel.select_matching_card(tp, filter, tp, LOCATION_DECK, 0, 1, 1, None)?.get_first() else {
        return Ok(());
    };
    if tohand_chk {
        let send = |duel: &mut Duel| duel.send_to_grave(sc, REASON_EFFECT).map(|_| ());
        aux::to_hand_or_else(duel, sc, tp, |c, duel| c.is_able_to_grave(duel), send, 574)
    } else {
        duel.send_to_grave(sc, REASON_EFFECT)?;
        Ok(())
    }
}
