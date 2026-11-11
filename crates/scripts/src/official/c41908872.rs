// Supreme King Dragon Lightwurm
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT, PZONE};

const ID: u32 = 41908872;

pub const LISTED_NAMES: &[u32] = &[CARD_ZARC];
// In the Pendulum Zone with Z-ARC and Darkwurm on the field, in the hand to Summon, and the opponent's Dark Hole to destroy the Pendulum Monsters.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, PZONE),
    (ID, HAND),
    (ID, EXTRA),
    (CARD_ZARC, MZONE),
    (69610326, MZONE),
    (69610326, HAND),
    (14513273, MZONE),
    (53129443, HAND | OPPONENT),
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
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_LVCHANGE);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_range(duel, LOCATION_PZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_SUMMON_SUCCESS);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 2));
    e4.set_category(duel, CATEGORY_TOHAND);
    e4.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e4.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DELAY);
    e4.set_code(duel, EVENT_DESTROYED);
    e4.set_range(duel, LOCATION_EXTRA);
    e4.set_count_limit(duel, 1, (ID, 2));
    e4.set_condition(duel, ethcon);
    e4.set_target(duel, ethtg);
    e4.set_operation(duel, stock::self_to_hand);
    c.register_effect(duel, e4);
}

fn zarcfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_faceup(duel)
        && c.is_code(duel, CARD_ZARC)
        && duel.is_existing_matching_card(
            aux::faceup_filter(|c, duel| c.is_type(duel, TYPE_PENDULUM)),
            tp,
            LOCATION_MZONE,
            0,
            1,
            Some(c.into()),
        )
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_matching_card(|c, duel| zarcfilter(c, duel, tp), tp, LOCATION_ONFIELD, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, 0);
    Ok(false)
}

fn pendfilter(c: CardId, duel: &Duel, att: u32, lv: u32) -> bool {
    c.is_faceup(duel) && c.is_type(duel, TYPE_PENDULUM) && !(c.get_attribute(duel) == att && c.is_level(duel, &[lv]))
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) || duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? == 0 {
        return Ok(());
    }
    let (att, lv) = (c.get_attribute(duel), c.get_level(duel));
    let g = duel.get_matching_group(|c, duel| pendfilter(c, duel, att, lv), tp, LOCATION_MZONE, 0, Some(c.into()));
    if g.is_empty() || !duel.select_yes_no(tp, aux::stringid(ID, 3))? {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    let mut sg = g.select(duel, tp, 1, 1, None)?;
    if sg.is_empty() {
        return Ok(());
    }
    duel.hint_selection(sg);
    sg.insert(c);
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 4));
    let Some(sc) = sg.select(duel, tp, 1, 1, None)?.get_first() else {
        return Ok(());
    };
    duel.hint_selection(sc);
    duel.break_effect()?;
    let Some(tc) = sg.difference(sc.into()).get_first() else {
        return Ok(());
    };
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e1.set_code(duel, EFFECT_CHANGE_ATTRIBUTE);
    e1.set_value(duel, sc.get_attribute(duel) as i32);
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
    tc.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_CHANGE_LEVEL);
    e2.set_value(duel, sc.get_level(duel) as i32);
    tc.register_effect(duel, e2);
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_SUPREME_KING_GATE, SET_SUPREME_KING_DRAGON]) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter, tp, LOCATION_EXTRA, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_EXTRA as i32);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn exfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_SUPREME_KING_DRAGON]) && (c.is_synchro_summonable(duel, None, None) || c.is_xyz_summonable(duel, None, None))
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_EXTRA, 0, 1, 1, None)?;
    let Some(tc) = g.get_first() else { return Ok(()) };
    if duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)? == 0 || !tc.is_location(duel, LOCATION_HAND) {
        return Ok(());
    }
    duel.confirm_cards(1 - tp, g)?;
    let sg = duel.get_matching_group(exfilter, tp, LOCATION_EXTRA, 0, None);
    if sg.is_empty() || !duel.select_yes_no(tp, aux::stringid(ID, 5))? {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let Some(sc) = sg.select(duel, tp, 1, 1, None)?.get_first() else {
        return Ok(());
    };
    duel.break_effect()?;
    if sc.is_type(duel, TYPE_SYNCHRO) {
        duel.synchro_summon(tp, sc, None, None)?;
    } else if sc.is_type(duel, TYPE_XYZ) {
        duel.xyz_summon(tp, sc, None, None)?;
    }
    Ok(())
}

fn ethfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_reason(duel, REASON_BATTLE | REASON_EFFECT)
        && c.is_previous_controler(duel, tp)
        && c.is_previous_location(duel, LOCATION_MZONE)
        && c.is_previous_position(duel, POS_FACEUP)
        && c.get_previous_type_on_field(duel) & TYPE_PENDULUM == TYPE_PENDULUM
}

fn ethcon(duel: &Duel, a: &Args) -> bool {
    let (eg, tp) = (a.eg.unwrap_or_default(), a.tp);
    !eg.is_contains(a.e.get_handler(duel)) && eg.is_exists(duel, |c, duel| ethfilter(c, duel, tp), 1, None)
}

fn ethtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(c.is_able_to_hand(duel));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, Some(c.into()), 1, tp, 0);
    Ok(false)
}
