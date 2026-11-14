// Grandtusk Dragon
use crate::prelude::*;
use crate::testing::{Check, HAND};

const ID: u32 = 98898163;

pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY + CATEGORY_ATKCHANGE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e2);
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc != c && chkc.is_on_field(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(|_, _| true, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, |_, _| true, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 2, Some(c.into()))?;
    let count = g.map_or(0, |g| g.len()) as u8;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, count, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_target_cards(a.e).unwrap_or_default();
    if g.is_empty() {
        return Ok(());
    }
    let ct = duel.destroy(g, REASON_EFFECT)?;
    let c = a.e.get_handler(duel);
    if ct > 0 && c.is_faceup(duel) && c.is_relate_to_effect(duel, a.e) {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
        e1.set_range(duel, LOCATION_MZONE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_value(duel, ct * 600);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD_DISABLE);
        c.register_effect(duel, e1);
    }
    Ok(())
}
