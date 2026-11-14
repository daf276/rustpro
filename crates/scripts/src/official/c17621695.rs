// Enneapolis
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, PZONE};

const ID: u32 = 17621695;

pub const LISTED_SERIES: &[u16] = &[SET_ENNEACRAFT];
pub const COUNTER_LIST: &[u16] = &[COUNTER_EC];

// In the Field Zone and the hand, with Enneacraft monsters in the Pendulum Zones to return and face-down on the field to flip.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, 0),
    (ID, HAND),
    (28454232, PZONE),
    (82359538, PZONE),
    (54842941, MZONE),
    (54842941, MZONE),
    (81237046, MZONE),
    (81237046, MZONE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e0.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DAMAGE);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_PHASE + PHASE_END);
    e1.set_range(duel, LOCATION_FZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_target(duel, damtg);
    e1.set_operation(duel, damop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_FZONE);
    e2.set_count_limit(duel, 1, (ID, 0));
    e2.set_target(duel, rthtg);
    e2.set_operation(duel, rthop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_TOHAND);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_CUSTOM + ID);
    e3.set_range(duel, LOCATION_FZONE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_target(duel, thpztg);
    e3.set_operation(duel, thpzop);
    c.register_effect(duel, e3);
    e3.set_label_object_group(duel, Some(CardSet::default()));
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e4.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e4.set_code(duel, EVENT_FLIP);
    e4.set_range(duel, LOCATION_FZONE);
    e4.set_label_object(duel, Some(e3));
    e4.set_condition(duel, |duel, _| duel.is_main_phase());
    e4.set_operation(duel, regop);
    c.register_effect(duel, e4);
}

fn damtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let tp = a.tp;
    let ct = duel.get_counter(tp, 1, 0, COUNTER_EC);
    duel.set_target_player(1 - tp);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, (ct * 900) as i32);
    Ok(false)
}

fn damop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_matching_group(|c, duel| c.has_counter(duel, COUNTER_EC), tp, LOCATION_ONFIELD, 0, None);
    if g.is_empty() {
        return Ok(());
    }
    let mut total_count = 0;
    for sc in g {
        let sc_count = sc.get_counter(duel, COUNTER_EC);
        if sc.remove_counter(duel, tp, COUNTER_EC, sc_count, REASON_EFFECT)? {
            total_count += u32::from(sc_count);
        }
    }
    if total_count > 0 {
        let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
        duel.damage(p, total_count * 900, REASON_EFFECT)?;
    }
    Ok(())
}

fn rthfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_ENNEACRAFT]) && c.is_original_type(duel, TYPE_PENDULUM) && c.is_faceup(duel) && c.is_able_to_hand(duel)
}

fn rthtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_controler(duel, tp) && rthfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(rthfilter, tp, LOCATION_ONFIELD, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
    let g = duel.select_target(tp, rthfilter, tp, LOCATION_ONFIELD, 0, 1, 99, None)?;
    let count = g.map_or(0, |g| g.len() as u8);
    duel.set_operation_info(0, CATEGORY_TOHAND, g, count, tp, 0);
    Ok(false)
}

fn rthop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tg) = duel.get_target_cards(a.e)
        && !tg.is_empty()
    {
        duel.send_to_hand(tg, PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}

fn thpzfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_ENNEACRAFT])
        && c.is_type(duel, TYPE_PENDULUM)
        && c.is_location(duel, LOCATION_MZONE)
        && c.is_controler(duel, tp)
        && (c.is_able_to_hand(duel) || (!c.is_forbidden(duel) && duel.check_pendulum_zones(tp)))
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let tg =
        a.eg.map_or_else(CardSet::default, |eg| eg.filter(duel, |c, duel| thpzfilter(c, duel, tp), None));
    if tg.is_empty() {
        return Ok(());
    }
    for tc in tg {
        tc.register_flag_effect(duel, ID, RESET_CHAIN, 0, 1);
    }
    let Some(e3) = a.e.get_label_object(duel) else { return Ok(()) };
    let mut g = e3.get_label_object_group(duel).unwrap_or_default();
    if duel.get_current_chain() == 0 {
        g.clear();
    }
    g.merge(tg);
    g.remove_matching(duel, |c, duel| c.get_flag_effect(duel, ID) == 0, None);
    e3.set_label_object_group(duel, Some(g));
    let c = a.e.get_handler(duel);
    duel.raise_single_event(c, EVENT_CUSTOM + ID, Some(a.e), 0, tp, tp, 0)
}

fn thpztg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let g =
        a.e.get_label_object_group(duel)
            .unwrap_or_default()
            .filter(duel, |c, duel| thpzfilter(c, duel, tp), None);
    if chk == 0 {
        return Ok(!g.is_empty());
    }
    duel.set_target_card(g);
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, Some(g), 1, tp, 0);
    Ok(false)
}

fn thpzop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel
        .get_target_cards(a.e)
        .unwrap_or_default()
        .filter(duel, |c, duel| thpzfilter(c, duel, tp), None);
    if g.is_empty() {
        return Ok(());
    }
    let sc = if g.len() == 1 {
        g.first()
    } else {
        duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 3));
        g.select(duel, tp, 1, 1, None)?.first()
    };
    let Some(sc) = sc else { return Ok(()) };
    duel.hint_selection(sc);
    let b1 = sc.is_able_to_hand(duel);
    let b2 = duel.check_pendulum_zones(tp) && !sc.is_forbidden(duel);
    match duel.select_effect(tp, &[(b1, aux::stringid(ID, 4)), (b2, aux::stringid(ID, 5))])? {
        Some(1) => {
            duel.send_to_hand(sc, PLAYER_NONE, REASON_EFFECT)?;
        }
        Some(2) => {
            duel.move_to_field(sc, tp, tp, LOCATION_PZONE, POS_FACEUP, true)?;
        }
        _ => {}
    }
    Ok(())
}
