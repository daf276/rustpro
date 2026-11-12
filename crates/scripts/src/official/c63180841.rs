// Shooting Star Dragon T.G. EX
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE, OPPONENT};

const ID: u32 = 63180841;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f1(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_type_full(duel, TYPE_SYNCHRO, Some(sc), sumtype, tp)
    }

    fn f2(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_not_tuner(duel, sc, tp) && c.is_type_full(duel, TYPE_SYNCHRO, Some(sc), sumtype, tp)
    }
}

// A copy in the Graveyard and Extra Deck, Synchro Summoned with T.G. Wonder Magician (a Level 5 Tuner) and T.G. Hyper Librarian,
// with a Tuner in the Graveyard to banish and Predaplant Lilizard of the opponent's to target.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, GRAVE),
    (ID, EXTRA),
    (98558751, MZONE),
    (98558751, MZONE),
    (90953320, MZONE),
    (90953320, MZONE),
    (74627016, GRAVE),
    (74627016, GRAVE),
    (18595008, MZONE | OPPONENT),
    (18595008, MZONE | OPPONENT),
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
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_NEGATE + CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, discon);
    e1.set_cost(duel, discost);
    e1.set_target(duel, stock::negate_destroy_target);
    e1.set_operation(duel, disop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_TRIGGER_O + EFFECT_TYPE_FIELD);
    e2.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, dacon);
    e2.set_operation(duel, daop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_code(duel, EVENT_FREE_CHAIN);
    e3.set_range(duel, LOCATION_GRAVE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_condition(duel, spcon);
    e3.set_cost(duel, spcost);
    e3.set_target(duel, sptg);
    e3.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e3);
}

fn disfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_location(duel, LOCATION_MZONE) && c.is_controler(duel, tp)
}

fn discon(duel: &Duel, a: &Args) -> bool {
    let Some(re) = a.re else { return false };
    if !re.is_has_property(duel, EFFECT_FLAG_CARD_TARGET) {
        return false;
    }
    let Some(g) = duel.get_chain_info::<Option<CardSet>>(a.ev as u8, CHAININFO_TARGET_CARDS) else {
        return false;
    };
    re.is_monster_effect(duel)
        && g.is_exists(duel, |c, duel| disfilter(c, duel, a.tp), 1, None)
        && !a.e.get_handler(duel).is_status(duel, STATUS_BATTLE_DESTROYED)
        && duel.is_chain_negatable(a.ev as u8)
}

fn discfilter(c: CardId, duel: &Duel) -> bool {
    c.is_type(duel, TYPE_TUNER) && c.is_able_to_remove_as_cost(duel) && aux::sp_elim_filter(c, duel, true, true)
}

fn discost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    stock::banish_cost(duel, a, chk, discfilter, LOCATION_GRAVE)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    if duel.negate_activation(a.ev as u8)
        && let Some(re) = a.re
        && re.get_handler(duel).is_relate_to_effect(duel, re)
    {
        duel.destroy(a.eg.unwrap_or_default(), REASON_EFFECT)?;
    }
    Ok(())
}

fn dacon(duel: &Duel, a: &Args) -> bool {
    duel.get_attacker().is_some_and(|ac| ac.is_controler(duel, 1 - a.tp))
}

fn daop(duel: &mut Duel, _: &Args) -> Script<()> {
    duel.negate_attack()?;
    Ok(())
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    duel.is_turn_player(1 - a.tp)
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    let synchro = |c: CardId, duel: &Duel| c.is_type(duel, TYPE_SYNCHRO);
    if chk == 0 {
        return Ok(duel.check_release_group_cost(tp, synchro, (2, 2), false, Some(&aux::release_check_mmz), None));
    }
    let g = duel.select_release_group_cost(tp, synchro, (2, 2), false, Some(&aux::release_check_mmz), None)?;
    duel.release(g, REASON_COST)?;
    Ok(false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_can_be_special_summoned(duel, a.e, 0, a.tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, 0);
    Ok(false)
}
