// Paladins of Bonds and Unity
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 99217226;

// In the Extra Deck, Synchro Summoned from Angel Trumpeter (Level 4 Tuner) and Pesanta (Level 8, 2500 ATK and DEF).
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (87979586, MZONE),
    (87979586, MZONE),
    (66429798, MZONE),
    (66429798, MZONE),
];
// The negation needs 25 cards in the opponent's Graveyard.
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 0)))];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, |c, duel| {
            c.is_base_attack(duel, 2500) && c.is_base_defense(duel, 2500)
        })
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_synchro_summoned(duel));
    e1.set_value(duel, Value::EffectPlayer(aux::indoval));
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
    e2.set_value(duel, Value::EffectPlayer(aux::tgoval));
    c.register_effect(duel, e2);
    let e3 = e1.clone_effect(duel);
    e3.set_code(duel, EFFECT_UPDATE_ATTACK);
    e3.set_condition(duel, |duel, a| {
        let e = a.e;
        e.get_handler(duel).is_synchro_summoned(duel) && duel.get_field_group_count(e.get_handler_player(duel), LOCATION_GRAVE, 0) >= 25
    });
    e3.set_value(
        duel,
        Value::Card(|duel, _, _| {
            (duel.get_field_group_count(0, LOCATION_ONFIELD | LOCATION_GRAVE, LOCATION_ONFIELD | LOCATION_GRAVE) * 100) as i32
        }),
    );
    c.register_effect(duel, e3);
    let e4 = e3.clone_effect(duel);
    e4.set_code(duel, EFFECT_UPDATE_DEFENSE);
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_description(duel, aux::stringid(ID, 0));
    e5.set_category(duel, CATEGORY_NEGATE + CATEGORY_DESTROY);
    e5.set_type(duel, EFFECT_TYPE_QUICK_O);
    e5.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e5.set_code(duel, EVENT_CHAINING);
    e5.set_range(duel, LOCATION_MZONE);
    e5.set_count_limit(duel, 1, 0);
    e5.set_condition(duel, negcon);
    e5.set_target(duel, negtg);
    e5.set_operation(duel, negop);
    c.register_effect(duel, e5);
}

fn negcon(duel: &Duel, a: &Args) -> bool {
    let Some(re) = a.re else { return false };
    re.is_spell_effect(duel)
        && re.is_has_type(duel, EFFECT_TYPE_ACTIVATE)
        && duel.get_field_group_count(a.tp, 0, LOCATION_GRAVE) >= 25
        && !a.e.get_handler(duel).is_status(duel, STATUS_BATTLE_DESTROYED)
        && duel.is_chain_negatable(a.ev as u8)
}

fn negtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let Some(re) = a.re else { return Ok(false) };
    let rc = re.get_handler(duel);
    duel.set_operation_info(0, CATEGORY_NEGATE, a.eg, 1, a.tp, 0);
    if rc.is_destructable(duel) && rc.is_relate_to_effect(duel, re) {
        duel.set_operation_info(0, CATEGORY_DESTROY, a.eg, 1, a.tp, 0);
    }
    Ok(false)
}

fn negop(duel: &mut Duel, a: &Args) -> Script<()> {
    if duel.negate_activation(a.ev as u8)
        && let Some(re) = a.re
        && re.get_handler(duel).is_relate_to_effect(duel, re)
    {
        duel.destroy(a.eg.unwrap_or_default(), REASON_EFFECT)?;
    }
    Ok(())
}
