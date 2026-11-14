// Number C79: Battlin' Boxer General Kaiser
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 10300821;

pub const LISTED_SERIES: &[u16] = &[SET_BATTLIN_BOXER];
pub const XYZ_NUMBER: u32 = 79;
const NOVA_KAISER: u32 = 71921856;

pub const LISTED_NAMES: &[u32] = &[NOVA_KAISER];

// In the Extra Deck, Xyz Summoned with three Level 5 Normal Monsters, and Special Summoning cards in the opponent's hand.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (2971090, MZONE),
    (4035199, MZONE),
    (4179849, MZONE),
    (26202165, HAND | OPPONENT),
    (14558127, HAND | OPPONENT),
];
// The attach effect needs "Number 79: Battlin' Boxer Nova Kaiser" as material, which no procedure of a Level 5 Xyz Monster allows.
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 1))),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(5);
    const CT: usize = 3;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(duel, Value::Card(|duel, _, c| c.get_overlay_count(duel) as i32 * 200));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_DISABLE_SUMMON + CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_SPSUMMON);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_condition(duel, |duel, a| a.tp == 1 - a.ep && duel.get_current_chain_full(true) == 0);
    e2.set_target(duel, negsumtg);
    e2.set_operation(duel, negsumop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_TOGRAVE);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_condition(duel, tgattachcon);
    e3.set_target(duel, tgattachtg);
    e3.set_operation(duel, tgattachop);
    c.register_effect(duel, e3);
}

fn negsumtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(a.e.get_handler(duel).check_remove_overlay_card(duel, a.tp, 2, REASON_EFFECT));
    }
    let count = a.eg.map_or(0, |eg| eg.len()) as u8;
    duel.set_operation_info(0, CATEGORY_DISABLE_SUMMON, a.eg, count, a.tp, 0);
    duel.set_operation_info(0, CATEGORY_DESTROY, a.eg, count, a.tp, 0);
    Ok(false)
}

fn negsumop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp, c) = (a.e, a.tp, a.e.get_handler(duel));
    if c.is_relate_to_effect(duel, e) && c.remove_overlay_card(duel, tp, 2, 2, REASON_EFFECT)? == 2 {
        let eg = a.eg.unwrap_or_default();
        duel.negate_summon(eg);
        duel.destroy(eg, REASON_EFFECT)?;
    }
    Ok(())
}

fn tgattachcon(duel: &Duel, a: &Args) -> bool {
    let overlay = a.e.get_handler(duel).get_overlay_group(duel);
    if !overlay.is_exists(duel, |c, duel| c.is_code(duel, NOVA_KAISER), 1, None) {
        return false;
    }
    let (mine, theirs) = duel.get_battle_monster(a.tp);
    theirs.is_some() && mine.is_some_and(|mine| mine.is_faceup(duel) && mine.is_set_card(duel, &[SET_BATTLIN_BOXER]))
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_BATTLIN_BOXER]) && c.is_monster(duel) && c.is_able_to_grave(duel)
}

fn tgattachtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let (_, b) = duel.get_battle_monster(tp);
    unported!(duel, b.is_none());
    let Some(b) = b else { return Ok(false) };
    if chk == 0 {
        return Ok(b.is_able_to_change_controler(duel)
            && b.is_can_be_xyz_material(duel, Some(e.get_handler(duel)), tp, REASON_EFFECT)
            && duel.is_existing_matching_card(tgfilter, tp, LOCATION_HAND | LOCATION_DECK, 0, 1, None));
    }
    duel.set_target_card(b);
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, (LOCATION_HAND | LOCATION_DECK) as i32);
    Ok(false)
}

fn tgattachop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, tgfilter, tp, LOCATION_HAND | LOCATION_DECK, 0, 1, 1, None)?;
    if let Some(first) = g.get_first()
        && duel.send_to_grave(g, REASON_EFFECT)? > 0
        && first.is_location(duel, LOCATION_GRAVE)
    {
        let c = e.get_handler(duel);
        let tc = duel.get_first_target();
        unported!(duel, tc.is_none());
        if let Some(tc) = tc
            && tc.is_relate_to_effect(duel, e)
            && tc.is_controler(duel, 1 - tp)
            && !tc.is_immune_to_effect(duel, e)
            && c.is_relate_to_effect(duel, e)
        {
            duel.overlay_full(c, tc, true)?;
        }
    }
    Ok(())
}
