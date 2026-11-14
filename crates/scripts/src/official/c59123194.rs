// Enlightenment Paladin
use super::c95040215::Procedure;
use crate::prelude::*;
use crate::testing::{EXTRA, GRAVE, MZONE, OPPONENT};

const ID: u32 = 59123194;

pub const LISTED_SERIES: &[u16] = &[SET_MAGICIAN];
// In the Extra Deck, Synchro Summoned from Nobledragon Magician (Level 3 Tuner) and Stargazer Magician (Level 5), both Pendulum
// Magicians, with Spells in the Graveyard and weak monsters of the opponent's to battle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (88935103, MZONE),
    (88935103, MZONE),
    (88935103, MZONE),
    (88935103, MZONE),
    (94415058, MZONE),
    (94415058, MZONE),
    (94415058, MZONE),
    (94415058, MZONE),
    (46130346, GRAVE),
    (46130346, GRAVE),
    (1929294, MZONE | OPPONENT),
    (1929294, MZONE | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure<99>>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_TOHAND);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, thcon);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, stock::first_target_to_hand);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_category(duel, CATEGORY_DAMAGE);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_code(duel, EVENT_BATTLE_DESTROYING);
    e2.set_condition(duel, aux::bdocon);
    e2.set_target(duel, damtg);
    e2.set_operation(duel, damop);
    c.register_effect(duel, e2);
}

fn thcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_synchro_summoned(duel) && c.get_material(duel).is_exists(duel, |mc, duel| pmfilter(mc, duel, c), 1, None)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_spell(duel) && c.is_able_to_hand(duel)
}

fn pmfilter(c: CardId, duel: &Duel, sc: CardId) -> bool {
    c.is_set_card(duel, &[SET_MAGICIAN]) && c.is_type_full(duel, TYPE_PENDULUM, Some(sc), SUMMON_TYPE_SYNCHRO as u64, PLAYER_NONE)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && thfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(thfilter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_target(tp, thfilter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOHAND, g, 1, 0, 0);
    Ok(false)
}

fn damtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let Some(bc) = a.e.get_handler(duel).get_battle_target(duel) else {
        return Ok(false);
    };
    let dam = bc.get_text_attack(duel);
    if chk == 0 {
        return Ok(dam > 0);
    }
    duel.set_target_card(bc);
    duel.set_target_player(1 - tp);
    duel.set_target_param(dam as u32);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, dam);
    Ok(false)
}

fn damop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
        let dam = tc.get_text_attack(duel).max(0);
        duel.damage(p, dam as u32, REASON_EFFECT)?;
    }
    Ok(())
}
