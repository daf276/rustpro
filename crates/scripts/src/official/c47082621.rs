// Regenesis Overlord
use crate::prelude::*;
use crate::testing::{Check, GRAVE, MZONE, OPPONENT};

const ID: u32 = 47082621;

pub const LISTED_NAMES: &[u32] = &[ID];

const DARK_MAGICIAN: u32 = 36996508;
const WOODBORG_INPACHI: u32 = 35322812;

// In the GY with Dark Magician and Woodborg Inpachi (2500 ATK or DEF) to banish, against the opponent's Elfnote Lucina.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, GRAVE),
    (DARK_MAGICIAN, GRAVE),
    (DARK_MAGICIAN, GRAVE),
    (WOODBORG_INPACHI, GRAVE),
    (WOODBORG_INPACHI, MZONE),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    c.add_must_be_special_summoned(duel);
    let e0 = duel.create_effect(c);
    e0.set_description(duel, aux::stringid(ID, 0));
    e0.set_type(duel, EFFECT_TYPE_FIELD);
    e0.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e0.set_code(duel, EFFECT_SPSUMMON_PROC);
    e0.set_range(duel, LOCATION_GRAVE);
    e0.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e0.set_condition(duel, selfspcon);
    e0.set_target(duel, selfsptg);
    e0.set_operation(duel, selfspop);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_category(duel, CATEGORY_TOHAND);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_target(duel, rthtg);
    e1.set_operation(duel, rthop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target_range(duel, 0, LOCATION_MZONE);
    e2.set_condition(duel, |duel, _| duel.is_battle_phase());
    e2.set_value(duel, -2500);
    c.register_effect(duel, e2);
}

fn selfspconfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_attack(duel, &[2500]) || c.is_defense(duel, &[2500]))
        && c.is_faceup(duel)
        && c.is_able_to_remove_as_cost(duel)
        && aux::sp_elim_filter(c, duel, true, false)
}

fn selfspcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = c.get_controler(duel);
    let g = duel.get_matching_group(selfspconfilter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, Some(c.into()));
    g.len() >= 2 && duel.get_mzone_count(tp, Some(g)) > 0
}

fn selfsptg(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let rg = duel.get_matching_group(selfspconfilter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, a.c.map(Into::into));
    let g = aux::select_unselect_group_full(duel, rg, 2, 2, &aux::chkf_mmz(tp, 1), tp, HINTMSG_REMOVE, None, true)?;
    if g.is_empty() {
        return Ok(false);
    }
    e.set_label_object_group(duel, Some(g));
    Ok(true)
}

fn selfspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(g) = a.e.get_label_object_group(duel) else { return Ok(()) };
    duel.remove(g, POS_FACEUP, REASON_COST)?;
    Ok(())
}

fn rthctfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_attack(duel, &[2500]) || c.is_defense(duel, &[2500])) && c.is_faceup(duel)
}

fn rthtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, 1 - tp)
            && chkc.is_location(duel, LOCATION_ONFIELD | LOCATION_GRAVE)
            && chkc.is_able_to_hand(duel));
    }
    let ct = duel.get_matching_group_count(rthctfilter, tp, LOCATION_REMOVED, 0, None);
    if chk == 0 {
        return Ok(ct > 0 && duel.is_existing_target(CardId::is_able_to_hand, tp, 0, LOCATION_ONFIELD | LOCATION_GRAVE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
    let g = duel.select_target(
        tp,
        CardId::is_able_to_hand,
        tp,
        0,
        LOCATION_ONFIELD | LOCATION_GRAVE,
        1,
        ct as u8,
        None,
    )?;
    duel.set_operation_info(0, CATEGORY_TOHAND, g, g.map_or(0, |g| g.len() as u8), tp, 0);
    Ok(false)
}

fn rthop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(tg) = duel.get_target_cards(a.e) else { return Ok(()) };
    if !tg.is_empty() {
        duel.send_to_hand(tg, PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}
