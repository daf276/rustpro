// Aromaseraphy Rosemary
use super::c44035031::Procedure;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 38148100;

// In the Extra Deck, Synchro Summoned with Angel Trumpeter (a Level 4 Plant Tuner) and Key Mace (Level 1), with Aroma monsters
// (Angelica, Rosalina) and Dian Keto the Cure Master and Goblin's Secret Remedy to gain LP, and an opponent's Sangan to negate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (87979586, MZONE),
    (87979586, MZONE),
    (1929294, MZONE),
    (1929294, MZONE),
    (16759958, MZONE),
    (24689197, MZONE),
    (84257639, HAND),
    (11868825, HAND),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, LOCATION_MZONE, 0);
    e1.set_condition(duel, adcon);
    e1.set_target_card(duel, |duel, _, c| c.is_race(duel, RACE_PLANT));
    e1.set_value(duel, 500);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_UPDATE_DEFENSE);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_DISABLE);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_code(duel, EVENT_RECOVER);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_condition(duel, |_, a| a.ep == a.tp);
    e3.set_target(duel, negtg);
    e3.set_operation(duel, negop);
    c.register_effect(duel, e3);
}

fn adcon(duel: &Duel, a: &Args) -> bool {
    let tp = a.e.get_handler_player(duel);
    duel.get_lp(tp) > duel.get_lp(1 - tp)
}

fn negtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, 1 - tp) && chkc.is_on_field(duel) && chkc.is_negatable(duel));
    }
    if chk == 0 {
        return Ok(true);
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    let g = duel.select_target(tp, CardId::is_negatable, tp, 0, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DISABLE, g, 1, 0, 0);
    Ok(false)
}

fn negop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if let Some(tc) = duel.get_first_target()
        && ((tc.is_faceup(duel) && !tc.is_disabled(duel)) || tc.is_type(duel, TYPE_TRAPMONSTER))
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.negate_related_chain(tc, RESET_TURN_SET);
        super::c97268402::negate(duel, c, tc, EFFECT_FLAG_CANNOT_DISABLE);
        if tc.is_type(duel, TYPE_TRAPMONSTER) {
            let e3 = duel.create_effect(c);
            e3.set_type(duel, EFFECT_TYPE_SINGLE);
            e3.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
            e3.set_code(duel, EFFECT_DISABLE_TRAPMONSTER);
            e3.set_reset(duel, RESETS_STANDARD_PHASE_END);
            tc.register_effect(duel, e3);
        }
    }
    Ok(())
}
