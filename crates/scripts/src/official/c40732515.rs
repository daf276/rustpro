// Endymion, the Master Magician
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 40732515;
const MAGICAL_CITADEL_OF_ENDYMION: u32 = 39910367;

pub const COUNTER_LIST: &[u16] = &[COUNTER_SPELL];
pub const LISTED_NAMES: &[u32] = &[MAGICAL_CITADEL_OF_ENDYMION];

// On the field with Spells (Raigeki) in the hand to discard and on the opponent's side to destroy. Magical Citadel of Endymion is not
// ported, so the Special Summon procedure and the Spell it adds are only reported.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (12580477, HAND),
    (12580477, HAND),
    (12580477, GRAVE),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 0))),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e1.set_range(duel, LOCATION_HAND | LOCATION_GRAVE);
    e1.set_condition(duel, spcon);
    e1.set_operation(duel, spop);
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TOHAND);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_condition(duel, |duel, a| {
        a.e.get_handler(duel).get_summon_type(duel) == SUMMON_TYPE_SPECIAL + 1
    });
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_DESTROY);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_cost(duel, descost);
    e3.set_target(duel, destg);
    e3.set_operation(duel, stock::destroy_first_target);
    c.register_effect(duel, e3);
}

fn spfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_code(duel, MAGICAL_CITADEL_OF_ENDYMION) && c.is_can_remove_counter(duel, tp, COUNTER_SPELL, 1, REASON_COST)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = c.get_controler(duel);
    let g = duel.get_matching_group(|c, duel| spfilter(c, duel, tp), tp, LOCATION_ONFIELD, 0, None);
    let ct: u32 = g.iter().map(|tc| u32::from(tc.get_counter(duel, COUNTER_SPELL))).sum();
    ct >= 6 && duel.get_location_count(tp, LOCATION_MZONE) > 0
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_matching_group(|c, duel| spfilter(c, duel, tp), tp, LOCATION_ONFIELD, 0, None);
    if g.len() == 1 {
        if let Some(tc) = g.get_first() {
            tc.remove_counter(duel, tp, COUNTER_SPELL, 6, REASON_COST)?;
        }
    } else {
        for _ in 0..6 {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_SELECT);
            if let Some(tc) = duel
                .select_matching_card(tp, |c, duel| spfilter(c, duel, tp), tp, LOCATION_ONFIELD, 0, 1, 1, None)?
                .get_first()
            {
                tc.remove_counter(duel, tp, COUNTER_SPELL, 1, REASON_COST)?;
            }
        }
    }
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_spell(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && thfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(true);
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_target(tp, thfilter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOHAND, g, g.map_or(0, |g| g.len() as u8), 0, 0);
    Ok(false)
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_spell(duel) && c.is_discardable(duel)
}

fn descost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        let c = a.e.get_handler(duel);
        return Ok(duel.is_existing_matching_card(cfilter, tp, LOCATION_HAND, 0, 1, Some(c.into())));
    }
    duel.discard_hand(tp, cfilter, 1, 1, REASON_COST | REASON_DISCARD)?;
    Ok(false)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, 0, 0);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.send_to_hand(tc, PLAYER_NONE, REASON_EFFECT)?;
        duel.shuffle_hand(a.tp)?;
    }
    Ok(())
}
