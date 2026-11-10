// Turbo Warrior
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 46195773;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f1(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_summon_code(duel, Some(sc), sumtype, tp, &[67270095]) || c.is_has_effect(duel, 20932152).is_some()
    }

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

pub const MATERIAL: &[u32] = &[67270095];
pub const LISTED_NAMES: &[u32] = &[67270095];
pub const MATERIAL_SETCODE: &[u16] = &[SET_SYNCHRON];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];
// Two on the field attacking Dragunity Knight - Luin, and one in the Extra Deck, Synchro Summoned with Turbo Synchron and Shapesnatch (Level 5) or Quickdraw Synchron and Key Mace.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (ID, MZONE),
    (67270095, MZONE),
    (4035199, MZONE),
    (20932152, MZONE),
    (1929294, MZONE),
    (12496261, MZONE | OPPONENT),
    (12496261, MZONE | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_ATKCHANGE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e1.set_target(duel, atktg);
    e1.set_operation(duel, atkop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_value(
        duel,
        Value::EffectPlayer(|duel, _, re, _| (1..=6).contains(&re.get_card_level(duel))),
    );
    c.register_effect(duel, e2);
}

fn atktg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let d = duel.get_attack_target();
    if chk == 0 {
        return Ok(d.is_some_and(|d| d.is_faceup(duel) && d.get_level(duel) >= 6 && d.is_type(duel, TYPE_SYNCHRO)));
    }
    if let Some(d) = d {
        d.create_effect_relation(duel, a.e);
        duel.set_operation_info(0, CATEGORY_POSITION, Some(d.into()), 1, 0, 0);
    }
    Ok(false)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let d = duel.get_attack_target();
    unported!(duel, d.is_none());
    if let Some(d) = d.filter(|d| d.is_relate_to_effect(duel, a.e)) {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_SET_ATTACK_FINAL);
        e1.set_value(duel, (f64::from(d.get_attack(duel)) / 2.0).round() as i32);
        e1.set_reset(duel, RESET_PHASE | PHASE_DAMAGE);
        d.register_effect(duel, e1);
    }
    Ok(())
}
