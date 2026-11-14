// Windwitch - Diamond Bell
use super::c14577226::Procedure;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE};

const ID: u32 = 73667937;

pub const LISTED_SERIES: &[u16] = &[SET_WINDWITCH];
// In the Extra Deck, Synchro Summoned with Windwitch - Blizzard Bell (a Level 5 Tuner) and Wicked Dragon with the Ersatz Head (WIND,
// Level 3), with Windwitch monsters in the Graveyard to burn with and a second Synchro Summon to damage the opponent.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (84851250, MZONE),
    (84851250, MZONE),
    (2957055, MZONE),
    (2957055, MZONE),
    (70117860, GRAVE),
    (71007216, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DAMAGE);
    e1.set_type(duel, EFFECT_TYPE_TRIGGER_O + EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_DELAY);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_synchro_summoned(duel));
    e1.set_target(duel, damtg);
    e1.set_operation(duel, damop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY + EFFECT_FLAG_DAMAGE_STEP);
    e2.set_code(duel, EVENT_DAMAGE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, |_, a| a.ep == 1 - a.tp && (a.r & (REASON_EFFECT + REASON_BATTLE)) != 0);
    e2.set_target(duel, destg);
    e2.set_operation(duel, stock::destroy_first_target);
    c.register_effect(duel, e2);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_SINGLE);
    e4.set_code(duel, EFFECT_MATERIAL_CHECK);
    e4.set_value(duel, Value::MaterialCheck(valcheck));
    c.register_effect(duel, e4);
}

fn damfil(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel) && c.is_set_card(duel, &[SET_WINDWITCH]) && c.is_attack_above(duel, 1)
}

fn damtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_GRAVE) && damfil(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(damfil, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    let g = duel.select_target(tp, damfil, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    let d = g.and_then(|g| g.get_first()).map_or(0, |tc| tc.get_attack(duel) / 2);
    duel.set_target_player(1 - tp);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, d);
    Ok(false)
}

fn damop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    if tc.is_relate_to_effect(duel, a.e) {
        duel.damage(p, (tc.get_attack(duel) / 2) as u32, REASON_EFFECT)?;
    }
    Ok(())
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel));
    }
    if chk == 0 {
        let wwchk = if c.is_synchro_summoned(duel) {
            c.get_flag_effect(duel, ID) + 1
        } else {
            1
        };
        let ct = c.get_flag_effect(duel, ID + 1);
        return Ok(ct < wwchk && duel.is_existing_target(aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None));
    }
    c.register_flag_effect(duel, ID + 1, RESETS_STANDARD_PHASE_END, 0, 1);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, 0, 0);
    Ok(false)
}

fn valcheck(duel: &mut Duel, e: EffectId, c: CardId) {
    let g = c.get_material(duel);
    let ct = g.filter_count(duel, |c, duel| c.is_set_card(duel, &[SET_WINDWITCH]), None);
    if ct == g.len() {
        e.get_handler(duel)
            .register_flag_effect(duel, ID, RESETS_STANDARD - RESET_TOFIELD, 0, 1);
    }
}
