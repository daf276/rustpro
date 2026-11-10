// Majesty of the White Dragons
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 43219114;

pub const LISTED_NAMES: &[u32] = &[CARD_BLUEEYES_W_DRAGON];

struct Params;

impl RitualParams for Params {
    const DESC: u64 = aux::stringid(ID, 1);

    fn matfilter(c: CardId, duel: &Duel, _: EffectId, _: u8) -> bool {
        c.is_code(duel, CARD_BLUEEYES_W_DRAGON)
    }
}

// In the hand and the Graveyard, with Blue-Eyes White Dragon in the hand, on the field and in the GY to show or Tribute for the Level 8 Magician of Black Chaos
// in the hand, and a monster of the opponent's to destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (CARD_BLUEEYES_W_DRAGON, HAND),
    (CARD_BLUEEYES_W_DRAGON, MZONE),
    (CARD_BLUEEYES_W_DRAGON, GRAVE),
    (30208479, HAND),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit(duel, 1, ID);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = Ritual::create_proc::<Params, RITPROC_EQUAL>(duel, c);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e2.set_cost(duel, Cost::self_banish);
    c.register_effect(duel, e2);
}

fn showfilter(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, CARD_BLUEEYES_W_DRAGON) && (c.is_faceup(duel) || !c.is_location(duel, LOCATION_MZONE))
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(
            duel.is_existing_matching_card(showfilter, tp, LOCATION_HAND | LOCATION_MZONE | LOCATION_GRAVE, 0, 1, None)
                && duel.get_field_group_count(tp, 0, LOCATION_ONFIELD) > 0,
        );
    }
    let g = duel.get_matching_group(aux::TRUE, tp, 0, LOCATION_ONFIELD, None);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), 1, tp, 0);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if duel.get_field_group_count(tp, 0, LOCATION_ONFIELD) == 0 {
        return Ok(());
    }
    let showg = duel.get_matching_group(showfilter, tp, LOCATION_HAND | LOCATION_MZONE | LOCATION_GRAVE, 0, None);
    if showg.is_empty() {
        return Ok(());
    }
    let rescon =
        |duel: &Duel, sg: CardSet, _: CardSet, _: Option<CardId>| (duel.get_field_group_count(tp, 0, LOCATION_ONFIELD) >= sg.len(), false);
    let g = aux::select_unselect_group(duel, showg, 1, 3, &rescon, tp, HINTMSG_CONFIRM)?;
    let ct = g.len();
    if ct == 0 {
        return Ok(());
    }
    let (confirmg, hintg) = g.split(duel, |c, duel| c.is_location(duel, LOCATION_HAND), None);
    if !confirmg.is_empty() {
        duel.confirm_cards(1 - tp, confirmg)?;
        duel.shuffle_hand(tp)?;
    }
    if !hintg.is_empty() {
        duel.hint_selection(hintg);
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let desg = duel.select_matching_card(tp, aux::TRUE, tp, 0, LOCATION_ONFIELD, ct as u8, ct as u8, None)?;
    if !desg.is_empty() {
        duel.hint_selection(desg);
        duel.break_effect()?;
        duel.destroy(desg, REASON_EFFECT)?;
    }
    Ok(())
}
