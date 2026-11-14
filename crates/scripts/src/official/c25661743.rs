// Prohibited Power Patron Portal - Terminus
use crate::prelude::*;

const ID: u32 = 25661743;

pub const LISTED_SERIES: &[u16] = &[SET_POWER_PATRON];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOGRAVE + CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_REMOVE);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_BATTLE_START);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, bancon);
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, bantg);
    e2.set_operation(duel, banop);
    c.register_effect(duel, e2);
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_POWER_PATRON]) && c.is_monster(duel) && c.is_able_to_grave(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(tgfilter, tp, LOCATION_DECK | LOCATION_EXTRA, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, (LOCATION_DECK | LOCATION_EXTRA) as i32);
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_attribute(duel, ATTRIBUTE_DARK) && c.is_race(duel, RACE_FAIRY) && c.is_able_to_hand(duel)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let sc = duel
        .select_matching_card(tp, tgfilter, tp, LOCATION_DECK | LOCATION_EXTRA, 0, 1, 1, None)?
        .get_first();
    if let Some(sc) = sc
        && duel.send_to_grave(sc, REASON_EFFECT)? > 0
        && sc.is_location(duel, LOCATION_GRAVE)
        && duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None)
        && duel.select_yes_no(tp, aux::stringid(ID, 2))?
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
        let sg = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
        if !sg.is_empty() {
            duel.break_effect()?;
            duel.send_to_hand(sg, PLAYER_NONE, REASON_EFFECT)?;
            duel.confirm_cards(1 - tp, sg)?;
        }
    }
    let c = a.e.get_handler(duel);
    aux::register_client_hint(duel, c, 0, tp, 1, 0, Some(aux::stringid(ID, 3)));
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    e1.set_code(duel, EFFECT_CANNOT_ATTACK_ANNOUNCE);
    e1.set_target_range(duel, LOCATION_MZONE, 0);
    e1.set_target_card(duel, |duel, _, c| !c.is_set_card(duel, &[SET_POWER_PATRON]));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    Ok(())
}

fn bancon(duel: &Duel, a: &Args) -> bool {
    let (Some(a), Some(b)) = duel.get_battle_monster(a.tp) else {
        return false;
    };
    a.is_set_card(duel, &[SET_POWER_PATRON]) && a.is_faceup(duel) && b.is_able_to_remove_full(duel, None, POS_FACEDOWN, REASON_EFFECT)
}

fn bantg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let (_, b) = duel.get_battle_monster(a.tp);
    duel.set_operation_info(0, CATEGORY_REMOVE, b.map(CardSet::from), 1, a.tp, 0);
    Ok(false)
}

fn banop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let (_, b) = duel.get_battle_monster(tp);
    if let Some(b) = b
        && b.is_relate_to_battle(duel)
        && b.is_controler(duel, 1 - tp)
    {
        duel.remove(b, POS_FACEDOWN, REASON_EFFECT)?;
    }
    Ok(())
}
