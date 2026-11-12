// Zubaba General
use crate::prelude::*;
use crate::testing::{EXTRA, HAND, MZONE};

const ID: u32 = 31563350;

// Twice in the Extra Deck, Xyz Summoned with Kojikocy (Level 4 Warriors), with more in the hand to equip.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (1184620, MZONE),
    (1184620, MZONE),
    (1184620, MZONE),
    (1184620, HAND),
    (1184620, HAND),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(4);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_category(duel, CATEGORY_EQUIP);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, eqtg);
    e1.set_operation(duel, eqop);
    c.register_effect(duel, e1);
    aux::add_er_equip_limit(duel, c, None, e1);
}

fn filter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_race(duel, RACE_WARRIOR) && c.check_unique_on_field(duel, tp) && !c.is_forbidden(duel)
}

fn eqtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_SZONE) > 0
            && duel.is_existing_matching_card(|c, duel| filter(c, duel, tp), tp, LOCATION_HAND, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_EQUIP, None, 1, tp, LOCATION_HAND as i32);
    Ok(false)
}

fn equipop(duel: &mut Duel, c: CardId, e: EffectId, tp: u8, tc: CardId) -> Script<()> {
    if !c.equip_by_effect_and_limit_register(duel, e, tp, tc, None, Some(true))? {
        return Ok(());
    }
    let atk = tc.get_text_attack(duel);
    if atk > 0 {
        let e2 = duel.create_effect(c);
        e2.set_type(duel, EFFECT_TYPE_EQUIP);
        e2.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE + EFFECT_FLAG_OWNER_RELATE);
        e2.set_code(duel, EFFECT_UPDATE_ATTACK);
        e2.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        e2.set_value(duel, atk);
        tc.register_effect(duel, e2);
    }
    Ok(())
}

fn eqop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if duel.get_location_count(tp, LOCATION_SZONE) <= 0 || c.is_facedown(duel) || !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_EQUIP);
    let g = duel.select_matching_card(tp, |c, duel| filter(c, duel, tp), tp, LOCATION_HAND, 0, 1, 1, None)?;
    if let Some(tc) = g.get_first() {
        equipop(duel, c, e, tp, tc)?;
    }
    Ok(())
}
