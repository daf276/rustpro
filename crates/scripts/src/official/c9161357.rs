// Number 6: Chronomaly Atlandis
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE};

const ID: u32 = 9161357;
pub const XYZ_NUMBER: u32 = 6;

// In the Extra Deck, Xyz Summoned with Level 6 Normal Monsters, with Number monsters in the Graveyard to equip.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (6631034, MZONE),
    (8353769, MZONE),
    (9540040, MZONE),
    (11761845, MZONE),
    (84013237, GRAVE),
    (84013237, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(6);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_EQUIP);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_xyz_summoned(duel));
    e1.set_target(duel, eqtg);
    e1.set_operation(duel, eqop);
    c.register_effect(duel, e1);
    aux::add_er_equip_limit(duel, c, None, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_value(
        duel,
        Value::Card(|duel, _, c| {
            let sum = c
                .get_equip_group(duel)
                .filter(duel, |tc, duel| tc.has_flag_effect(duel, ID), None)
                .get_sum(duel, |tc, duel| i64::from(tc.get_attack(duel)));
            // Lua halves with a float, which the core rounds.
            (sum as f64 / 2.0).round() as i32
        }),
    );
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_cost(duel, |duel, a, chk| {
        Cost::and(duel, a, chk, &[Cost::detach_from_self::<1, 1>, halvelpcost])
    });
    e3.set_operation(duel, |duel, a| {
        let opp = 1 - a.tp;
        let lp = (f64::from(duel.get_lp(opp)) / 2.0).round() as i32;
        duel.set_lp(opp, lp);
        Ok(())
    });
    c.register_effect(duel, e3);
}

fn eqfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_NUMBER]) && c.is_monster(duel) && !c.is_forbidden(duel) && c.check_unique_on_field(duel, tp)
}

fn eqtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_GRAVE) && eqfilter(chkc, duel, tp));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_SZONE) > 0
            && duel.is_existing_target(|c, duel| eqfilter(c, duel, tp), tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_EQUIP);
    let g = duel.select_target(tp, |c, duel| eqfilter(c, duel, tp), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_EQUIP, g, 1, tp, 0);
    Ok(false)
}

fn eqop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c) = (a.e, a.e.get_handler(duel));
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if c.is_relate_to_effect(duel, e) && c.is_faceup(duel) && tc.is_relate_to_effect(duel, e) {
        c.equip_by_effect_and_limit_register(duel, e, a.tp, tc, Some(ID), None)?;
    }
    Ok(())
}

fn halvelpcostfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.has_flag_effect(duel, ID) && c.is_controler(duel, tp) && c.is_able_to_grave_as_cost(duel)
}

fn halvelpcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let eqg = c
        .get_equip_group(duel)
        .filter(duel, |tc, duel| halvelpcostfilter(tc, duel, tp), None);
    if chk == 0 {
        return Ok(duel.is_phase(PHASE_MAIN1) && !eqg.is_empty());
    }
    let g = if eqg.len() == 1 {
        eqg
    } else {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
        eqg.select(duel, tp, 1, 1, None)?
    };
    duel.send_to_grave(g, REASON_COST)?;
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_OATH + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_BP);
    e1.set_target_range(duel, 1, 0);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    Ok(false)
}
