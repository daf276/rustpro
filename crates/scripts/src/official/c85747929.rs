// Galaxy Photon Dragon
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE};

const ID: u32 = 85747929;

pub const LISTED_SERIES: &[u16] = &[SET_PHOTON, SET_GALAXY];
// Xyz Summoned from the Extra Deck with two Level 4 LIGHT monsters (Alexandrite Dragon), with Galaxy-Eyes Photon Dragon in the Deck to
// search and Galaxy Soldier in hand to Special Summon a LIGHT monster.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (43096270, MZONE),
    (43096270, MZONE),
    (43096270, MZONE),
    (43096270, MZONE),
    (93717133, DECK),
    (93717133, DECK),
    (46659709, HAND),
    (46659709, HAND),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(4);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, LOCATION_MZONE, 0);
    e1.set_target_card(duel, |duel, e, c| c.is_attribute(duel, ATTRIBUTE_LIGHT) && c != e.get_handler(duel));
    e1.set_value(duel, 500);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_TOGRAVE);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e2.set_target(duel, target);
    e2.set_operation(duel, operation);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_LVCHANGE);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_CUSTOM + ID);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_target(duel, lvtg);
    e3.set_operation(duel, lvop);
    c.register_effect(duel, e3);
    e3.set_label_object_group(duel, Some(CardSet::default()));
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e4.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e4.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_label_object(duel, Some(e3));
    e4.set_operation(duel, regop);
    c.register_effect(duel, e4);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_PHOTON, SET_GALAXY]) && (c.is_able_to_hand(duel) || c.is_able_to_grave(duel))
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(filter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    duel.set_possible_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    if let Some(tc) = duel.select_matching_card(tp, filter, tp, LOCATION_DECK, 0, 1, 1, None)?.get_first() {
        aux::to_hand_or_else(
            duel,
            tc,
            tp,
            CardId::is_able_to_grave,
            |duel| duel.send_to_grave(tc, REASON_EFFECT).map(drop),
            574,
        )?;
    }
    Ok(())
}

fn cfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_faceup(duel)
        && c.is_attribute(duel, ATTRIBUTE_LIGHT)
        && c.has_level(duel)
        && c.is_controler(duel, tp)
        && c.is_location(duel, LOCATION_MZONE)
        && c.is_can_be_effect_target(duel, e)
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let tg = a.eg.unwrap_or_default().filter(duel, |c, duel| cfilter(c, duel, e, tp), None);
    if tg.is_empty() {
        return Ok(());
    }
    for tc in tg {
        tc.register_flag_effect(duel, ID, RESET_CHAIN, 0, 1);
    }
    let Some(e3) = e.get_label_object(duel) else { return Ok(()) };
    let mut g = e3.get_label_object_group(duel).unwrap_or_default();
    if duel.get_current_chain() == 0 {
        g = CardSet::default();
    }
    g.merge(tg);
    g.remove_matching(duel, |c, duel| c.get_flag_effect(duel, ID) == 0, None);
    e3.set_label_object_group(duel, Some(g));
    duel.raise_single_event(e.get_handler(duel), EVENT_CUSTOM + ID, Some(e), 0, tp, tp, 0)?;
    Ok(())
}

fn lvtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let g = e
        .get_label_object_group(duel)
        .unwrap_or_default()
        .filter(duel, |c, duel| cfilter(c, duel, e, tp), None);
    if let Some(chkc) = chkc {
        return Ok(g.is_contains(chkc) && cfilter(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(!g.is_empty());
    }
    let tc = if g.len() == 1 {
        g
    } else {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
        g.select(duel, tp, 1, 1, None)?
    };
    duel.set_target_card(tc);
    duel.set_operation_info(0, CATEGORY_LVCHANGE, Some(tc), 1, tp, LOCATION_MZONE as i32);
    Ok(false)
}

fn lvop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_faceup(duel) && tc.is_relate_to_effect(duel, e) && tc.has_level(duel) {
        let level = u64::from(tc.get_level(duel));
        let lv = duel.announce_number(tp, &[4, 8].into_iter().filter(|&n| n != level).collect::<Vec<_>>())?;
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_code(duel, EFFECT_CHANGE_LEVEL);
        e1.set_value(duel, lv as i32);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        tc.register_effect(duel, e1);
    }
    Ok(())
}
