// Steamed Sabersaurus
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 3743515;

// Xyz Summoned from the Extra Deck with Level 4 Dinosaurs (Uraby), which also attack the opponent's monsters (Key Mace), and copies on
// the field for the Battle Phase effect.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (1784619, MZONE),
    (1784619, MZONE),
    (1784619, MZONE),
    (1784619, HAND),
    (1929294, MZONE | OPPONENT),
    (1929294, MZONE | OPPONENT),
    (ID, MZONE),
    (ID, MZONE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(4);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY + CATEGORY_POSITION);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DESTROY + CATEGORY_ATKCHANGE);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_BATTLE_START);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, atkcon);
    e2.set_target(duel, atktg);
    e2.set_operation(duel, atkop);
    c.register_effect(duel, e2);
}

fn desfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_DINOSAUR) && (c.is_faceup(duel) || c.is_location(duel, LOCATION_HAND))
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(desfilter, tp, LOCATION_HAND | LOCATION_MZONE, 0, 1, None));
    }
    let g = duel.get_matching_group(desfilter, tp, LOCATION_HAND | LOCATION_MZONE, 0, None);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), 1, tp, 0);
    duel.set_possible_operation_info(0, CATEGORY_POSITION, None, 1, tp, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_matching_card(tp, desfilter, tp, LOCATION_HAND | LOCATION_MZONE, 0, 1, 1, None)?;
    let Some(dc) = g.get_first() else { return Ok(()) };
    if dc.is_on_field(duel) {
        duel.hint_selection(dc);
    }
    if duel.destroy(dc, REASON_EFFECT)? > 0
        && duel.is_existing_matching_card(CardId::is_can_change_position, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None)
        && duel.select_yes_no(tp, aux::stringid(ID, 2))?
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_POSCHANGE);
        let g = duel.select_matching_card(tp, CardId::is_can_change_position, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
        if g.is_empty() {
            return Ok(());
        }
        duel.hint_selection(g);
        duel.break_effect()?;
        duel.change_position_full(
            g,
            [POS_FACEUP_DEFENSE, POS_FACEDOWN_DEFENSE, POS_FACEUP_ATTACK, POS_FACEUP_ATTACK],
            false,
        )?;
    }
    Ok(())
}

// The attacker, or the attack target when the opponent attacks; the Lua errors on a nil one.
fn battling_monster(duel: &Duel, tp: u8) -> Option<CardId> {
    let (attacker, target) = (duel.get_attacker(), duel.get_attack_target());
    let monster = if attacker.is_some_and(|attacker| attacker.is_controler(duel, 1 - tp)) {
        target
    } else {
        attacker
    };
    unported!(duel, monster.is_none());
    monster
}

fn atkcon(duel: &Duel, a: &Args) -> bool {
    battling_monster(duel, a.tp).is_some_and(|atk| atk.is_faceup(duel) && atk.is_race(duel, RACE_DINOSAUR) && atk != a.e.get_handler(duel))
}

fn atktg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let atk = battling_monster(duel, tp);
    if let Some(atk) = atk {
        duel.set_target_card(atk);
    }
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(c.into()), 1, tp, 0);
    duel.set_operation_info(0, CATEGORY_ATKCHANGE, atk.map(CardSet::from), 1, tp, 2000);
    Ok(false)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let bc = duel.get_first_target();
    if c.is_relate_to_effect(duel, a.e)
        && duel.destroy(c, REASON_EFFECT)? > 0
        && let Some(bc) = bc
        && bc.is_relate_to_effect(duel, a.e)
        && bc.is_faceup(duel)
        && bc.is_race(duel, RACE_DINOSAUR)
    {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_value(duel, 2000);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD | RESET_PHASE | PHASE_BATTLE);
        bc.register_effect(duel, e1);
    }
    Ok(())
}
