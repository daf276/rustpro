// Blackwing - Vayu the Emblem of Honor
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE};

const ID: u32 = 72714392;

pub const LISTED_SERIES: &[u16] = &[SET_BLACKWING];
// Blackwing - Sirocco the Dawn (Level 5) in the GY and Blackwing - Boreastorm the Wicked Wind (Level 6) in the Extra Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, GRAVE),
    (ID, GRAVE),
    (75498415, GRAVE),
    (75498415, GRAVE),
    (10602628, EXTRA),
    (10602628, EXTRA),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_GRAVE);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_CANNOT_BE_SYNCHRO_MATERIAL);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_value(duel, Value::Card(|_, _, _| 1));
    c.register_effect(duel, e2);
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    let lv = c.get_level(duel);
    lv > 0
        && c.is_set_card(duel, &[SET_BLACKWING])
        && !c.is_type(duel, TYPE_TUNER)
        && c.is_able_to_remove(duel)
        && duel.is_existing_matching_card(|ec, duel| exfilter(ec, duel, lv + 1, e, tp), tp, LOCATION_EXTRA, 0, 1, None)
}

fn exfilter(c: CardId, duel: &Duel, lv: u32, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_BLACKWING])
        && c.get_level(duel) == lv
        && duel.get_location_count_from_ex(tp, tp, None, c) > 0
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let target = |tc, duel: &Duel| filter(tc, duel, e, tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && target(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel
            .is_player_affected_by_effect(e.get_handler_player(duel), CARD_SPIRIT_ELIMINATION)
            .is_none()
            && c.is_able_to_remove(duel)
            && duel.is_existing_target(target, tp, LOCATION_GRAVE, 0, 1, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_target(tp, target, tp, LOCATION_GRAVE, 0, 1, 1, Some(c.into()))?;
    duel.set_operation_info(0, CATEGORY_REMOVE, g, 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !tc.is_relate_to_effect(duel, e) || !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    if duel.remove(CardSet::from_iter([c, tc]), POS_FACEUP, REASON_EFFECT)? == 2 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let lv = tc.get_level(duel) + 1;
        let sg = duel.select_matching_card(tp, |ec, duel| exfilter(ec, duel, lv, e, tp), tp, LOCATION_EXTRA, 0, 1, 1, None)?;
        if let Some(sc) = sg.get_first()
            && duel.special_summon(sg, 0, tp, tp, false, false, POS_FACEUP)? > 0
        {
            for code in [EFFECT_DISABLE, EFFECT_DISABLE_EFFECT] {
                let e1 = duel.create_effect(c);
                e1.set_type(duel, EFFECT_TYPE_SINGLE);
                e1.set_code(duel, code);
                e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
                sc.register_effect(duel, e1);
            }
        }
    }
    Ok(())
}
