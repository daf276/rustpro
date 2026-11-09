// Witchcrafter Madame Verre
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 21522601;

// On the field next to Spellcaster Normal Monsters to battle, with Polymerization (never activated without Fusion materials) in the hand to reveal or
// discard and the opponent's monsters to battle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (46986414, MZONE),
    (46986414, MZONE),
    (50930991, MZONE),
    (CARD_POLYMERIZATION, HAND),
    (CARD_POLYMERIZATION, HAND),
    (CARD_POLYMERIZATION, HAND),
    (7892180, MZONE | OPPONENT),
    (59053232, MZONE | OPPONENT),
    (59053232, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_ATKCHANGE + CATEGORY_DEFCHANGE);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_PRE_DAMAGE_CALCULATE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, atkcon);
    e1.set_target(duel, atktg);
    e1.set_operation(duel, atkop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DISABLE);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, Witchcrafter::discard_cost);
    e2.set_target(duel, distg);
    e2.set_operation(duel, disop);
    c.register_effect(duel, e2);
}

fn atkcon(duel: &Duel, a: &Args) -> bool {
    let (Some(tc), Some(bc)) = duel.get_battle_monster(a.tp) else {
        return false;
    };
    tc.is_race(duel, RACE_SPELLCASTER) && bc.is_controler(duel, 1 - a.tp)
}

fn rvfilt(c: CardId, duel: &Duel) -> bool {
    c.is_spell(duel) && !c.is_public(duel)
}

fn atktg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    Ok(chk == 0 && duel.is_existing_matching_card(rvfilt, a.tp, LOCATION_HAND, 0, 1, None))
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let (Some(tc), _) = duel.get_battle_monster(tp) else {
        return Ok(());
    };
    if !(tc.is_relate_to_battle(duel) && tc.is_faceup(duel) && tc.is_controler(duel, tp)) {
        return Ok(());
    }
    let sg = duel.get_matching_group(rvfilt, tp, LOCATION_HAND, 0, None);
    let ct = sg.get_class_count(duel, |c, duel| i64::from(c.get_code(duel)));
    let g = aux::select_unselect_group(duel, sg, 1, ct, &aux::dncheck, tp, HINTMSG_CONFIRM)?;
    if g.is_empty() {
        return Ok(());
    }
    duel.confirm_cards(1 - tp, g)?;
    duel.shuffle_hand(tp)?;
    let e1 = duel.create_effect(e.get_handler(duel));
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_value(duel, g.len() as i32 * 1000);
    e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
    tc.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_UPDATE_DEFENSE);
    tc.register_effect(duel, e2);
    Ok(())
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    Ok(chk == 0 && duel.is_existing_matching_card(CardId::is_negatable_monster, a.tp, 0, LOCATION_MZONE, 1, None))
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let g = duel.get_matching_group(CardId::is_negatable_monster, a.tp, 0, LOCATION_MZONE, None);
    for tc in g {
        tc.negate_effects(duel, c, RESET_PHASE | PHASE_END, false);
    }
    Ok(())
}
