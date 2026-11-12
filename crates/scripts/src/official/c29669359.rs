// Number 61: Volcasaurus
use crate::prelude::*;
use crate::testing::{EXTRA, MZONE, OPPONENT};

const ID: u32 = 29669359;
pub const XYZ_NUMBER: u32 = 61;

// In the Extra Deck, Xyz Summoned with two Level 5 monsters, with the opponent's monsters to destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (2971090, MZONE),
    (4035199, MZONE),
    (7562372, MZONE),
    (26202165, MZONE | OPPONENT),
    (14558127, MZONE | OPPONENT),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(5);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY + CATEGORY_DAMAGE);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_cost(duel, |duel, a, chk| {
        Cost::and(duel, a, chk, &[Cost::detach_from_self::<1, 1>, descost])
    });
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    c.register_effect(duel, e1);
}

fn descost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(!c.is_direct_attacked(duel));
    }
    let e1 = duel.create_effect(c);
    e1.set_description(duel, 3207);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_OATH + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_DIRECT_ATTACK);
    e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
    c.register_effect(duel, e1);
    Ok(false)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, 1 - tp) && chkc.is_location(duel, LOCATION_MZONE) && chkc.is_faceup(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_faceup, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let Some(tc) = duel
        .select_target(tp, CardId::is_faceup, tp, 0, LOCATION_MZONE, 1, 1, None)?
        .and_then(|g| g.get_first())
    else {
        return Ok(false);
    };
    let atk = tc.get_text_attack(duel).max(0);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(tc.into()), 1, tp, 0);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, atk);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, e) && tc.is_monster(duel) && tc.is_controler(duel, 1 - tp) {
        let atk = tc.get_text_attack(duel).max(0);
        if duel.destroy(tc, REASON_EFFECT)? > 0 {
            duel.damage(1 - tp, atk as u32, REASON_EFFECT)?;
        }
    }
    Ok(())
}
