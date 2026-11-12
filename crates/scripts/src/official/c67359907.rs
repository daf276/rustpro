// Disaster, Dragon Ruler of All Apocalypses
use crate::prelude::*;
use crate::testing::{EXTRA, GRAVE, MZONE, REMOVED};

const ID: u32 = 67359907;

// In the Extra Deck, Xyz Summoned with two Rank 7 Xyz Monsters (Chasma, Eclipse), with the other Level 7 Dragon Rulers in the Graveyard and banished
// to attach.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (4965193, MZONE),
    (30350202, MZONE),
    (26400609, GRAVE),
    (53804307, GRAVE),
    (89399912, REMOVED),
    (90411554, REMOVED),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> = Some(|c, duel, _, _, _| c.is_rank(duel, &[7]));
    const LV: Option<u32> = None;
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_xyz_summoned(duel));
    e1.set_target(duel, attachtg);
    e1.set_operation(duel, attachop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, attributematcon);
    e2.set_value(duel, 4600);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EFFECT_UPDATE_DEFENSE);
    c.register_effect(duel, e3);
    let e4 = e2.clone_effect(duel);
    e4.set_code(duel, EFFECT_IMMUNE_EFFECT);
    e4.set_value(duel, Value::Effect(|duel, e, te| te.get_owner(duel) != e.get_owner(duel)));
    c.register_effect(duel, e4);
}

fn attachfilter(c: CardId, duel: &Duel, xc: CardId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_DRAGON_RULER])
        && c.is_level(duel, &[7])
        && c.is_can_be_xyz_material(duel, Some(xc), tp, REASON_EFFECT)
        && c.is_faceup(duel)
}

// The Lua tests an undefined `chkc` here, which is always nil, so the target never takes the chkc path.
fn attachtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(duel.is_existing_target(
            |tc, duel| attachfilter(tc, duel, c, tp),
            tp,
            LOCATION_GRAVE | LOCATION_REMOVED,
            0,
            1,
            None,
        ));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_XMATERIAL);
    let g = duel
        .select_target(
            tp,
            |tc, duel| attachfilter(tc, duel, c, tp),
            tp,
            LOCATION_GRAVE | LOCATION_REMOVED,
            0,
            1,
            4,
            None,
        )?
        .unwrap_or_default();
    let gyg = g.filter(duel, |tc, duel| tc.is_location(duel, LOCATION_GRAVE), None);
    if !gyg.is_empty() {
        duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, Some(gyg), gyg.len() as u8, tp, 0);
    }
    duel.set_possible_operation_info(0, CATEGORY_REMOVE, None, 1, 1 - tp, (LOCATION_MZONE | LOCATION_GRAVE) as i32);
    Ok(false)
}

fn rmfilter(c: CardId, duel: &Duel, og: CardSet) -> bool {
    c.is_monster(duel)
        && c.is_able_to_remove(duel)
        && c.is_faceup(duel)
        && og.is_exists(duel, |oc, duel| oc.is_attribute(duel, c.get_attribute(duel)), 1, None)
}

fn attachop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) || c.is_immune_to_effect(duel, e) {
        return Ok(());
    }
    let mut tg = duel
        .get_target_cards(e)
        .unwrap_or_default()
        .filter(duel, |tc, duel| attachfilter(tc, duel, c, tp), None);
    tg.remove_matching(duel, |tc, duel| tc.is_immune_to_effect(duel, e), None);
    if tg.is_empty() {
        return Ok(());
    }
    duel.overlay(c, tg)?;
    let og = c.get_overlay_group(duel);
    if duel.is_existing_matching_card(|tc, duel| rmfilter(tc, duel, og), tp, 0, LOCATION_MZONE | LOCATION_GRAVE, 1, None)
        && duel.select_yes_no(tp, aux::stringid(ID, 1))?
    {
        let rg = duel.get_matching_group(|tc, duel| rmfilter(tc, duel, og), tp, 0, LOCATION_MZONE | LOCATION_GRAVE, None);
        duel.break_effect()?;
        duel.remove(rg, POS_FACEUP, REASON_EFFECT)?;
    }
    Ok(())
}

fn attributematcon(duel: &Duel, a: &Args) -> bool {
    let og =
        a.e.get_handler(duel)
            .get_overlay_group(duel)
            .filter(duel, |c, duel| c.is_set_card(duel, &[SET_DRAGON_RULER]), None);
    [
        ATTRIBUTE_LIGHT,
        ATTRIBUTE_DARK,
        ATTRIBUTE_EARTH,
        ATTRIBUTE_WATER,
        ATTRIBUTE_FIRE,
        ATTRIBUTE_WIND,
    ]
    .into_iter()
    .all(|attribute| og.is_exists(duel, |c, duel| c.is_attribute(duel, attribute), 1, None))
}
