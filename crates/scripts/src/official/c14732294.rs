// White Knight Lord
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 14732294;

pub const LISTED_SERIES: &[u16] = &[SET_WHITE_KNIGHT_NIGHT];

const WHITE_VEIL: u32 = 49306994;

pub const LISTED_NAMES: &[u32] = &[WHITE_VEIL];

// Copies in the hand and Deck to Special Summon, Normal Monsters in the Graveyard to banish, White Veil in the Deck to add, and the
// opponent's monster to attack and Raigeki to destroy it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, DECK),
    (ID, DECK),
    (WHITE_VEIL, DECK),
    (WHITE_VEIL, DECK),
    (756652, GRAVE),
    (756652, GRAVE),
    (756652, GRAVE),
    (756652, GRAVE),
    (30655537, MZONE | OPPONENT),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_SEARCH + CATEGORY_TOHAND);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, spcost);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_ATKCHANGE);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e2.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, atkcon);
    e2.set_target(duel, atktg);
    e2.set_operation(duel, atkop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_DAMAGE);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e3.set_code(duel, EVENT_DESTROYED);
    e3.set_condition(duel, |duel, a| {
        a.rp == 1 - a.tp && a.e.get_handler(duel).is_previous_controler(duel, a.tp)
    });
    e3.set_target(duel, damtg);
    e3.set_operation(duel, stock::damage);
    c.register_effect(duel, e3);
}

fn spcostfilter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel) && c.is_able_to_remove_as_cost(duel) && aux::sp_elim_filter(c, duel, false, false)
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    let g = duel.get_matching_group(spcostfilter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, None);
    if chk == 0 {
        return Ok(g.len() >= 3 && aux::select_unselect_group_check(duel, g, 3, 3, &aux::chkf_mmz(tp, 1)));
    }
    let sg = aux::select_unselect_group(duel, g, 3, 3, &aux::chkf_mmz(tp, 1), tp, HINTMSG_REMOVE)?;
    duel.remove(sg, POS_FACEUP, REASON_COST)?;
    Ok(false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, (LOCATION_HAND | LOCATION_DECK) as i32);
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_WHITE_KNIGHT_NIGHT]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, WHITE_VEIL) && c.is_able_to_hand(duel)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) && duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? > 0 {
        let b1 = duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_matching_card(|sc, duel| spfilter(sc, duel, e, tp), tp, LOCATION_HAND | LOCATION_DECK, 0, 1, None);
        let b2 = duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None);
        if !((b1 || b2) && duel.select_yes_no(tp, aux::stringid(ID, 3))?) {
            return Ok(());
        }
        let op = duel.select_effect(tp, &[(b1, aux::stringid(ID, 4)), (b2, aux::stringid(ID, 5))])?;
        if op == Some(1) {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
            let filter = |sc, duel: &Duel| spfilter(sc, duel, e, tp);
            let g = duel.select_matching_card(tp, filter, tp, LOCATION_HAND | LOCATION_DECK, 0, 1, 1, None)?;
            if !g.is_empty() {
                duel.break_effect()?;
                duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
            }
        } else if op == Some(2) {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
            let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
            if !g.is_empty() {
                duel.break_effect()?;
                duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
                duel.confirm_cards(1 - tp, g)?;
            }
        }
    }
    Ok(())
}

fn atkcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel)
        .get_battle_target(duel)
        .is_some_and(|bc| bc.is_faceup(duel) && bc.is_controler(duel, 1 - a.tp))
}

fn atktg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    if let Some(bc) = a.e.get_handler(duel).get_battle_target(duel) {
        a.e.set_label_object_group(duel, Some(bc.into()));
        bc.create_effect_relation(duel, a.e);
    }
    Ok(false)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if let Some(bc) = e.get_label_object_group(duel).and_then(|g| g.get_first())
        && bc.is_relate_to_effect(duel, e)
        && bc.is_faceup(duel)
        && bc.is_controler(duel, 1 - tp)
    {
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_code(duel, EFFECT_SET_ATTACK);
        e1.set_value(duel, 0);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        bc.register_effect(duel, e1);
    }
    Ok(())
}

fn damtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(true);
    }
    duel.set_target_player(1 - tp);
    duel.set_target_param(1000);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, 1000);
    Ok(false)
}
