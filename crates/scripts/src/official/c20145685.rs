// Xyz Armor Fortress
use crate::prelude::*;
use crate::testing::{Check, DECK, MZONE};

const ID: u32 = 20145685;

pub const LISTED_SERIES: &[u16] = &[SET_ARMORED_XYZ];
pub const LISTED_NAMES: &[u32] = &[ID];

// Tornado Dragon to Xyz Summon on, and Armored Xyz, Full-Armored Xyz and Reincarnation Unveiling Mail in the Deck to search.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (6983839, MZONE),
    (73046708, DECK),
    (73046708, DECK),
    (39733924, DECK),
    (68663427, DECK),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 1))];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(5);
    const CT: usize = 2;
    const MAXCT: usize = 2;
    const ALTERF: Option<AlterFilter> = Some(xyzfilter);
    const DESC: u64 = aux::stringid(ID, 0);

    fn op_check(duel: &Duel, _: EffectId, tp: u8, _: CardId) -> bool {
        !duel.has_flag_effect(tp, ID)
    }

    fn op(duel: &mut Duel, _: EffectId, tp: u8, _: CardId) -> Script<bool> {
        duel.register_flag_effect(tp, ID, RESET_PHASE | PHASE_END, EFFECT_FLAG_OATH, 1);
        Ok(true)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_SINGLE);
    e0.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e0.set_code(duel, EFFECT_CANNOT_BE_XYZ_MATERIAL);
    e0.set_condition(duel, |duel, a| a.e.get_handler(duel).get_overlay_count(duel) > 0);
    e0.set_value(duel, 1);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_category(duel, CATEGORY_SEARCH + CATEGORY_TOHAND);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_cost(duel, thcost);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_EQUIP);
    e2.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    e2.set_code(duel, EFFECT_CHANGE_BATTLE_DAMAGE);
    e2.set_condition(duel, |duel, a| {
        a.e.get_handler(duel)
            .get_equip_target(duel)
            .is_some_and(|ec| ec.get_battle_target(duel).is_some())
    });
    e2.set_value(duel, Value::BattleDamage(aux::change_battle_damage::<1, DOUBLE_DAMAGE>));
    c.register_effect(duel, e2);
}

fn xyzfilter(c: CardId, duel: &Duel, tp: u8, xyzc: CardId) -> bool {
    c.is_rank(duel, &[3, 4]) && c.is_type_full(duel, TYPE_XYZ, Some(xyzc), SUMMON_TYPE_XYZ as u64, tp) && c.is_faceup(duel)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_ARMORED_XYZ]) && c.is_able_to_hand(duel)
}

fn thmaxcost(duel: &Duel, tp: u8) -> u16 {
    let classes = duel
        .get_matching_group(thfilter, tp, LOCATION_DECK, 0, None)
        .get_class_count(duel, |c, duel| i64::from(c.get_code(duel)));
    classes.min(2) as u16
}

fn thcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let max = thmaxcost(duel, a.tp);
    Cost::detach_from_self_with(duel, a, chk, 1, max, |duel, e, og| e.set_label(duel, og.len() as i64))
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let count = a.e.get_label(duel) as u8;
    duel.set_operation_info(0, CATEGORY_TOHAND, None, count, a.tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let ct = a.e.get_label(duel) as usize;
    let g = duel.get_matching_group(thfilter, tp, LOCATION_DECK, 0, None);
    if g.len() < ct {
        return Ok(());
    }
    let thg = aux::select_unselect_group(duel, g, ct, ct, &aux::dncheck, tp, HINTMSG_ATOHAND)?;
    if !thg.is_empty() {
        duel.send_to_hand(thg, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, thg)?;
    }
    Ok(())
}
