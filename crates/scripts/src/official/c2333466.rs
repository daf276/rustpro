// Gandora-G the Dragon of Destruction
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 2333466;

pub const LISTED_NAMES: &[u32] = &[CARD_SHINING_SARCOPHAGUS];

// Shining Sarcophagus on the field and Dark Magician Girl the Magician's Apprentice (lists it) in the Deck to summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (CARD_SHINING_SARCOPHAGUS, SZONE),
    (2501624, DECK),
    (2501624, DECK),
    (2501624, DECK),
    (4035199, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(
        duel,
        Value::Card(|duel, _, _| duel.get_field_group_count(0, LOCATION_REMOVED, LOCATION_REMOVED) as i32 * 300),
    );
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_HAND);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, spcon);
    e2.set_target(duel, stock::special_summon_self_target_player_0);
    e2.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(
        duel,
        CATEGORY_DESTROY + CATEGORY_REMOVE + CATEGORY_SPECIAL_SUMMON + CATEGORY_LVCHANGE,
    );
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_cost(duel, descost);
    e3.set_target(duel, destg);
    e3.set_operation(duel, desop);
    c.register_effect(duel, e3);
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(
        |c, duel| c.is_faceup(duel) && c.is_code(duel, CARD_SHINING_SARCOPHAGUS),
        a.tp,
        LOCATION_ONFIELD,
        0,
        1,
        None,
    )
}

fn descost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let lp = duel.get_lp(a.tp);
    duel.pay_lp_cost(a.tp, (lp / 2).max(0) as u32)?;
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_level_below(duel, 7) && c.lists_code(duel, CARD_SHINING_SARCOPHAGUS) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let rg = duel.get_matching_group(
        |c, duel| c.is_able_to_remove(duel),
        tp,
        LOCATION_ONFIELD,
        LOCATION_ONFIELD,
        Some(e.get_handler(duel).into()),
    );
    if chk == 0 {
        return Ok(!rg.is_empty()
            && duel.get_mzone_count(tp, Some(rg)) > 0
            && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(rg), rg.len() as u8, 0, 0);
    duel.set_operation_info(0, CATEGORY_REMOVE, Some(rg), rg.len() as u8, 0, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let exc = c.is_relate_to_effect(duel, e).then(|| c.into());
    let rg = duel.get_matching_group(aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, exc);
    let ct = duel.destroy_full(rg, REASON_EFFECT, LOCATION_REMOVED, duel.get_reason_player())?;
    if ct == 0 || duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let Some(sc) = duel
        .select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, 1, None)?
        .get_first()
    else {
        return Ok(());
    };
    duel.break_effect()?;
    if duel.special_summon_step(sc, 0, tp, tp, false, false, POS_FACEUP)? {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_code(duel, EFFECT_UPDATE_LEVEL);
        e1.set_value(duel, ct);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        sc.register_effect(duel, e1);
    }
    duel.special_summon_complete()?;
    Ok(())
}
