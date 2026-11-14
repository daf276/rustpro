// Fire King Avatar Rangbali
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 38910263;
const FIRE_KING_ISLAND: u32 = 57554544;

pub const LISTED_SERIES: &[u16] = &[SET_FIRE_KING];
// Copies in hand and on the field, Fire King Island and Fire King Courtier Ulcanix to destroy a Fire King and to pay for the
// negation, and the opponent's Raigeki and Dark Hole to negate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, MZONE),
    (FIRE_KING_ISLAND, HAND),
    (44455560, MZONE),
    (44455560, HAND),
    (12580477, HAND | OPPONENT),
    (53129443, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_DESTROYED);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| {
        a.eg.unwrap_or_default()
            .is_exists(duel, |c, duel| spconfilter(c, duel, a.tp), 1, None)
    });
    e1.set_target(duel, stock::special_summon_self_target_player_0);
    e1.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_NEGATE + CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| {
        a.re.is_some_and(|re| re.is_spell_trap_effect(duel))
            && duel.is_chain_negatable(a.ev as u8)
            && !a.e.get_handler(duel).is_status(duel, STATUS_BATTLE_DESTROYED)
    });
    e2.set_target(duel, |duel, a, chk, _| negtg(duel, a, chk, desfilter));
    e2.set_operation(duel, negop);
    c.register_effect(duel, e2);
}

fn spconfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_previous_location(duel, LOCATION_MZONE)
        && c.is_previous_position(duel, POS_FACEUP)
        && c.is_previous_controler(duel, tp)
        && c.is_previous_set_card(duel, &[SET_FIRE_KING])
        && c.is_reason(duel, REASON_EFFECT)
}

fn desfilter(c: CardId, duel: &Duel) -> bool {
    c.is_attribute(duel, ATTRIBUTE_FIRE) && (c.is_location(duel, LOCATION_HAND) || c.is_faceup(duel))
}

pub fn negtg(duel: &mut Duel, a: &Args, chk: i32, desfilter: fn(CardId, &Duel) -> bool) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        let c = a.e.get_handler(duel);
        return Ok(duel.is_existing_matching_card(desfilter, tp, LOCATION_MZONE | LOCATION_HAND, 0, 1, Some(c.into())));
    }
    duel.set_operation_info(0, CATEGORY_NEGATE, a.eg, 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_DESTROY, None, 1, tp, (LOCATION_MZONE | LOCATION_HAND) as i32);
    Ok(false)
}

fn negop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if !duel.negate_activation(a.ev as u8) {
        return Ok(());
    }
    let c = e.get_handler(duel);
    let exc = c.is_relate_to_effect(duel, e).then_some(c.into());
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_matching_card(tp, desfilter, tp, LOCATION_MZONE | LOCATION_HAND, 0, 1, 1, exc)?;
    if !g.is_empty() {
        duel.destroy(g, REASON_EFFECT)?;
    }
    Ok(())
}
