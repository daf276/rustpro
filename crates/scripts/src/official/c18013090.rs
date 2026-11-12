// Nitro Warrior
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 18013090;
const NITRO_SYNCHRON: u32 = 96182448;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f1(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_summon_code(duel, Some(sc), sumtype, tp, &[NITRO_SYNCHRON]) || c.is_has_effect(duel, 20932152).is_some()
    }

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

pub const MATERIAL: &[u32] = &[NITRO_SYNCHRON];
pub const LISTED_NAMES: &[u32] = &[NITRO_SYNCHRON];
pub const MATERIAL_SETCODE: &[u16] = &[SET_SYNCHRON];
// Synchro Summoned from the Extra Deck with Nitro Synchron and Shapesnatch (Level 5). The position change needs an attack that
// destroys a Defense Position monster, which random play rarely makes.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (NITRO_SYNCHRON, MZONE),
    (NITRO_SYNCHRON, MZONE),
    (4035199, MZONE),
    (4035199, MZONE),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 0)))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e0a = duel.create_effect(c);
    e0a.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e0a.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e0a.set_code(duel, EVENT_CHAINING);
    e0a.set_range(duel, LOCATION_MZONE);
    e0a.set_condition(duel, |duel, a| a.ep == a.tp && duel.is_turn_player(a.tp));
    e0a.set_operation(duel, aux::chainreg);
    c.register_effect(duel, e0a);
    let e0b = duel.create_effect(c);
    e0b.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e0b.set_code(duel, EVENT_CHAIN_SOLVED);
    e0b.set_range(duel, LOCATION_MZONE);
    e0b.set_operation(duel, regop);
    c.register_effect(duel, e0b);
    let e1a = duel.create_effect(c);
    e1a.set_type(duel, EFFECT_TYPE_SINGLE);
    e1a.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1a.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1a.set_range(duel, LOCATION_MZONE);
    e1a.set_condition(duel, atkcon);
    e1a.set_value(duel, 1000);
    c.register_effect(duel, e1a);
    let e1b = duel.create_effect(c);
    e1b.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e1b.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e1b.set_code(duel, EVENT_BATTLED);
    e1b.set_condition(duel, |duel, a| a.e.get_handler(duel).has_flag_effect(duel, ID));
    e1b.set_operation(duel, |duel, a| {
        a.e.get_handler(duel).reset_flag_effect(duel, ID);
        Ok(())
    });
    c.register_effect(duel, e1b);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_POSITION);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_BATTLED);
    e2.set_condition(duel, chainattackposcon);
    e2.set_target(duel, chainattackpostg);
    e2.set_operation(duel, chainattackposop);
    c.register_effect(duel, e2);
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if let Some(re) = a.re
        && re.is_has_type(duel, EFFECT_TYPE_ACTIVATE)
        && re.is_spell_effect(duel)
        && c.get_flag_effect(duel, 1) > 0
    {
        c.register_flag_effect(duel, ID, RESETS_STANDARD_PHASE_END, 0, 1);
    }
    Ok(())
}

fn atkcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    duel.get_attacker() == Some(c) && c.has_flag_effect(duel, ID) && duel.is_phase(PHASE_DAMAGE_CAL)
}

fn chainattackposcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.get_battle_target(duel)
        .is_some_and(|bc| bc.is_status(duel, STATUS_BATTLE_DESTROYED))
        && duel.check_mut(|duel| c.can_chain_attack(duel))
}

fn chainattackposfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_defense_pos(duel) && !c.is_status(duel, STATUS_BATTLE_DESTROYED) && c.is_can_change_position(duel)
}

fn chainattackpostg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, 1 - tp) && chkc.is_location(duel, LOCATION_MZONE) && chainattackposfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(chainattackposfilter, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_POSCHANGE);
    let g = duel.select_target(tp, chainattackposfilter, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_POSITION, g, 1, 0, 0);
    Ok(false)
}

fn chainattackposop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
        && tc.is_faceup(duel)
    {
        duel.change_position(tc, POS_FACEUP_ATTACK)?;
        duel.chain_attack_full(Some(tc));
    }
    Ok(())
}
