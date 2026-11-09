// DoomZ XII End - Drastrius
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 95626382;

// In the Extra Deck, Xyz Summoned with three Level 8 monsters (Dark Magician's Level 7 does not fit; Drastea and Blue-Eyes
// White Dragon do), with the opponent's monsters to equip and negate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (92472273, MZONE),
    (92472273, MZONE),
    (89631139, MZONE),
    (89631139, MZONE),
    (7892180, MZONE | OPPONENT),
    (59053232, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(8);
    const CT: usize = 3;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_COUNT);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    // The script's `r&REASON_BATTLE|REASON_EFFECT` binds as `(r&REASON_BATTLE)|REASON_EFFECT`, which is always true.
    e1.set_value(duel, Value::Reason(|_, _, _, _, _| true));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_EQUIP);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).is_xyz_summoned(duel));
    e2.set_target(duel, eqtg);
    e2.set_operation(duel, eqop);
    c.register_effect(duel, e2);
    aux::add_er_equip_limit(duel, c, None, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_DISABLE + CATEGORY_EQUIP);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_code(duel, EVENT_CHAINING);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_condition(duel, discon);
    e3.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e3.set_target(duel, distg);
    e3.set_operation(duel, disop);
    c.register_effect(duel, e3);
    aux::add_er_equip_limit(duel, c, None, e3);
}

fn eqfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_faceup(duel) && (c.is_controler(duel, tp) || c.is_able_to_change_controler(duel))
}

fn eqtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_SZONE) > 0
            && duel.is_existing_matching_card(
                |c, duel| eqfilter(c, duel, tp),
                tp,
                LOCATION_MZONE,
                LOCATION_MZONE,
                1,
                Some(c.into()),
            ));
    }
    duel.set_operation_info(0, CATEGORY_EQUIP, None, 1, PLAYER_EITHER, LOCATION_MZONE as i32);
    Ok(false)
}

fn eqop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_facedown(duel) || !c.is_relate_to_effect(duel, e) || duel.get_location_count(tp, LOCATION_SZONE) == 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_EQUIP);
    let filter = |c, duel: &Duel| eqfilter(c, duel, tp);
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, Some(c.into()))?;
    if let Some(ec) = g.get_first() {
        duel.hint_selection(ec);
        c.equip_by_effect_and_limit_register(duel, e, tp, ec, None, None)?;
    }
    Ok(())
}

fn discon(duel: &Duel, a: &Args) -> bool {
    let trig_loc: u32 = duel.get_chain_info(a.ev as u8, CHAININFO_TRIGGERING_LOCATION);
    a.ep == 1 - a.tp
        && a.re.is_some_and(|re| re.is_monster_effect(duel))
        && trig_loc & (LOCATION_MZONE | LOCATION_GRAVE) > 0
        && duel.is_chain_disablable(a.ev as u8)
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_DISABLE, a.eg, 1, tp, 0);
    duel.set_possible_operation_info(0, CATEGORY_EQUIP, None, 1, 1 - tp, LOCATION_MZONE as i32);
    Ok(false)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if !duel.negate_effect(a.ev as u8) {
        return Ok(());
    }
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) || c.is_facedown(duel) {
        return Ok(());
    }
    let g = duel.get_matching_group(|c, duel| eqfilter(c, duel, tp), tp, 0, LOCATION_MZONE, Some(c.into()));
    if g.is_empty() || !duel.select_yes_no(tp, aux::stringid(ID, 2))? {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_EQUIP);
    if let Some(ec) = g.select(duel, tp, 1, 1, None)?.get_first() {
        duel.hint_selection(ec);
        duel.break_effect()?;
        c.equip_by_effect_and_limit_register(duel, e, tp, ec, None, None)?;
    }
    Ok(())
}
