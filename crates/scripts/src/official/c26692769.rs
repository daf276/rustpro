// The Phantom Knights of Rusty Bardiche
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, MZONE};

const ID: u32 = 26692769;

pub const LISTED_SERIES: &[u16] = &[SET_THE_PHANTOM_KNIGHTS, SET_PHANTOM_KNIGHTS];
// Four copies, so the linked zones cover the Xyz Summons; Doomed Soleret (Xyz material, and the monster to send), Umbrage Veil
// (the "Phantom Knights" Trap to Set), Break Sword and Malevolent Scythe (the DARK Xyz Monsters).
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (83566725, MZONE),
    (83566725, MZONE),
    (83566725, MZONE),
    (83566725, DECK),
    (85257384, DECK),
    (62709239, EXTRA),
    (78449284, EXTRA),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_attribute_full(duel, ATTRIBUTE_DARK, Some(lc), sumtype, tp)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Link::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOGRAVE + CATEGORY_SET);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, descon);
    e2.set_target(duel, destg);
    e2.set_operation(duel, stock::destroy_first_target);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e3.set_code(duel, EFFECT_CANNOT_BE_LINK_MATERIAL);
    e3.set_value(duel, 1);
    c.register_effect(duel, e3);
}

fn setfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_PHANTOM_KNIGHTS]) && c.is_spell_trap(duel) && c.is_ssetable(duel)
}

fn tgfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_THE_PHANTOM_KNIGHTS])
        && c.is_monster(duel)
        && c.is_able_to_grave(duel)
        && duel.is_existing_matching_card(setfilter, tp, LOCATION_DECK, 0, 1, Some(c.into()))
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_SZONE) > 0
            && duel.is_existing_matching_card(|c, duel| tgfilter(c, duel, tp), tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, |c, duel| tgfilter(c, duel, tp), tp, LOCATION_DECK, 0, 1, 1, None)?;
    if g.is_empty() {
        return Ok(());
    }
    duel.send_to_grave(g, REASON_EFFECT)?;
    let og = duel
        .get_operated_group()
        .filter(duel, |c, duel| c.is_location(duel, LOCATION_GRAVE), None);
    if !og.is_empty() && duel.is_existing_matching_card(setfilter, tp, LOCATION_DECK, 0, 1, None) {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SET);
        if let Some(tc) = duel
            .select_matching_card(tp, setfilter, tp, LOCATION_DECK, 0, 1, 1, None)?
            .get_first()
        {
            duel.break_effect()?;
            duel.sset(tp, tc)?;
        }
    }
    Ok(())
}

fn descfilter(c: CardId, duel: &Duel, lg: CardSet) -> bool {
    c.is_faceup(duel) && c.is_type(duel, TYPE_XYZ) && c.is_attribute(duel, ATTRIBUTE_DARK) && lg.is_contains(c)
}

fn descon(duel: &Duel, a: &Args) -> bool {
    let lg = a.e.get_handler(duel).get_linked_group(duel);
    a.eg.is_some_and(|eg| eg.is_exists(duel, |c, duel| descfilter(c, duel, lg), 1, None))
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, PLAYER_ALL, LOCATION_ONFIELD as i32);
    Ok(false)
}
