// Gaming Gamer GG
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, MZONE, OPPONENT};

const ID: u32 = 15778492;

// In the Extra Deck, Xyz Summoned with Level 4 Machines (Overdrive, Mechanicalchaser, Golgoil), with Machines in the Deck to send,
// Machine Xyz Monsters in the Graveyard to copy the name of and Sangan on the opponent's side to flip to Attack Position.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (2311603, MZONE),
    (7359741, MZONE),
    (7526150, MZONE),
    (7526150, MZONE),
    (2311603, DECK),
    (7359741, DECK),
    (2091298, GRAVE),
    (11646785, GRAVE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> = Some(|c, duel, scard, sumtype, tp| c.is_race_full(duel, RACE_MACHINE, Some(scard), sumtype, tp));
    const LV: Option<u32> = Some(4);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_POSITION);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_target(duel, postg);
    e1.set_operation(duel, posop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOGRAVE);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, |duel, a| {
        duel.is_existing_matching_card(CardId::is_monster, a.tp, 0, LOCATION_MZONE | LOCATION_GRAVE, 1, None)
    });
    e2.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e2.set_target(duel, tgtg);
    e2.set_operation(duel, tgop);
    c.register_effect(duel, e2);
}

fn posfilter(c: CardId, duel: &Duel) -> bool {
    c.is_defense_pos(duel) && c.is_can_change_position(duel)
}

fn postg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let g = duel.get_matching_group(posfilter, tp, 0, LOCATION_MZONE, None);
    if chk == 0 {
        return Ok(!g.is_empty());
    }
    duel.set_operation_info(0, CATEGORY_POSITION, Some(g), g.len() as u8, tp, POS_FACEUP_ATTACK as i32);
    Ok(false)
}

fn posop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_matching_group(posfilter, a.tp, 0, LOCATION_MZONE, None);
    if !g.is_empty() {
        duel.change_position(g, POS_FACEUP_ATTACK)?;
    }
    Ok(())
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_MACHINE) && c.is_able_to_grave(duel)
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, tgfilter, LOCATION_DECK | LOCATION_EXTRA, CATEGORY_TOGRAVE)
}

fn namefilter(c: CardId, duel: &Duel, current_name: u32) -> bool {
    c.is_race(duel, RACE_MACHINE) && c.is_xyz_monster(duel) && !c.is_code(duel, current_name)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let code = c.get_code(duel);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, tgfilter, tp, LOCATION_DECK | LOCATION_EXTRA, 0, 1, 1, None)?;
    if let Some(sc) = g.get_first()
        && duel.send_to_grave(sc, REASON_EFFECT)? > 0
        && sc.is_location(duel, LOCATION_GRAVE)
        && c.is_relate_to_effect(duel, e)
        && c.is_faceup(duel)
        && duel.is_existing_matching_card(|c, duel| namefilter(c, duel, code), tp, LOCATION_GRAVE, 0, 1, None)
        && duel.select_yes_no(tp, aux::stringid(ID, 2))?
    {
        duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 3));
        let g = duel.select_matching_card(tp, |c, duel| namefilter(c, duel, code), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
        let Some(xyz) = g.get_first() else { return Ok(()) };
        duel.hint_selection(xyz);
        duel.break_effect()?;
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_code(duel, EFFECT_CHANGE_CODE);
        e1.set_value(duel, xyz.get_code(duel) as i32);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        c.register_effect(duel, e1);
        let e2 = duel.create_effect(c);
        e2.set_description(duel, aux::stringid(ID, 4));
        e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e2.set_code(duel, EVENT_PHASE + PHASE_END);
        e2.set_range(duel, LOCATION_MZONE);
        e2.set_count_limit(duel, 1, 0);
        e2.set_operation(duel, resetop);
        e2.set_label_object(duel, Some(e1));
        e2.set_reset(duel, RESETS_STANDARD_PHASE_END);
        c.register_effect(duel, e2);
    }
    Ok(())
}

fn resetop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if let Some(e1) = e.get_label_object(duel) {
        e1.reset(duel);
    }
    duel.hint_selection(e.get_handler(duel));
    duel.hint(HINT_OPSELECTED, 1 - tp, e.get_description(duel));
    Ok(())
}
