// Number 101: Silent Honor ARK - Soul Asylum
use engine::constants::MSG_SELECT_EFFECTYN;

use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 2419596;
pub const XYZ_NUMBER: u32 = 101;
const CHOICE: u8 = 0;

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(4);
    const CT: usize = 2;
}

// Level 4 EARTH Warriors (Kojikocy, Kanan, Beautiful Headhuntress) to Xyz Summon it, in the Graveyard and Deck to add one with
// another name; Xyz Import takes Sangan as material to detach, and the opponent's monster to attach.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (1184620, MZONE),
    (12829151, MZONE),
    (16899564, MZONE),
    (1184620, MZONE),
    (1184620, GRAVE),
    (12829151, GRAVE),
    (12829151, DECK),
    (16899564, DECK),
    (14602126, HAND),
    (14602126, HAND),
    (16195942, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::Message(MSG_SELECT_EFFECTYN)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, efftg);
    e1.set_operation(duel, effop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_code(duel, EFFECT_DESTROY_REPLACE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target(duel, reptg);
    c.register_effect(duel, e2);
}

fn reptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(c.check_remove_overlay_card(duel, tp, 1, REASON_EFFECT) && !c.is_reason(duel, REASON_REPLACE));
    }
    Ok(duel.select_effect_yes_no_full(tp, c, 96)? && c.remove_overlay_card(duel, tp, 1, 1, REASON_EFFECT)? > 0)
}

fn thfilter(c: CardId, duel: &Duel, race: u64, attribute: u32, code: u32) -> bool {
    c.is_level(duel, &[4]) && c.is_race(duel, race) && c.is_attribute(duel, attribute) && !c.is_code(duel, code)
}

fn gyfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    let (race, attribute, code) = (c.get_race(duel), c.get_attribute(duel), c.get_code(duel));
    c.is_level(duel, &[4])
        && duel.is_existing_matching_card(|c, duel| thfilter(c, duel, race, attribute, code), tp, LOCATION_DECK, 0, 1, None)
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, c, tp) = (a.e, a.e.get_handler(duel), a.tp);
    let in_chain = duel.get_current_chain() > 0;
    let detached = in_chain.then(|| e.get_chain_data_group(duel, COST_DETACHED_MATERIALS));
    // The Lua indexes the detached materials of a chain link whose cost never ran.
    unported!(duel, matches!(detached, Some(None)));
    let exc = detached.flatten().and_then(|g| g.get_first());
    if let Some(chkc) = chkc {
        return Ok(in_chain
            && e.get_chain_data(duel, CHOICE) == Some(2)
            && chkc.is_location(duel, LOCATION_MZONE)
            && chkc.is_controler(duel, 1 - tp)
            && chkc.is_can_be_xyz_material(duel, Some(c), tp, REASON_EFFECT));
    }
    let option_1 = duel.is_existing_matching_card(|tc, duel| gyfilter(tc, duel, tp), tp, LOCATION_GRAVE, 0, 1, exc.map(Into::into));
    let option_2 = c.is_xyz_monster(duel)
        && duel.is_existing_target(
            |tc, duel| tc.is_can_be_xyz_material(duel, Some(c), tp, REASON_EFFECT),
            tp,
            0,
            LOCATION_MZONE,
            1,
            None,
        );
    if chk == 0 {
        return Ok(option_1 || option_2);
    }
    let choice = duel.select_effect(tp, &[(option_1, aux::stringid(ID, 1)), (option_2, aux::stringid(ID, 2))])?;
    if let Some(choice) = choice {
        e.set_chain_data(duel, CHOICE, choice as i64);
    }
    match choice {
        Some(1) => {
            e.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
            e.set_property(duel, 0);
            if let Some(exc) = exc {
                exc.create_effect_relation(duel, e);
            }
            duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
        }
        Some(2) => {
            e.set_category(duel, 0);
            e.set_property(duel, EFFECT_FLAG_CARD_TARGET);
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATTACH);
            duel.select_target(
                tp,
                |tc, duel| tc.is_can_be_xyz_material(duel, Some(c), tp, REASON_EFFECT),
                tp,
                0,
                LOCATION_MZONE,
                1,
                1,
                None,
            )?;
        }
        _ => {}
    }
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    match e.get_chain_data(duel, CHOICE) {
        Some(1) => {
            let detached_mat = e.get_chain_data_group(duel, COST_DETACHED_MATERIALS).and_then(|g| g.get_first());
            let exc = detached_mat.filter(|mat| mat.is_relate_to_effect(duel, e));
            duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 3));
            let gyg = duel.select_matching_card(
                tp,
                |c, duel| gyfilter(c, duel, tp),
                tp,
                LOCATION_GRAVE,
                0,
                1,
                1,
                exc.map(Into::into),
            )?;
            let Some(gyc) = gyg.get_first() else { return Ok(()) };
            duel.hint_selection(gyc);
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
            let (race, attribute, code) = (gyc.get_race(duel), gyc.get_attribute(duel), gyc.get_code(duel));
            let g = duel.select_matching_card(
                tp,
                |c, duel| thfilter(c, duel, race, attribute, code),
                tp,
                LOCATION_DECK,
                0,
                1,
                1,
                None,
            )?;
            if !g.is_empty() {
                duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
                duel.confirm_cards(1 - tp, g)?;
            }
        }
        Some(2) => {
            let c = e.get_handler(duel);
            if let Some(tc) = duel.get_first_target()
                && c.is_relate_to_effect(duel, e)
                && tc.is_relate_to_effect(duel, e)
                && tc.is_can_be_xyz_material(duel, Some(c), tp, REASON_EFFECT)
            {
                duel.overlay_full(c, tc, true)?;
            }
        }
        _ => {}
    }
    Ok(())
}
