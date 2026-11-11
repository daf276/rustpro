// Marshmao☆Yummy
use crate::prelude::*;
use crate::testing::{Check, HAND};

const ID: u32 = 10966439;

// No Yummy Spell/Trap is ported yet, so the second effect never has a target.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (ID, HAND)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 1))),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, spcon);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e1);
    let e2a = duel.create_effect(c);
    e2a.set_description(duel, aux::stringid(ID, 1));
    e2a.set_category(duel, CATEGORY_TOHAND);
    e2a.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2a.set_property(duel, EFFECT_FLAG_DELAY);
    e2a.set_code(duel, EVENT_SUMMON_SUCCESS);
    e2a.set_count_limit(duel, 1, (ID, 1));
    e2a.set_target(duel, thpltg);
    e2a.set_operation(duel, thplop);
    c.register_effect(duel, e2a);
    let e2b = e2a.clone_effect(duel);
    e2b.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e2b);
}

fn spconfilter(c: CardId, duel: &Duel) -> bool {
    !(c.is_attribute(duel, ATTRIBUTE_LIGHT) && c.is_race(duel, RACE_BEAST) && c.is_faceup(duel))
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    !duel.is_existing_matching_card(spconfilter, a.tp, LOCATION_MZONE, 0, 1, None)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (tp, c) = (a.tp, a.e.get_handler(duel));
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && c.is_can_be_special_summoned(duel, a.e, 0, tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    Ok(false)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_YUMMY]) && c.is_spell_trap(duel) && c.is_able_to_hand(duel)
}

fn plfilter(c: CardId, duel: &Duel, tp: u8, szone_chk: bool) -> bool {
    c.is_set_card(duel, &[SET_YUMMY])
        && (c.is_field_spell(duel) || (szone_chk && c.is_continuous_spell_trap(duel)))
        && (c.is_faceup(duel) || c.is_location(duel, LOCATION_DECK))
        && !c.is_forbidden(duel)
        && c.check_unique_on_field(duel, tp)
}

fn thpltg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (tp, e) = (a.tp, a.e);
    let sp_chk = a.re.is_some_and(|re| {
        e.get_handler(duel).is_special_summoned(duel)
            && re.is_monster_effect(duel)
            && re.get_handler(duel).is_original_type(duel, TYPE_SYNCHRO)
    });
    if chk == 0 {
        let szone_chk = duel.get_location_count(tp, LOCATION_SZONE) > 0;
        return Ok(duel.is_existing_matching_card(thfilter, tp, LOCATION_GRAVE, 0, 1, None)
            || (sp_chk
                && duel.is_existing_matching_card(
                    |c, duel| plfilter(c, duel, tp, szone_chk),
                    tp,
                    LOCATION_DECK | LOCATION_REMOVED,
                    0,
                    1,
                    None,
                )));
    }
    e.set_label(duel, i64::from(sp_chk));
    if sp_chk {
        duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_GRAVE as i32);
    } else {
        duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_GRAVE as i32);
    }
    Ok(false)
}

fn thplop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let sp_chk = a.e.get_label(duel) == 1;
    let szone_chk = sp_chk && duel.get_location_count(tp, LOCATION_SZONE) > 0;
    let mut g = duel.get_matching_group(aux::necro_valley_filter(thfilter), tp, LOCATION_GRAVE, 0, None);
    if sp_chk {
        g.merge(duel.get_matching_group(
            |c, duel| plfilter(c, duel, tp, szone_chk),
            tp,
            LOCATION_DECK | LOCATION_REMOVED,
            0,
            None,
        ));
    }
    if g.is_empty() {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, if sp_chk { aux::stringid(ID, 2) } else { HINTMSG_ATOHAND });
    let Some(sc) = g.select(duel, tp, 1, 1, None)?.get_first() else {
        return Ok(());
    };
    if sc.is_location(duel, LOCATION_GRAVE) {
        duel.hint_selection(sc);
        duel.send_to_hand(sc, PLAYER_NONE, REASON_EFFECT)?;
    } else {
        if sc.is_location(duel, LOCATION_REMOVED) {
            duel.hint_selection(sc);
        }
        let place_location = if sc.is_field_spell(duel) { LOCATION_FZONE } else { LOCATION_SZONE };
        duel.move_to_field(sc, tp, tp, place_location, POS_FACEUP, true)?;
    }
    Ok(())
}
