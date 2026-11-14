// Primathmech Laplacian
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};
use engine::constants::MSG_SELECT_EFFECTYN;

const ID: u32 = 88021907;

pub const LISTED_SERIES: &[u16] = &[SET_MATHMECH];

const SELECTED_EFFECTS: u8 = 0;

// In the Extra Deck, Xyz Summoned with Level 4 Mathmechs (Diameter, Sigma, Nabla) and Normal Monsters (Jellyfish, Hyosube), with the
// opponent's Dark Hole to destroy the Mathmechs and monsters in the opponent's hand and field to send.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (17946349, MZONE),
    (27182739, MZONE),
    (14851496, MZONE),
    (14851496, MZONE),
    (2118022, MZONE),
    (2118022, MZONE),
    (53129443, HAND | OPPONENT),
    (26202165, HAND | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::Message(MSG_SELECT_EFFECTYN)];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(4);
    const CT: usize = 3;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EFFECT_DESTROY_REPLACE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target(duel, reptg);
    e1.set_value(
        duel,
        Value::Card(|duel, e, c| i32::from(repfilter(c, duel, e.get_handler_player(duel)))),
    );
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TOGRAVE);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).is_xyz_summoned(duel));
    e2.set_cost(duel, effcost);
    e2.set_target(duel, efftg);
    e2.set_operation(duel, effop);
    c.register_effect(duel, e2);
}

fn repfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_MATHMECH])
        && c.is_controler(duel, tp)
        && c.is_on_field(duel)
        && c.is_faceup(duel)
        && c.is_reason(duel, REASON_EFFECT)
        && !c.is_reason(duel, REASON_REPLACE)
}

fn reptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(a
            .eg
            .unwrap_or_default()
            .is_exists(duel, |sc, duel| repfilter(sc, duel, tp), 1, None)
            && c.check_remove_overlay_card(duel, tp, 1, REASON_EFFECT));
    }
    Ok(duel.select_effect_yes_no_full(tp, c, 96)? && c.remove_overlay_card(duel, tp, 1, 1, REASON_EFFECT)? > 0)
}

fn sttgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_spell_trap(duel) && c.is_able_to_grave(duel)
}

fn available_effects(duel: &Duel, tp: u8) -> [bool; 3] {
    [
        duel.is_existing_matching_card(CardId::is_able_to_grave, tp, 0, LOCATION_HAND, 1, None),
        duel.is_existing_matching_card(CardId::is_able_to_grave, tp, 0, LOCATION_MZONE, 1, None),
        duel.is_existing_matching_card(sttgfilter, tp, 0, LOCATION_ONFIELD, 1, None),
    ]
}

fn effcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let max = available_effects(duel, a.tp).into_iter().filter(|&available| available).count() as u16;
    Cost::detach_from_self_with(duel, a, chk, 1, max, |_, _, _| {})
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let (e, tp) = (a.e, a.tp);
    let [b1, b2, b3] = available_effects(duel, tp);
    let mut options = [(b1, aux::stringid(ID, 1)), (b2, aux::stringid(ID, 2)), (b3, aux::stringid(ID, 3))];
    let ct = e.get_chain_data_group(duel, COST_DETACHED_MATERIALS).unwrap_or_default().len();
    let (mut locations, mut selected) = (0, 0);
    for _ in 0..ct {
        let op = duel.select_effect(tp, &options)?;
        unported!(duel, op.is_none());
        let op = op.unwrap_or_default();
        if let Some(option) = options.get_mut(op - 1) {
            option.0 = false;
        }
        selected |= 1 << (op - 1);
        locations |= match op {
            1 => LOCATION_HAND,
            2 => LOCATION_MZONE,
            _ => LOCATION_ONFIELD,
        };
    }
    e.set_chain_data(duel, SELECTED_EFFECTS, selected);
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, ct as u8, 1 - tp, locations as i32);
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let selected = e.get_chain_data(duel, SELECTED_EFFECTS).unwrap_or_default();
    let mut break_chk = false;
    if selected & 1 != 0 {
        let g = duel.get_matching_group(CardId::is_able_to_grave, tp, 0, LOCATION_HAND, None);
        if !g.is_empty() {
            break_chk = true;
            let sg = g.random_select(duel, 1 - tp, 1);
            duel.send_to_grave(sg, REASON_EFFECT)?;
        }
    }
    for (bit, filter, location) in [
        (2, CardId::is_able_to_grave as fn(CardId, &Duel) -> bool, LOCATION_MZONE),
        (4, sttgfilter, LOCATION_ONFIELD),
    ] {
        if selected & bit != 0 {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
            let g = duel.select_matching_card(tp, filter, tp, 0, location, 1, 1, None)?;
            if !g.is_empty() {
                duel.hint_selection(g);
                if break_chk {
                    duel.break_effect()?;
                }
                break_chk = true;
                duel.send_to_grave(g, REASON_EFFECT)?;
            }
        }
    }
    Ok(())
}
