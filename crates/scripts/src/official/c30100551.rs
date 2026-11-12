// Minerva, the Exalted Lightsworn
use crate::prelude::*;
use crate::testing::{DECK, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 30100551;

// In the Extra Deck, Xyz Summoned with two of four Level 4 monsters, with Lightsworn monsters in the Deck to send, and destroyed by
// the opponent's Raigeki.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (2118022, MZONE),
    (2483611, MZONE),
    (4042268, MZONE),
    (5628232, MZONE),
    (22624373, DECK),
    (22624373, DECK),
    (58996430, DECK),
    (21502796, DECK),
    (12580477, HAND | OPPONENT),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(4);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_DECKDES + CATEGORY_DRAW);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, distg);
    e1.set_operation(duel, drop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_category(duel, CATEGORY_DECKDES + CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_DESTROYED);
    e2.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DELAY);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, descon);
    e2.set_target(duel, distg);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_player_can_discard_deck(tp, 3));
    }
    duel.set_target_player(tp);
    duel.set_target_param(3);
    duel.set_operation_info(0, CATEGORY_DECKDES, None, 0, tp, 3);
    Ok(false)
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_LIGHTSWORN]) && c.is_location(duel, LOCATION_GRAVE)
}

fn discard(duel: &mut Duel) -> Script<usize> {
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let d: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    duel.discard_deck(p, d as u16, REASON_EFFECT)?;
    Ok(duel.get_operated_group().filter_count(duel, cfilter, None))
}

fn drop(duel: &mut Duel, a: &Args) -> Script<()> {
    let ct = discard(duel)?;
    if ct > 0 {
        duel.break_effect()?;
        duel.draw(a.tp, ct as u32, REASON_EFFECT)?;
    }
    Ok(())
}

fn descon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_reason(duel, REASON_BATTLE) || (a.rp != a.tp && c.is_reason(duel, REASON_EFFECT) && c.is_previous_controler(duel, a.tp))
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let ct = discard(duel)?;
    let dg = duel.get_matching_group(aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, None);
    if ct != 0 && !dg.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 0))? {
        duel.break_effect()?;
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
        let sdg = dg.select(duel, tp, 1, ct as u8, None)?;
        duel.hint_selection(sdg);
        duel.destroy(sdg, REASON_EFFECT)?;
    }
    Ok(())
}
