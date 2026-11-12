// Mental Tuner
use crate::prelude::*;
use crate::testing::{GRAVE, HAND, MZONE, REMOVED};

const ID: u32 = 30291086;
const CHOICE: u8 = 0;
const BANISHED_AMOUNT: u8 = 1;

// On the field with LIGHT (Gradius) and DARK (Mechanicalchaser) Normal Monsters in the hand, Graveyard and banished to cost or return.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (10992251, HAND),
    (7359741, HAND),
    (10992251, GRAVE),
    (7359741, GRAVE),
    (10992251, REMOVED),
    (7359741, REMOVED),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, lvtg);
    e1.set_operation(duel, lvop);
    c.register_effect(duel, e1);
}

fn rmfilter(c: CardId, duel: &Duel) -> bool {
    c.is_attribute(duel, ATTRIBUTE_LIGHT | ATTRIBUTE_DARK)
        && c.is_able_to_remove_as_cost(duel)
        && (c.is_location(duel, LOCATION_HAND) || aux::sp_elim_filter(c, duel, true, false))
}

fn tgfilter(c: CardId, duel: &Duel, e: EffectId) -> bool {
    c.is_faceup(duel) && c.is_attribute(duel, ATTRIBUTE_LIGHT | ATTRIBUTE_DARK) && c.is_can_be_effect_target(duel, e)
}

fn attrescon(duel: &Duel, sg: CardSet) -> (bool, bool) {
    (
        sg.get_class_count(duel, |c, duel| i64::from(c.get_attribute(duel))) == sg.len(),
        false,
    )
}

fn lvtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let rmg = duel.get_matching_group(rmfilter, tp, LOCATION_MZONE | LOCATION_HAND | LOCATION_GRAVE, 0, Some(c.into()));
    let tgg = duel.get_matching_group(|c, duel| tgfilter(c, duel, e), tp, LOCATION_REMOVED, 0, None);
    if chk == 0 {
        return Ok(c.has_level(duel) && (!rmg.is_empty() || !tgg.is_empty()));
    }
    e.set_category(duel, 0);
    e.set_property(duel, 0);
    let choice = duel.select_effect(
        tp,
        &[(!rmg.is_empty(), aux::stringid(ID, 1)), (!tgg.is_empty(), aux::stringid(ID, 2))],
    )?;
    e.set_chain_data(duel, CHOICE, choice.map_or(0, |choice| choice as i64));
    if choice == Some(1) {
        let rg = aux::select_unselect_group(duel, rmg, 1, 2, &|duel, sg, _, _| attrescon(duel, sg), tp, HINTMSG_REMOVE)?;
        duel.remove(rg, POS_FACEUP, REASON_COST)?;
        let banished = duel
            .get_operated_group()
            .filter_count(duel, |c, duel| c.is_location(duel, LOCATION_REMOVED), None);
        e.set_chain_data(duel, BANISHED_AMOUNT, banished as i64);
    } else if choice == Some(2) {
        e.set_category(duel, CATEGORY_TOGRAVE);
        e.set_property(duel, EFFECT_FLAG_CARD_TARGET);
        let tg = aux::select_unselect_group(duel, tgg, 1, 2, &|duel, sg, _, _| attrescon(duel, sg), tp, HINTMSG_TOGRAVE)?;
        duel.set_target_card(tg);
        duel.set_operation_info(0, CATEGORY_TOGRAVE, Some(tg), tg.len() as u8, 0, 0);
    }
    Ok(false)
}

fn lvop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let choice = e.get_chain_data(duel, CHOICE);
    let mut ct = e.get_chain_data(duel, BANISHED_AMOUNT);
    if choice == Some(2) {
        let tg = duel.get_target_cards(e).unwrap_or_default();
        if !tg.is_empty() {
            ct = Some(i64::from(duel.send_to_grave(tg, REASON_EFFECT | REASON_RETURN)?));
        }
    } else if choice != Some(1) {
        return Ok(());
    }
    let Some(ct) = ct.filter(|&ct| ct != 0) else { return Ok(()) };
    let c = e.get_handler(duel);
    if !(c.is_relate_to_effect(duel, e) && c.is_faceup(duel) && c.has_level(duel)) {
        return Ok(());
    }
    let b1 = c.is_level_above(duel, ct as u32 + 1);
    let lvop = duel.select_effect(tp, &[(b1, aux::stringid(ID, 3)), (true, aux::stringid(ID, 4))])?;
    if let Some(lvop) = lvop {
        c.update_level(
            duel,
            if lvop == 1 { ct as i32 } else { -ct as i32 },
            Some(RESETS_STANDARD_DISABLE_PHASE_END),
            None,
        );
    }
    Ok(())
}
