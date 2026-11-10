// Galaxy-Eyes Cipher Dragon
use crate::prelude::*;
use crate::testing::{EXTRA, MZONE};

const ID: u32 = 18963306;

// Twice in the Extra Deck, Xyz Summoned from Galaxy-Eyes Photon Dragons (Level 8) on the field.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (CARD_GALAXYEYES_P_DRAGON, MZONE),
    (CARD_GALAXYEYES_P_DRAGON, MZONE),
    (CARD_GALAXYEYES_P_DRAGON, MZONE),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(8);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_CONTROL);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_count_limit(duel, 1, 0);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_controler_can_be_changed(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, 1 - tp) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONTROL);
    let g = duel.select_target(tp, filter, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_CONTROL, g, 1, 0, 0);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CANNOT_DIRECT_ATTACK);
    e1.set_target_range(duel, LOCATION_MZONE, 0);
    e1.set_target_card(duel, |duel, e, c| e.get_label(duel) != i64::from(c.get_field_id(duel)));
    let field_id = c.get_field_id(duel);
    e1.set_label(duel, field_id.into());
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
    {
        // The core returns the control result as an integer, which is truthy in Lua even when 0.
        let reason_player = duel.get_reason_player();
        duel.get_control_full(tc, tp, PHASE_END as u16, 1, 0xff, reason_player)?;
        duel.negate_related_chain(tc, RESET_TURN_SET);
        for code in [EFFECT_DISABLE, EFFECT_DISABLE_EFFECT, EFFECT_SET_ATTACK_FINAL, EFFECT_CHANGE_CODE] {
            let e2 = duel.create_effect(c);
            e2.set_type(duel, EFFECT_TYPE_SINGLE);
            e2.set_code(duel, code);
            e2.set_condition(duel, concon);
            match code {
                EFFECT_SET_ATTACK_FINAL => e2.set_value(duel, 3000),
                EFFECT_CHANGE_CODE => e2.set_value(duel, ID as i32),
                _ => {}
            }
            e2.set_reset(duel, RESETS_STANDARD_PHASE_END);
            tc.register_effect(duel, e2);
        }
    }
    Ok(())
}

fn concon(duel: &Duel, a: &Args) -> bool {
    let owner = a.e.get_owner(duel);
    a.e.get_handler(duel)
        .get_card_effect(duel, EFFECT_SET_CONTROL)
        .iter()
        .any(|eff| eff.get_owner(duel) == owner)
}
