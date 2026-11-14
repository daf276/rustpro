// Cyberdark Wurm
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE};

const ID: u32 = 56100345;

pub const LISTED_SERIES: &[u16] = &[SET_CYBER, SET_CYBERNETIC, SET_CYBER_DRAGON];
pub const LISTED_NAMES: &[u32] = &[CARD_CYBER_DRAGON];

// In the hand and in the Graveyard, with Cyber Dragons on the field and in the hand to send as the cost, and Cyber Emergency in the
// Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (CARD_CYBER_DRAGON, MZONE),
    (CARD_CYBER_DRAGON, HAND),
    (CARD_CYBER_DRAGON, HAND),
    (60600126, GRAVE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_CHANGE_CODE);
    e1.set_range(duel, LOCATION_MZONE | LOCATION_GRAVE);
    e1.set_value(duel, CARD_CYBER_DRAGON as i32);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_HAND | LOCATION_GRAVE);
    e2.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_DUEL);
    e2.set_condition(duel, |duel, a| {
        duel.is_existing_matching_card(
            cybermachinefilter,
            a.tp,
            LOCATION_MZONE | LOCATION_GRAVE,
            0,
            1,
            Some(a.e.get_handler(duel).into()),
        )
    });
    e2.set_cost(duel, spcost);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn cybermachinefilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_CYBER]) && c.is_race(duel, RACE_MACHINE) && c.is_faceup(duel)
}

fn spcostfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_CYBER_DRAGON]) && c.is_monster(duel) && c.is_able_to_grave_as_cost(duel)
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    stock::send_to_grave_cost(duel, a, chk, spcostfilter, LOCATION_HAND | LOCATION_DECK)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && c.is_can_be_special_summoned(duel, a.e, 0, tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_GRAVE as i32);
    Ok(false)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_CYBER, SET_CYBERNETIC]) && c.is_spell_trap(duel) && c.is_able_to_hand(duel)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if c.is_relate_to_effect(duel, a.e) && duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? > 0 {
        let g = duel.get_matching_group(aux::necro_valley_filter(thfilter), tp, LOCATION_GRAVE, 0, None);
        if !g.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 1))? {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
            let sg = g.select(duel, tp, 1, 1, None)?;
            duel.break_effect()?;
            duel.send_to_hand(sg, PLAYER_NONE, REASON_EFFECT)?;
            duel.confirm_cards(1 - tp, sg)?;
        }
    }
    Ok(())
}
