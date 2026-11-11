// Infernoble Knight Captain Roland
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, MZONE};

const ID: u32 = 40251688;

// A copy in the Extra Deck, Synchro Summoned with Junk Synchron and Atlantean Pikeman, and one in the Graveyard, with Elemental HERO
// Burstinatrix (a Warrior) on the field to equip to and in the Deck to search, and Infernoble Arms - Durendal to send.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, GRAVE),
    (63977008, MZONE),
    (26976414, MZONE),
    (58932615, MZONE),
    (58932615, DECK),
    (37478723, DECK),
    (37478723, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<super::c37675907::Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOGRAVE + CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_synchro_summoned(duel));
    e1.set_operation(duel, regop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_EQUIP);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER | TIMING_MAIN_END);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, _| duel.is_main_phase());
    e2.set_target(duel, equiptg);
    e2.set_operation(duel, equipop);
    c.register_effect(duel, e2);
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_PHASE + PHASE_END);
    e1.set_count_limit(duel, 1, 0);
    e1.set_condition(duel, tgcon);
    e1.set_operation(duel, tgop);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, a.tp);
    Ok(())
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_type(duel, TYPE_EQUIP) && c.is_spell(duel) && c.is_able_to_grave(duel)
}

fn tgcon(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(tgfilter, a.tp, LOCATION_DECK, 0, 1, None)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_WARRIOR) && c.is_able_to_hand(duel)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let gc = duel
        .select_matching_card(tp, tgfilter, tp, LOCATION_DECK, 0, 1, 1, None)?
        .get_first();
    if let Some(gc) = gc
        && duel.send_to_grave(gc, REASON_EFFECT)? > 0
        && gc.is_location(duel, LOCATION_GRAVE)
    {
        let sg = duel.get_matching_group(thfilter, tp, LOCATION_DECK, 0, None);
        if sg.is_empty() {
            return Ok(());
        }
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
        let hg = sg.select(duel, tp, 1, 1, None)?;
        duel.break_effect()?;
        duel.send_to_hand(hg, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, hg)?;
    }
    Ok(())
}

fn equiptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE)
            && chkc.is_controler(duel, tp)
            && chkc.is_faceup(duel)
            && chkc.is_race(duel, RACE_WARRIOR));
    }
    let filter = aux::faceup_filter(|c, duel| c.is_race(duel, RACE_WARRIOR));
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_SZONE) > 0 && duel.is_existing_target(&filter, tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_EQUIP);
    duel.select_target(tp, &filter, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    let c = e.get_handler(duel);
    duel.set_operation_info(0, CATEGORY_EQUIP, Some(c.into()), 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, Some(c.into()), 1, 0, 0);
    Ok(false)
}

fn equipop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) || duel.get_location_count(tp, LOCATION_SZONE) == 0 {
        return Ok(());
    }
    let tc = duel.get_first_target();
    unported!(duel, tc.is_none());
    let Some(tc) = tc else { return Ok(()) };
    if !tc.is_relate_to_effect(duel, e) || tc.is_facedown(duel) || tc.is_controler(duel, 1 - tp) {
        return Ok(());
    }
    if duel.equip(tp, c, tc)? {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_EQUIP_LIMIT);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        e1.set_value(duel, Value::Card(super::c38423248::equip_limit));
        e1.set_label_object_group(duel, Some(tc.into()));
        c.register_effect(duel, e1);
        let e2 = duel.create_effect(c);
        e2.set_type(duel, EFFECT_TYPE_EQUIP);
        e2.set_code(duel, EFFECT_UPDATE_ATTACK);
        e2.set_value(duel, 500);
        e2.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        c.register_effect(duel, e2);
    }
    Ok(())
}
