// Psychic End Punisher
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 60465049;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

// In the Extra Deck, Synchro Summoned with Ash Blossom & Joyous Spring and Stealth Angecradle; on the field with monsters
// on both sides to banish.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (14558127, MZONE),
    (64413994, MZONE),
    (79629370, MZONE | OPPONENT),
    (15025844, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_IMMUNE_EFFECT);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, immcon);
    e1.set_value(
        duel,
        Value::Effect(|duel, e, te| te.get_owner_player(duel) != e.get_handler_player(duel) && te.is_activated(duel)),
    );
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_REMOVE);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_cost(duel, Cost::pay_lp::<1000>);
    e2.set_target(duel, rmtg);
    e2.set_operation(duel, rmop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_ATKCHANGE);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_PHASE | PHASE_BATTLE_START);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_condition(duel, |duel, a| duel.get_lp(a.tp) != duel.get_lp(1 - a.tp));
    e3.set_operation(duel, atkop);
    c.register_effect(duel, e3);
}

fn immcon(duel: &Duel, a: &Args) -> bool {
    let tp = a.e.get_handler_player(duel);
    duel.get_lp(tp) <= duel.get_lp(1 - tp) && a.e.get_handler(duel).is_synchro_summoned(duel)
}

fn rescon(duel: &Duel, sg: CardSet, _: CardSet, _: Option<CardId>, tp: u8) -> (bool, bool) {
    (sg.filter_count(duel, |c, duel| c.is_controler(duel, tp), None) == 1, false)
}

fn rmtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chkc.is_some() {
        return Ok(false);
    }
    let g = duel.get_matching_group(
        |c, duel| c.is_able_to_remove(duel) && c.is_can_be_effect_target(duel, e),
        tp,
        LOCATION_MZONE,
        LOCATION_ONFIELD,
        None,
    );
    let rescon = |duel: &Duel, sg, mg, c| rescon(duel, sg, mg, c, tp);
    if chk == 0 {
        return Ok(aux::select_unselect_group_check(duel, g, 2, 2, &rescon));
    }
    let rg = aux::select_unselect_group(duel, g, 2, 2, &rescon, tp, HINTMSG_REMOVE)?;
    duel.set_target_card(rg);
    duel.set_operation_info(0, CATEGORY_REMOVE, Some(rg), 2, 0, 0);
    Ok(false)
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_target_cards(a.e).unwrap_or_default();
    if !g.is_empty() {
        duel.remove(g, POS_FACEUP, REASON_EFFECT)?;
    }
    Ok(())
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_faceup(duel) && c.is_relate_to_effect(duel, e) {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD_DISABLE);
        e1.set_value(duel, (duel.get_lp(tp) - duel.get_lp(1 - tp)).abs());
        c.register_effect(duel, e1);
    }
    Ok(())
}
