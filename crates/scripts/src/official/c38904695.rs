// Infernity Doom Archfiend
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 38904695;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

// In the Extra Deck, Synchro Summoned with The Magical King of Dimension Zeta (a Level 4 Tuner) and Ojama Green (Level 2), against Sangan.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (42026772, MZONE),
    (12482652, MZONE),
    (12482652, MZONE),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DISABLE + CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_target(duel, negtg);
    e1.set_operation(duel, negop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_CHANGE_BATTLE_DAMAGE);
    e2.set_condition(duel, damcon);
    e2.set_value(duel, Value::BattleDamage(aux::change_battle_damage::<1, DOUBLE_DAMAGE>));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e3.set_code(duel, EVENT_BE_MATERIAL);
    e3.set_condition(duel, efcon);
    e3.set_operation(duel, efop);
    c.register_effect(duel, e3);
}

fn negtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_negatable(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_negatable, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_NEGATE);
    let g = duel.select_target(tp, CardId::is_negatable, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DISABLE, g, 1, 0, 0);
    Ok(false)
}

fn negop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_negatable(duel) && tc.is_relate_to_effect(duel, a.e) {
        for code in [EFFECT_DISABLE, EFFECT_DISABLE_EFFECT] {
            let e = duel.create_effect(c);
            e.set_type(duel, EFFECT_TYPE_SINGLE);
            e.set_code(duel, code);
            e.set_reset(duel, RESETS_STANDARD_PHASE_END);
            tc.register_effect(duel, e);
        }
        if duel.get_field_group_count(tp, LOCATION_HAND, 0) == 0 && duel.select_yes_no(tp, aux::stringid(ID, 1))? {
            duel.adjust_instantly(Some(tc));
            duel.destroy(tc, REASON_EFFECT)?;
        }
    }
    Ok(())
}

fn damcon(duel: &Duel, a: &Args) -> bool {
    let (c, tp) = (a.e.get_handler(duel), a.e.get_handler_player(duel));
    c.get_battle_target(duel).is_some_and(|bc| bc.is_controler(duel, 1 - tp)) && duel.get_field_group_count(tp, LOCATION_HAND, 0) == 0
}

fn efcon(duel: &Duel, a: &Args) -> bool {
    a.r == REASON_SYNCHRO
        && a.e
            .get_handler(duel)
            .get_reason_card(duel)
            .is_some_and(|rc| rc.is_attribute(duel, ATTRIBUTE_DARK))
}

fn efop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let Some(rc) = c.get_reason_card(duel) else { return Ok(()) };
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE + EFFECT_FLAG_CLIENT_HINT);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_EXTRA_ATTACK_MONSTER);
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
    e1.set_value(duel, 1);
    rc.register_effect(duel, e1);
    Ok(())
}
