// Black Luster Soldier - Soldier of Chaos
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 49202162;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 3;
    const MAX: Option<usize> = Some(3);

    fn specialchk(duel: &Duel, sg: CardSet, lc: CardId, sumtype: u64, tp: u8) -> bool {
        aux::check_different_property(duel, sg, &|c, duel| c.get_codes_full(duel, Some(lc), sumtype, tp))
    }
}

// Linked from Level 7 and 8 monsters on the field, and weak Sangan to battle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (CARD_BLUEEYES_W_DRAGON, MZONE),
    (CARD_DARK_MAGICIAN, MZONE),
    (30208479, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::CardHint(aux::stringid(ID, 0)),
    Check::ResolvesEffect(0),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_MATERIAL_CHECK);
    e1.set_value(duel, Value::MaterialCheck(matcheck));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_BATTLE_DESTROYING);
    e2.set_condition(duel, aux::bdocon);
    e2.set_target(duel, dtarget);
    e2.set_operation(duel, doperation);
    c.register_effect(duel, e2);
}

fn matcheck(duel: &mut Duel, _: EffectId, c: CardId) {
    if !c
        .get_material(duel)
        .is_exists(duel, |mc, duel| mc.is_monster(duel) && mc.is_level_above(duel, 7), 1, None)
    {
        return;
    }
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE + EFFECT_FLAG_CLIENT_HINT);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(duel, Value::EffectPlayer(aux::tgoval));
    e1.set_reset(duel, RESET_EVENT | (RESETS_STANDARD & !RESET_TOFIELD));
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e2.set_value(duel, Value::EffectPlayer(|duel, e, _, tp| tp != e.get_handler_player(duel)));
    c.register_effect(duel, e2);
}

fn dtarget(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(true);
    }
    let b3 = duel.is_existing_matching_card(CardId::is_able_to_remove, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None);
    let op = duel.select_effect(
        tp,
        &[
            (true, aux::stringid(ID, 1)),
            (true, aux::stringid(ID, 2)),
            (b3, aux::stringid(ID, 3)),
        ],
    )?;
    let op = op.map_or(0, |op| op as i64);
    e.set_label(duel, op);
    e.set_category(duel, 0);
    if op == 3 {
        e.set_category(duel, CATEGORY_REMOVE);
        let g = duel.get_matching_group(CardId::is_able_to_remove, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, None);
        duel.set_operation_info(0, CATEGORY_REMOVE, Some(g), 1, tp, 0);
    }
    Ok(false)
}

fn doperation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let op = e.get_label(duel);
    if op == 1 && c.is_relate_to_effect(duel, e) && c.is_faceup(duel) {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_value(duel, 1500);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        c.register_effect(duel, e1);
    } else if op == 2 {
        let turn = duel.get_turn_count();
        let e1 = duel.create_effect(c);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_EXTRA_ATTACK);
        e1.set_value(duel, 1);
        e1.set_label(duel, i64::from(turn));
        e1.set_condition(duel, |duel, a| i64::from(duel.get_turn_count()) > a.e.get_label(duel));
        e1.set_reset_full(duel, RESETS_STANDARD_PHASE_END | RESET_SELF_TURN, 2);
        c.register_effect(duel, e1);
    } else if op == 3 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
        let g = duel.select_matching_card(tp, CardId::is_able_to_remove, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, None)?;
        if !g.is_empty() {
            duel.remove(g, POS_FACEUP, REASON_EFFECT)?;
        }
    }
    Ok(())
}
