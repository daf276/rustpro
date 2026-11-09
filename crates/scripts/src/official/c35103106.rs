// Evolzar Lars
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 35103106;

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(6);
    const CT: usize = 2;
}

// In the Extra Deck, Xyz Summoned with Level 6 Dinosaurs (Frostosaurus, Sword Arm of Dragon) and Summoned Skull, with
// Raigeki in the opponent's hand to chain to and Elfnote Lucina on their field to negate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (6631034, MZONE),
    (13069066, MZONE),
    (70781052, MZONE),
    (12580477, HAND | OPPONENT),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).get_overlay_count(duel) > 0);
    e1.set_value(
        duel,
        Value::EffectPlayer(|duel, e, re, rp| rp == 1 - e.get_handler_player(duel) && re.is_monster_effect(duel)),
    );
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_DISABLE);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, |_, a| a.rp == 1 - a.tp);
    e2.set_cost(duel, discost);
    e2.set_target(duel, distg);
    e2.set_operation(duel, disop);
    c.register_effect(duel, e2);
}

fn discost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let g = a.e.get_handler(duel).get_overlay_group(duel);
    let min = if g.len() == g.filter_count(duel, |c, duel| c.is_race(duel, RACE_REPTILE | RACE_DINOSAUR), None) {
        1
    } else {
        2
    };
    Cost::detach_from_self_with(duel, a, chk, min, 2, |_, _, _| {})
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, 1 - tp) && chkc.is_on_field(duel) && chkc.is_negatable(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_negatable, tp, 0, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_NEGATE);
    let g = duel.select_target(tp, CardId::is_negatable, tp, 0, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DISABLE, g, 1, tp, 0);
    Ok(false)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, a.e) && tc.is_faceup(duel) {
        tc.negate_effects(duel, a.e.get_handler(duel), RESET_PHASE | PHASE_END, true);
    }
    Ok(())
}
