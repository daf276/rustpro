// Genex Ally Triarm
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT, SZONE};
use engine::{OperationFn, TargetFn};

const ID: u32 = 17760003;
const GENEX_CONTROLLER: u32 = 68505803;

pub const MATERIAL: &[u32] = &[GENEX_CONTROLLER];
pub const LISTED_NAMES: &[u32] = &[GENEX_CONTROLLER];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f1(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_summon_code(duel, Some(sc), sumtype, tp, &[GENEX_CONTROLLER])
    }

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

// In the Extra Deck, Synchro Summoned from Genex Controller (a Level 3 Tuner) and a DARK, a WATER and a WIND Level 1 non-Tuner
// (Kozaky, Gigobyte, Dancing Elf), for all three effects, with a Set Spell/Trap and a LIGHT monster (Key Mace) of the opponent's
// to destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (GENEX_CONTROLLER, MZONE),
    (GENEX_CONTROLLER, MZONE),
    (99171160, MZONE),
    (53776525, MZONE),
    (59983499, MZONE),
    (22359980, SZONE | OPPONENT),
    (1929294, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_MATERIAL_CHECK);
    e1.set_value(duel, Value::MaterialCheck(valcheck));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_condition(duel, regcon);
    e2.set_operation(duel, regop);
    c.register_effect(duel, e2);
    e2.set_label_object(duel, Some(e1));
}

fn valcheck(duel: &mut Duel, e: EffectId, c: CardId) {
    let g = c.get_material(duel);
    let mut att = 0;
    for tc in g {
        if !tc.is_code(duel, GENEX_CONTROLLER) || !tc.is_type(duel, TYPE_TUNER) {
            att |= tc.get_attribute(duel);
        }
    }
    e.set_label(duel, i64::from(att & (ATTRIBUTE_WIND | ATTRIBUTE_WATER | ATTRIBUTE_DARK)));
}

fn regcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).is_synchro_summoned(duel) && a.e.get_label_object(duel).is_some_and(|e1| e1.get_label(duel) != 0)
}

fn register(duel: &mut Duel, c: CardId, n: u64, category: u64, property: u32, target: TargetFn, operation: OperationFn) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, n));
    e1.set_category(duel, category);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, property);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit_full(duel, 1, 0, EFFECT_COUNT_CODE_SINGLE);
    e1.set_cost(duel, cost);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
    c.register_effect(duel, e1);
    c.register_flag_effect_full(
        duel,
        0,
        RESET_EVENT | RESETS_STANDARD,
        EFFECT_FLAG_CLIENT_HINT,
        1,
        0,
        aux::stringid(ID, n + 3),
    );
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    let att = a.e.get_label_object(duel).map_or(0, |e1| e1.get_label(duel)) as u32;
    let c = a.e.get_handler(duel);
    if att & ATTRIBUTE_WIND != 0 {
        register(duel, c, 0, 0, 0, target1, operation1);
    }
    if att & ATTRIBUTE_WATER != 0 {
        register(
            duel,
            c,
            1,
            CATEGORY_DESTROY,
            EFFECT_FLAG_CARD_TARGET,
            target2,
            stock::destroy_first_target,
        );
    }
    if att & ATTRIBUTE_DARK != 0 {
        register(
            duel,
            c,
            2,
            CATEGORY_DESTROY + CATEGORY_DRAW,
            EFFECT_FLAG_CARD_TARGET,
            target3,
            operation3,
        );
    }
    Ok(())
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (tp, c) = (a.tp, a.e.get_handler(duel));
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_discardable, tp, LOCATION_HAND, 0, 1, Some(c.into())));
    }
    duel.discard_hand(tp, CardId::is_discardable, 1, 1, REASON_COST | REASON_DISCARD)?;
    Ok(false)
}

fn target1(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_field_group_count(tp, 0, LOCATION_HAND) > 0);
    }
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, 1 - tp, LOCATION_HAND as i32);
    Ok(false)
}

fn operation1(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_field_group(tp, 0, LOCATION_HAND);
    if g.is_empty() {
        return Ok(());
    }
    let sg = g.random_select(duel, tp, 1);
    duel.send_to_grave(sg, REASON_EFFECT)?;
    Ok(())
}

fn target2(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_spell_trap(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_spell_trap, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, CardId::is_spell_trap, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, 0, 0);
    Ok(false)
}

fn filter3(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_attribute(duel, ATTRIBUTE_LIGHT)
}

fn target3(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && filter3(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_player_can_draw(tp, 1) && duel.is_existing_target(filter3, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, filter3, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 1);
    Ok(false)
}

fn operation3(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
        && tc.is_faceup(duel)
    {
        duel.destroy(tc, REASON_EFFECT)?;
        duel.draw(a.tp, 1, REASON_EFFECT)?;
    }
    Ok(())
}
