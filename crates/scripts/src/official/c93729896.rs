// Nightmare Throne
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 93729896;

// In the hand, with Yubel (Level 10, 0/0 Fiend) on the field for the opponent's Raigeki to destroy, and Terror Incarnate
// (Level 11) and Yubel in the Deck to search.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (78371393, MZONE),
    (78371393, MZONE),
    (78371393, DECK),
    (4779091, DECK),
    (4779091, DECK),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2a = duel.create_effect(c);
    e2a.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2a.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e2a.set_code(duel, EVENT_LEAVE_FIELD);
    e2a.set_range(duel, LOCATION_FZONE);
    e2a.set_condition(duel, regcon);
    e2a.set_operation(duel, regop);
    c.register_effect(duel, e2a);
    let e2b = duel.create_effect(c);
    e2b.set_description(duel, aux::stringid(ID, 1));
    e2b.set_category(duel, CATEGORY_TOHAND + CATEGORY_SPECIAL_SUMMON);
    e2b.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2b.set_property(duel, EFFECT_FLAG_DELAY);
    e2b.set_code(duel, EVENT_CUSTOM + ID);
    e2b.set_range(duel, LOCATION_FZONE);
    e2b.set_count_limit(duel, 1, 0);
    e2b.set_condition(duel, |_, a| a.ep == a.tp);
    e2b.set_target(duel, thtg);
    e2b.set_operation(duel, thop);
    e2b.set_label_object(duel, Some(e2a));
    c.register_effect(duel, e2b);
}

fn thdesfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_FIEND) && c.is_attack(duel, &[0]) && c.is_defense(duel, &[0])
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_matching_group(thdesfilter, tp, LOCATION_DECK, 0, None);
    if !g.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 0))? {
        duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 2));
        let Some(sc) = g.select(duel, tp, 1, 1, None)?.get_first() else {
            return Ok(());
        };
        aux::to_hand_or_else(
            duel,
            sc,
            tp,
            CardId::is_able_to_grave,
            |duel| duel.destroy(sc, REASON_EFFECT).map(drop),
            aux::stringid(ID, 3),
        )?;
    }
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel, levels: &[i64]) -> bool {
    c.is_set_card(duel, &[SET_YUBEL])
        && (c.is_faceup(duel) || !c.is_location(duel, LOCATION_REMOVED))
        && c.is_able_to_hand(duel)
        && c.has_level(duel)
        && levels.iter().any(|&lv| {
            [lv - 1, lv + 1]
                .into_iter()
                .any(|lv| u32::try_from(lv).is_ok_and(|lv| c.is_original_level(duel, lv)))
        })
}

// {e:GetLabel()}: an effect without labels returns its default 0.
fn labels(duel: &Duel, e: EffectId) -> Vec<i64> {
    match e.get_labels(duel) {
        [] => vec![0],
        labels => labels.to_vec(),
    }
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let levels = e.get_label_object(duel).map_or_else(|| vec![0], |e2a| labels(duel, e2a));
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(
            |c, duel| thfilter(c, duel, &levels),
            tp,
            LOCATION_REMOVED | LOCATION_DECK | LOCATION_GRAVE,
            0,
            1,
            None,
        ));
    }
    e.set_labels(duel, &levels);
    duel.set_operation_info(
        0,
        CATEGORY_TOHAND,
        None,
        1,
        tp,
        (LOCATION_REMOVED | LOCATION_DECK | LOCATION_GRAVE) as i32,
    );
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_HAND as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let levels = labels(duel, e);
    let g = duel.select_matching_card(
        tp,
        aux::necro_valley_filter(|c, duel| thfilter(c, duel, &levels)),
        tp,
        LOCATION_REMOVED | LOCATION_DECK | LOCATION_GRAVE,
        0,
        1,
        1,
        None,
    )?;
    let Some(sc) = g.get_first() else { return Ok(()) };
    if sc.is_location(duel, LOCATION_GRAVE | LOCATION_REMOVED) {
        duel.hint_selection_full(sc, true);
    }
    if !(duel.send_to_hand(sc, PLAYER_NONE, REASON_EFFECT)? > 0 && sc.is_location(duel, LOCATION_HAND)) {
        return Ok(());
    }
    duel.confirm_cards(1 - tp, sc)?;
    if duel.get_location_count(tp, LOCATION_MZONE) > 0
        && sc.is_can_be_special_summoned(duel, e, 0, tp, true, false)
        && duel.select_yes_no(tp, aux::stringid(ID, 4))?
    {
        duel.break_effect()?;
        duel.special_summon(sc, 0, tp, tp, true, false, POS_FACEUP)?;
    }
    Ok(())
}

fn thconfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_previous_set_card(duel, &[SET_YUBEL])
        && c.is_previous_controler(duel, tp)
        && c.is_previous_location(duel, LOCATION_MZONE)
        && c.is_previous_position(duel, POS_FACEUP)
        && c.is_reason(duel, REASON_EFFECT)
        && c.has_level(duel)
}

fn regcon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    a.eg.is_some_and(|eg| eg.is_exists(duel, |c, duel| thconfilter(c, duel, tp), 1, None))
}

// The effect's label object is the group of the cards that left, and its EffectId slot, free otherwise, keeps `re` for the
// delayed event. The delayed effects take (r, rp, tp, ev) as labels and this effect as label object.
fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let g = a.eg.unwrap_or_default().filter(duel, |c, duel| thconfilter(c, duel, tp), None);
    if !duel.is_chain_solving() {
        return duel.raise_event(g, EVENT_CUSTOM + ID, a.re, a.r, a.rp, tp, a.ev);
    }
    if let Some(mut label_obj) = e.get_label_object_group(duel) {
        label_obj.merge(g);
        e.set_label_object_group(duel, Some(label_obj));
        let mut levels = labels(duel, e);
        levels.extend(g.into_iter().map(|tc| i64::from(tc.get_original_level(duel))));
        e.set_labels(duel, &levels);
        return Ok(());
    }
    e.set_labels(duel, &g.get_class(duel, |c, duel| i64::from(c.get_original_level(duel))));
    e.set_label_object_group(duel, Some(g));
    e.set_label_object(duel, a.re);
    let c = e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e1.set_code(duel, EVENT_CHAIN_SOLVED);
    e1.set_range(duel, LOCATION_FZONE);
    e1.set_condition(duel, |duel, _| duel.get_current_chain() == 1);
    e1.set_operation(duel, raiseop);
    e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
    e1.set_label_object(duel, Some(e));
    e1.set_labels(duel, &[i64::from(a.r), i64::from(a.rp), i64::from(tp), i64::from(a.ev)]);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EVENT_CHAIN_NEGATED);
    c.register_effect(duel, e2);
    Ok(())
}

fn raiseop(duel: &mut Duel, a: &Args) -> Script<()> {
    let eff = a.e;
    let Some(e) = eff.get_label_object(duel) else { return Ok(()) };
    let &[r, rp, tp, ev] = eff.get_labels(duel) else { return Ok(()) };
    let (g, re) = (e.get_label_object_group(duel), e.get_label_object(duel));
    eff.reset(duel);
    e.set_label_object_group(duel, None);
    e.set_label_object(duel, None);
    if let Some(g) = g {
        duel.raise_event(g, EVENT_CUSTOM + ID, re, r as u32, rp as u8, tp as u8, ev as u32)?;
    }
    Ok(())
}
