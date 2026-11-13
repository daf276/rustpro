// Dinowrestler King T Wrextle
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 77967790;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;

    fn f(c: CardId, duel: &Duel, _: CardId, _: u64, _: u8) -> bool {
        c.is_set_card(duel, &[SET_DINOWRESTLER])
    }
}

// Link Summoned from Coelasilat and the like, with Elfnote Lucina in Attack Position on the opponent's side to target.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (75366958, MZONE),
    (75366958, MZONE),
    (75366958, MZONE),
    (82385847, MZONE),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Link::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, 0, 1);
    e1.set_value(
        duel,
        Value::EffectPlayer(|duel, _, re, _| re.is_has_type(duel, EFFECT_TYPE_ACTIVATE)),
    );
    e1.set_condition(duel, actcon);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_CANNOT_SELECT_BATTLE_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target_range(duel, 0, LOCATION_MZONE);
    e2.set_value(duel, Value::Card(|duel, e, c| i32::from(c != e.get_handler(duel))));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_PHASE | PHASE_BATTLE_START);
    e3.set_count_limit(duel, 1, 0);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_condition(duel, |duel, a| duel.is_turn_player(1 - a.tp));
    e3.set_target(duel, atktg);
    e3.set_operation(duel, atkop);
    c.register_effect(duel, e3);
}

fn actcon(duel: &Duel, a: &Args) -> bool {
    let c = Some(a.e.get_handler(duel));
    duel.get_attacker() == c || duel.get_attack_target() == c
}

fn atktg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, 1 - tp) && chkc.is_location(duel, LOCATION_MZONE) && chkc.is_attack_pos(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(|c, duel| c.is_attack_pos(duel), tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_POSCHANGE);
    duel.select_target(tp, |c, duel| c.is_attack_pos(duel), tp, 0, LOCATION_MZONE, 1, 1, None)?;
    Ok(false)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, a.e) {
        tc.register_flag_effect(duel, ID, RESETS_STANDARD_PHASE_END, 0, 1);
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_FIELD);
        e1.set_code(duel, EFFECT_CANNOT_ATTACK);
        e1.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
        e1.set_target_range(duel, 0, LOCATION_MZONE);
        e1.set_condition(duel, atkcon2);
        e1.set_target_card(duel, |duel, e, c| {
            Some(c) != e.get_label_object_group(duel).and_then(|g| g.get_first())
        });
        e1.set_label_object_group(duel, Some(tc.into()));
        e1.set_reset(duel, RESET_PHASE | PHASE_BATTLE);
        duel.register_effect(e1, tp);
        let e2 = duel.create_effect(c);
        e2.set_description(duel, aux::stringid(ID, 1));
        e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        e2.set_code(duel, EVENT_PHASE | PHASE_BATTLE);
        e2.set_count_limit(duel, 1, 0);
        e2.set_label_object_group(duel, Some(tc.into()));
        e2.set_condition(duel, atkcon2);
        e2.set_operation(duel, desop);
        e2.set_reset(duel, RESET_PHASE | PHASE_BATTLE);
        duel.register_effect(e2, tp);
    }
    Ok(())
}

fn atkcon2(duel: &Duel, a: &Args) -> bool {
    a.e.get_label_object_group(duel)
        .and_then(|g| g.get_first())
        .is_some_and(|tc| tc.get_flag_effect(duel, ID) != 0 && tc.get_attack_announced_count(duel) == 0)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = a.e.get_label_object_group(duel).and_then(|g| g.get_first()) {
        duel.hint(HINT_CARD, 0, ID.into());
        duel.destroy(tc, REASON_EFFECT)?;
    }
    Ok(())
}
