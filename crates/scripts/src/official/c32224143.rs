// Ghostrick Socuteboss
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};
use engine::constants::MSG_FIELD_DISABLED;

const ID: u32 = 32224143;

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(2);
    const CT: usize = 2;
}

// On the field and in the Extra Deck, with Xyz Import to attach the opponent's monsters, which are also targets to destroy;
// Ghostrick Fairy as another Ghostrick and as Xyz material.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (14602126, HAND),
    (14602126, HAND),
    (16195942, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (36239585, MZONE),
    (36239585, MZONE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::Message(MSG_FIELD_DISABLED)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_count_limit(duel, 1, 0);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_code(duel, EFFECT_CANNOT_BE_BATTLE_TARGET);
    e2.set_condition(duel, atkcon);
    e2.set_value(duel, Value::Card(aux::imval2));
    c.register_effect(duel, e2);
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_GHOSTRICK])
}

fn filter(c: CardId, duel: &Duel, atk: i32) -> bool {
    c.is_faceup(duel) && c.is_attack_below(duel, atk)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let cg = duel.get_matching_group(cfilter, tp, LOCATION_MZONE, LOCATION_MZONE, None);
    let atk = cg.get_sum(duel, |c, duel| c.get_attack(duel).into()) as i32;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && filter(chkc, duel, atk));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(|c, duel| filter(c, duel, atk), tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, |c, duel| filter(c, duel, atk), tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, 0, 0);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let tc = duel.get_first_target();
    unported!(duel, tc.is_none());
    let Some(tc) = tc else { return Ok(()) };
    let mut seq = tc.get_sequence(duel);
    if tc.is_controler(duel, 1 - tp) {
        seq += 16;
    }
    if tc.is_relate_to_effect(duel, e) && tc.is_faceup(duel) && duel.destroy(tc, REASON_EFFECT)? != 0 {
        let g = duel.get_matching_group(cfilter, e.get_handler_player(duel), LOCATION_MZONE, 0, None);
        for tc in g.iter() {
            tc.register_flag_effect(duel, ID, RESET_EVENT | RESETS_STANDARD | RESET_OVERLAY, 0, 1);
        }
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_FIELD);
        e1.set_code(duel, EFFECT_DISABLE_FIELD);
        e1.set_label(duel, seq.into());
        e1.set_condition_mut(duel, discon);
        e1.set_returning_operation(duel, disop);
        e1.set_reset(duel, 0);
        duel.register_effect(e1, tp);
    }
    Ok(())
}

fn cfilter2(c: CardId, duel: &Duel) -> bool {
    cfilter(c, duel) && c.get_flag_effect(duel, ID) != 0
}

fn discon(duel: &mut Duel, a: &Args) -> bool {
    let g = duel.get_matching_group(cfilter, a.e.get_handler_player(duel), LOCATION_MZONE, 0, None);
    if g.is_exists(duel, cfilter2, 1, None) {
        // The loop body's own g:GetNext() also advances aux.Next, so only every other card gets a flag.
        for tc in g.iter().step_by(2) {
            tc.register_flag_effect(duel, ID, RESET_EVENT | RESETS_STANDARD | RESET_OVERLAY, 0, 1);
        }
        return true;
    }
    a.e.reset(duel);
    false
}

fn disop(duel: &mut Duel, a: &Args) -> Script<u32> {
    Ok(1 << a.e.get_label(duel))
}

fn atkcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    duel.is_existing_matching_card(cfilter, a.e.get_handler_player(duel), LOCATION_MZONE, 0, 1, Some(c.into()))
}
