// Poissonniere de Nouvelles
use crate::prelude::*;
use crate::testing::{Check, DECK, MZONE};

const ID: u32 = 46485778;

// In the Pendulum Zone and on the field, with Relinquished and Clown Crew Flair (Level 1 Ritual Monsters) on the field and in the Deck. No
// "Nouvelles" Ritual Monster is ported, so the Special Summon option is never available.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, LOCATION_PZONE as u16),
    (ID, 0),
    (64631466, MZONE),
    (42759961, MZONE),
    (64631466, DECK),
    (42759961, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Pendulum::add_procedure(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_POSITION);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_PZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, spcon);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_RELEASE);
    e2.set_condition(duel, pencon);
    e2.set_target(duel, pentg);
    e2.set_operation(duel, penop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_SUMMON_SUCCESS);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_target(duel, efftg);
    e3.set_operation(duel, effop);
    c.register_effect(duel, e3);
    let e4 = e3.clone_effect(duel);
    e4.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e4);
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(aux::faceup_filter(CardId::is_ritual_monster), a.tp, LOCATION_MZONE, 0, 1, None)
}

// The Lua's chkc here is an undefined global, so a target check is never made.
fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
            && duel.is_existing_target(CardId::is_can_change_position, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_POSCHANGE);
    let g = duel.select_target(tp, CardId::is_can_change_position, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_POSITION, g, 1, 0, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let tc = duel.get_first_target();
    if c.is_relate_to_effect(duel, e)
        && duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? > 0
        && let Some(tc) = tc.filter(|tc| tc.is_relate_to_effect(duel, e))
    {
        duel.break_effect()?;
        duel.change_position_full(
            tc,
            [POS_FACEUP_DEFENSE, POS_FACEUP_ATTACK, POS_FACEUP_ATTACK, POS_FACEUP_ATTACK],
            false,
        )?;
    }
    Ok(())
}

fn pencon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_faceup(duel) && c.is_location(duel, LOCATION_EXTRA) && c.is_previous_location(duel, LOCATION_ONFIELD)
}

fn pentg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    Ok(chk == 0 && duel.check_pendulum_zones(a.tp))
}

fn penop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if !duel.check_pendulum_zones(tp) {
        return Ok(());
    }
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) {
        duel.move_to_field(c, tp, tp, LOCATION_PZONE, POS_FACEUP, true)?;
    }
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_set_card(duel, &[SET_RECIPE]) || (c.is_ritual_monster(duel) && c.is_level(duel, &[1]))) && c.is_able_to_hand(duel)
}

fn rmvfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_RECIPE]) && c.is_able_to_remove(duel)
}

fn nvlfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, lv: usize) -> bool {
    c.is_set_card(duel, &[SET_NOUVELLES])
        && c.is_ritual_monster(duel)
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, true)
        && c.is_level_below(duel, lv as u32)
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let rg = duel.get_matching_group(rmvfilter, tp, LOCATION_GRAVE, 0, None);
    let b1 = duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None);
    let b2 = duel.get_location_count(tp, LOCATION_MZONE) > 0
        && !rg.is_empty()
        && duel.is_existing_matching_card(|c, duel| nvlfilter(c, duel, e, tp, rg.len()), tp, LOCATION_HAND, 0, 1, None);
    if chk == 0 {
        return Ok(b1 || b2);
    }
    let op = duel.select_effect(tp, &[(b1, aux::stringid(ID, 3)), (b2, aux::stringid(ID, 4))])?;
    e.set_label(duel, op.map_or(0, |op| op as i64));
    match op {
        Some(1) => {
            e.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
            duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
        }
        Some(2) => {
            e.set_category(duel, CATEGORY_REMOVE + CATEGORY_SPECIAL_SUMMON);
            duel.set_operation_info(0, CATEGORY_REMOVE, None, 1, tp, LOCATION_GRAVE as i32);
            duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_HAND as i32);
        }
        _ => {}
    }
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, lv: usize) -> bool {
    c.is_set_card(duel, &[SET_NOUVELLES])
        && c.is_ritual_monster(duel)
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, true)
        && c.is_level(duel, &[lv as u32])
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    match e.get_label(duel) {
        1 => {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
            let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
            if !g.is_empty() {
                duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
                duel.confirm_cards(1 - tp, g)?;
            }
        }
        2 => {
            let rg = duel.get_matching_group(rmvfilter, tp, LOCATION_GRAVE, 0, None);
            if rg.is_empty() {
                return Ok(());
            }
            let rescon = |duel: &Duel, sg: CardSet, _: CardSet, _: Option<CardId>| {
                (
                    duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp, sg.len()), tp, LOCATION_HAND, 0, 1, None),
                    false,
                )
            };
            let rmg = aux::select_unselect_group_full(duel, rg, 1, rg.len(), &rescon, tp, HINTMSG_REMOVE, Some(&rescon), false)?;
            if !rmg.is_empty() && duel.remove(rmg, 0, REASON_EFFECT)? > 0 && duel.get_location_count(tp, LOCATION_MZONE) > 0 {
                duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
                let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp, rmg.len()), tp, LOCATION_HAND, 0, 1, 1, None)?;
                if !g.is_empty() {
                    duel.special_summon(g, 0, tp, tp, false, true, POS_FACEUP)?;
                }
            }
        }
        _ => {}
    }
    Ok(())
}
