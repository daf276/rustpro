// LeVirtue Dragon
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, MZONE};

const ID: u32 = 33113958;
const VIRTUE_STREAM: u32 = 80534031;

pub const LISTED_NAMES: &[u32] = &[ID, VIRTUE_STREAM];

// Xyz Summoned from the Extra Deck with Level 3 monsters, with Virtue Stream in the Deck to search, Atlantean Pikeman
// (WATER Aqua) in the Graveyard to add, and a second Xyz Monster on the field to move a material to.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (32864, MZONE),
    (1761063, MZONE),
    (2468169, MZONE),
    (VIRTUE_STREAM, DECK),
    (VIRTUE_STREAM, DECK),
    (26976414, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(3);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, efftg);
    e2.set_operation(duel, effop);
    c.register_effect(duel, e2);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, VIRTUE_STREAM) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}

fn thgyfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_FISH | RACE_SEASERPENT | RACE_AQUA) && c.is_able_to_hand(duel) && !c.is_code(duel, ID)
}

fn xyzfilter(c: CardId, duel: &Duel, e: EffectId) -> bool {
    c.is_type(duel, TYPE_XYZ) && c.is_faceup(duel) && c.is_can_be_effect_target(duel, e)
}

fn overlayfilter(c: CardId, duel: &Duel) -> bool {
    c.get_overlay_count(duel) > 0
}

fn rescon(duel: &Duel, sg: CardSet, _: CardSet, _: Option<CardId>) -> (bool, bool) {
    (sg.is_exists(duel, overlayfilter, 1, None), false)
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let b1 = duel.check_remove_overlay_card(tp, 1, 0, 1, REASON_EFFECT)
        && duel.is_existing_matching_card(thgyfilter, tp, LOCATION_GRAVE, 0, 1, None);
    let g = duel.get_matching_group(|c, duel| xyzfilter(c, duel, e), tp, LOCATION_MZONE, LOCATION_MZONE, None);
    let b2 = g.len() >= 2 && aux::select_unselect_group_check(duel, g, 2, 2, &rescon);
    if chk == 0 {
        return Ok(b1 || b2);
    }
    let op = duel.select_effect(tp, &[(b1, aux::stringid(ID, 2)), (b2, aux::stringid(ID, 3))])?;
    e.set_label(duel, op.map_or(0, |op| op as i64));
    if op == Some(1) {
        e.set_category(duel, CATEGORY_TOHAND);
        e.set_property(duel, 0);
        duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_GRAVE as i32);
    } else if op == Some(2) {
        e.set_category(duel, 0);
        e.set_property(duel, EFFECT_FLAG_CARD_TARGET);
        let g = aux::select_unselect_group(duel, g, 2, 2, &rescon, tp, HINTMSG_TARGET)?;
        duel.set_target_card(g);
    }
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let op = e.get_label(duel);
    if op == 1 {
        if duel.remove_overlay_card(tp, 1, 0, 1, 1, REASON_EFFECT)? == 0 {
            return Ok(());
        }
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
        let g = duel.select_matching_card(tp, thgyfilter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
        if !g.is_empty() {
            duel.break_effect()?;
            duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
            duel.confirm_cards(1 - tp, g)?;
        }
    } else if op == 2 {
        let mut g = duel.get_target_cards(e).unwrap_or_default();
        if g.len() != 2 {
            return Ok(());
        }
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_DEATTACHFROM);
        let Some(detachxyz) = g.filter_select(duel, tp, overlayfilter, 1, 1, None)?.get_first() else {
            return Ok(());
        };
        g.remove(detachxyz);
        let Some(attachxyz) = g.get_first() else { return Ok(()) };
        let mut attach_group = detachxyz.get_overlay_group(duel);
        if !attach_group.is_empty() {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATTACH);
            attach_group = attach_group.select(duel, tp, 1, 1, None)?;
        }
        duel.overlay(attachxyz, attach_group)?;
        duel.raise_single_event(detachxyz, EVENT_DETACH_MATERIAL, Some(e), 0, 0, 0, 0)?;
        duel.raise_event(detachxyz, EVENT_DETACH_MATERIAL, Some(e), REASON_EFFECT, tp, tp, 0)?;
    }
    Ok(())
}
