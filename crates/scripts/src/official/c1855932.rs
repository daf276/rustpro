// Bujintei Kagutsuchi
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE, OPPONENT};
use engine::constants::MSG_SELECT_EFFECTYN;

const ID: u32 = 1855932;

// In the Extra Deck, Xyz Summoned with Level 4 Beast-Warriors (Bujin Yamato, One Who Hunts Souls, Battle Ox), with Yamato in the
// Deck to mill and the opponent's Raigeki to destroy a Bujin Yamato it protects.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (32339440, MZONE),
    (3606209, MZONE),
    (5053103, MZONE),
    (32339440, DECK),
    (32339440, DECK),
    (32339440, DECK),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::Resolves, Check::Message(MSG_SELECT_EFFECTYN)];

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> = Some(|c, duel, scard, sumtype, tp| c.is_race_full(duel, RACE_BEASTWARRIOR, Some(scard), sumtype, tp));
    const LV: Option<u32> = Some(4);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.set_unique_on_field(duel, 1, 0, |c, duel| c.is_code(duel, ID), LOCATION_ONFIELD);
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_category(duel, CATEGORY_DECKDES);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_xyz_summoned(duel));
    e1.set_target(duel, distg);
    e1.set_operation(duel, disop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EFFECT_DESTROY_REPLACE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target(duel, reptg);
    e2.set_value(
        duel,
        Value::Card(|duel, e, c| i32::from(e.get_label_object_group(duel).and_then(|g| g.get_first()) == Some(c))),
    );
    c.register_effect(duel, e2);
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_DECKDES, None, 0, a.tp, 5);
    Ok(false)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.discard_deck(tp, 5, REASON_EFFECT)?;
    let c = e.get_handler(duel);
    if c.is_faceup(duel) && c.is_relate_to_effect(duel, e) {
        let ct = duel
            .get_operated_group()
            .filter_count(duel, |c, duel| c.is_set_card(duel, &[SET_BUJIN]), None);
        if ct > 0 {
            let e1 = duel.create_effect(c);
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_code(duel, EFFECT_UPDATE_ATTACK);
            e1.set_value(duel, ct as i32 * 100);
            e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD_DISABLE);
            c.register_effect(duel, e1);
        }
    }
    Ok(())
}

fn repfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_faceup(duel)
        && c.is_controler(duel, tp)
        && c.is_location(duel, LOCATION_MZONE)
        && c.is_set_card(duel, &[SET_BUJIN])
        && c.is_race(duel, RACE_BEASTWARRIOR)
}

fn reptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let eg = a.eg.unwrap_or_default();
    if chk == 0 {
        return Ok(c.is_destructable_full(duel, e) && eg.is_exists(duel, |c, duel| repfilter(c, duel, tp), 1, None));
    }
    if c.check_remove_overlay_card(duel, tp, 1, REASON_EFFECT) && duel.select_effect_yes_no_full(tp, c, 96)? {
        c.remove_overlay_card(duel, tp, 1, 1, REASON_EFFECT)?;
        let g = eg.filter(duel, |c, duel| repfilter(c, duel, tp), None);
        if g.len() == 1 {
            e.set_label_object_group(duel, g.get_first().map(Into::into));
        } else {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESREPLACE);
            let cg = g.select(duel, tp, 1, 1, None)?;
            e.set_label_object_group(duel, cg.get_first().map(Into::into));
        }
        return Ok(true);
    }
    Ok(false)
}
