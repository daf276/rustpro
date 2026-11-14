// Eidos the Underworld Monarch
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE};

const ID: u32 = 31596518;

pub const LISTED_SERIES: &[u16] = &[SET_MONARCH];

// Mobius the Frost Monarch to Tribute Summon with Sangan as the Tribute, Kashtira Ogre in the Deck to add.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, HAND),
    (ID, GRAVE),
    (4929256, HAND),
    (4929256, HAND),
    (26202165, MZONE),
    (26202165, MZONE),
    (94392192, DECK),
    (94392192, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_target(duel, attrtg);
    e3.set_operation(duel, attrop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 2));
    e4.set_category(duel, CATEGORY_TOHAND + CATEGORY_SPECIAL_SUMMON);
    e4.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e4.set_property_full(duel, EFFECT_FLAG_DELAY, EFFECT_FLAG2_CHECK_SIMULTANEOUS);
    e4.set_code(duel, EVENT_SUMMON_SUCCESS);
    e4.set_range(duel, LOCATION_GRAVE);
    e4.set_count_limit(duel, 1, (ID, 2));
    e4.set_condition(duel, thspcon);
    e4.set_target(duel, thsptg);
    e4.set_operation(duel, thspop);
    c.register_effect(duel, e4);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    ((c.is_set_card(duel, &[SET_MONARCH]) && c.is_spell_trap(duel)) || (c.is_attack(duel, &[2800]) && c.is_defense(duel, &[1000])))
        && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK | LOCATION_GRAVE, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, aux::necro_valley_filter(thfilter), LOCATION_DECK | LOCATION_GRAVE)
}

fn attrtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let g = duel.get_matching_group(CardId::is_faceup, a.tp, LOCATION_MZONE, LOCATION_MZONE, None);
    let attr = duel.announce_another_attribute(g, a.tp)?;
    a.e.set_label(duel, i64::from(attr));
    Ok(false)
}

fn attrop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let attr = a.e.get_label(duel) as u32;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_APPLYTO);
    let filter = |c: CardId, duel: &Duel| c.is_faceup(duel) && c.is_attribute_except(duel, attr);
    let Some(sc) = duel
        .select_matching_card(tp, filter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?
        .get_first()
    else {
        return Ok(());
    };
    duel.hint_selection(sc);
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e1.set_code(duel, EFFECT_CHANGE_ATTRIBUTE);
    e1.set_value(duel, attr as i32);
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
    sc.register_effect(duel, e1);
    Ok(())
}

fn thspcon(duel: &Duel, a: &Args) -> bool {
    a.eg.and_then(|eg| eg.get_first()).is_some_and(|ec| {
        ec.is_summon_player(duel, a.tp)
            && ec.is_tribute_summoned(duel)
            && ec.is_attack_above(duel, 2400)
            && ec.is_defense(duel, &[1000])
            && ec.is_faceup(duel)
    })
}

fn thsptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_able_to_hand(duel)
            || (duel.get_location_count(tp, LOCATION_MZONE) > 0 && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)));
    }
    duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, Some(c.into()), 1, tp, 0);
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, Some(c.into()), 1, tp, 0);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    Ok(false)
}

fn thspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    aux::to_hand_or_else(
        duel,
        c,
        tp,
        |sc, duel| duel.get_location_count(tp, LOCATION_MZONE) > 0 && sc.is_can_be_special_summoned(duel, e, 0, tp, false, false),
        |duel| duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP).map(drop),
        aux::stringid(ID, 3),
    )
}
