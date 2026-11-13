// Puzzlomino, the Drop-n-Deleter
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 84271823;
const LEVEL: u8 = 0;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;

    fn f(c: CardId, duel: &Duel, _: CardId, _: u64, _: u8) -> bool {
        mfilter(c, duel)
    }

    fn specialchk(duel: &Duel, sg: CardSet, lc: CardId, sumtype: u64, tp: u8) -> bool {
        matcheck(duel, sg, lc, sumtype, tp)
    }
}

// In the Extra Deck, Link Summoned from monsters of different Levels, with monsters of the same Level on both fields to destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (14181608, MZONE),
    (14708569, MZONE),
    (91152256, MZONE),
    (14181608, MZONE | OPPONENT),
    (14708569, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_LVCHANGE);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, lvcon);
    e1.set_target(duel, lvtg);
    e1.set_operation(duel, lvop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
}

fn mfilter(c: CardId, duel: &Duel) -> bool {
    c.has_level(duel) && !c.is_level(duel, &[0])
}

fn matcheck(duel: &Duel, g: CardSet, _: CardId, _: u64, _: u8) -> bool {
    g.get_class_count(duel, |c, duel| i64::from(c.get_level(duel))) == g.len()
}

fn cfilter(c: CardId, duel: &Duel, tp: u8, lg: CardSet) -> bool {
    c.is_level_above(duel, 1) && c.is_faceup(duel) && c.is_controler(duel, tp) && lg.is_contains(c)
}

fn lvcon(duel: &Duel, a: &Args) -> bool {
    let (tp, lg) = (a.tp, a.e.get_handler(duel).get_linked_group(duel));
    a.eg.unwrap_or_default()
        .is_exists(duel, |c, duel| cfilter(c, duel, tp, lg), 1, None)
}

fn lvtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(true);
    }
    let lg = e.get_handler(duel).get_linked_group(duel);
    let g = a.eg.unwrap_or_default().filter(duel, |c, duel| cfilter(c, duel, tp, lg), None);
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 2));
    let level = match g.get_first() {
        Some(tc) if g.len() == 1 => {
            let excluded = u64::from(tc.get_level(duel));
            duel.announce_number(tp, &(1..=8).filter(|&n| n != excluded).collect::<Vec<_>>())?
        }
        _ => duel.announce_number_range(tp, 1, 8)?,
    };
    e.set_chain_data(duel, LEVEL, level as i64);
    duel.set_operation_info(0, CATEGORY_LVCHANGE, Some(g), g.len() as u8, 0, level as i32);
    Ok(false)
}

fn lvop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let lg = c.get_linked_group(duel);
    let g = a.eg.unwrap_or_default().filter(duel, |c, duel| cfilter(c, duel, tp, lg), None);
    let level = e.get_chain_data(duel, LEVEL).unwrap_or(0);
    for tc in g {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_CHANGE_LEVEL);
        e1.set_value(duel, level as i32);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        tc.register_effect(duel, e1);
    }
    Ok(())
}

fn desfilter(c: CardId, duel: &Duel, e: EffectId) -> bool {
    c.is_faceup(duel) && c.has_level(duel) && c.is_can_be_effect_target(duel, e)
}

fn rescon(duel: &Duel, sg: CardSet) -> (bool, bool) {
    (
        sg.get_class_count(duel, |c, duel| i64::from(c.get_controler(duel))) == 2
            && sg.get_class_count(duel, |c, duel| i64::from(c.get_level(duel))) == 1,
        false,
    )
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chkc.is_some() {
        return Ok(false);
    }
    let g = duel.get_matching_group(|c, duel| desfilter(c, duel, e), 0, LOCATION_MZONE, LOCATION_MZONE, None);
    if chk == 0 {
        return Ok(g.len() >= 2 && aux::select_unselect_group_check(duel, g, 2, 2, &|duel, sg, _, _| rescon(duel, sg)));
    }
    let dg = aux::select_unselect_group(duel, g, 2, 2, &|duel, sg, _, _| rescon(duel, sg), tp, HINTMSG_DESTROY)?;
    duel.set_target_card(dg);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(dg), dg.len() as u8, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let dg = duel.get_target_cards(a.e).unwrap_or_default();
    if !dg.is_empty() {
        duel.destroy(dg, REASON_EFFECT)?;
    }
    Ok(())
}
