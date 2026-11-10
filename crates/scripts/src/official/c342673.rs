// Dark Magician the Magician of Black Magic
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 342673;

pub const LISTED_NAMES: &[u32] = &[CARD_DARK_MAGICIAN, CARD_SHINING_SARCOPHAGUS];
// In the hand and on the field with Shining Sarcophagus, Spells naming Dark Magician (Dark Magical Circle, Magician's Rod) in the
// Deck to Set, an opponent's Dark Magician (Level 7) to allow the Special Summon and their Raigeki to destroy it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (CARD_SHINING_SARCOPHAGUS, SZONE),
    (47222536, DECK),
    (47222536, DECK),
    (7084129, DECK),
    (CARD_DARK_MAGICIAN, MZONE | OPPONENT),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_CHANGE_CODE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(duel, CARD_DARK_MAGICIAN as i32);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_HAND);
    e2.set_count_limit(duel, 1, (ID, 0));
    e2.set_condition(duel, |duel, _| {
        duel.is_existing_matching_card(
            |c, duel| c.is_faceup(duel) && c.is_code(duel, CARD_SHINING_SARCOPHAGUS),
            0,
            LOCATION_ONFIELD,
            LOCATION_ONFIELD,
            1,
            None,
        )
    });
    e2.set_target(duel, hsptg);
    e2.set_operation(duel, hspop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_SET);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_DESTROYED);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_condition(duel, |duel, a| a.e.get_handler(duel).is_reason(duel, REASON_EFFECT));
    e3.set_target(duel, dessptg);
    e3.set_operation(duel, desspop);
    c.register_effect(duel, e3);
}

fn hsptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    Ok(false)
}

fn hspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) {
        duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}

fn dessptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
            && duel.is_existing_matching_card(
                |c, duel| c.is_faceup(duel) && c.is_level_above(duel, 5),
                tp,
                LOCATION_MZONE,
                LOCATION_MZONE,
                1,
                None,
            ));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    Ok(false)
}

fn setfilter(c: CardId, duel: &Duel) -> bool {
    c.lists_code(duel, CARD_DARK_MAGICIAN) && c.is_spell_trap(duel) && c.is_ssetable(duel)
}

fn desspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) || duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? == 0 {
        return Ok(());
    }
    let g = duel.get_matching_group(setfilter, tp, LOCATION_DECK, 0, None);
    if !g.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 2))? {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SET);
        let sg = g.select(duel, tp, 1, 1, None)?;
        duel.break_effect()?;
        duel.sset(tp, sg)?;
    }
    Ok(())
}
