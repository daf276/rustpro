// Gouki Jet Ogre
use super::c22510667::Procedure;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 59644128;

// Link Summoned from Gouki Guts and Gouki Machine Suprex, with Gouki Guts to destroy and face-down monsters to turn up.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (7540107, MZONE),
    (29099860, MZONE),
    (7540107, MZONE),
    (7540107, MZONE),
    (7540107, MZONE | OPPONENT),
    (29099860, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure<2, 2>>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY + CATEGORY_POSITION);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_target(duel, postg);
    e1.set_operation(duel, posop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).is_previous_location(duel, LOCATION_ONFIELD));
    e2.set_target(duel, atktg);
    e2.set_operation(duel, atkop);
    c.register_effect(duel, e2);
}

fn defense_or_facedown(c: CardId, duel: &Duel) -> bool {
    c.is_defense_pos(duel) || c.is_facedown(duel)
}

fn desfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_faceup(duel)
        && c.is_set_card(duel, &[SET_GOUKI])
        && duel.is_existing_matching_card(defense_or_facedown, tp, LOCATION_MZONE, LOCATION_MZONE, 1, Some(c.into()))
}

fn postg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_controler(duel, tp) && desfilter(chkc, duel, tp));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(|c, duel| desfilter(c, duel, tp), tp, LOCATION_ONFIELD, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, |c, duel| desfilter(c, duel, tp), tp, LOCATION_ONFIELD, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, 0, 0);
    Ok(false)
}

fn posop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(tc) = duel.get_first_target() else {
        return Ok(());
    };
    if tc.is_relate_to_effect(duel, a.e) && duel.destroy(tc, REASON_EFFECT)? > 0 {
        let g = duel.get_matching_group(defense_or_facedown, a.tp, LOCATION_MZONE, LOCATION_MZONE, None);
        if g.is_empty() {
            return Ok(());
        }
        duel.change_position(g, POS_FACEUP_ATTACK)?;
    }
    Ok(())
}

fn atktg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    Ok(chk != 0 || duel.is_existing_matching_card(aux::faceup_filter(gouki), a.tp, LOCATION_MZONE, 0, 1, None))
}

fn gouki(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_GOUKI])
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_matching_group(aux::faceup_filter(gouki), a.tp, LOCATION_MZONE, 0, None);
    let c = a.e.get_handler(duel);
    for tc in g {
        tc.update_attack(duel, 500, Some(RESETS_STANDARD_PHASE_END), Some(c), None);
    }
    Ok(())
}
