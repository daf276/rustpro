// Miasma Necromancer
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE};

const ID: u32 = 69176851;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn specialchk(duel: &Duel, sg: CardSet, lc: CardId, sumtype: u64, tp: u8) -> bool {
        sg.is_exists(duel, |c, duel| matfilter(c, duel, lc, sumtype, tp), 1, None)
    }
}

// In the Extra Deck, Link Summoned with Boo Koo and Dark Magician (a DARK Spellcaster), with Dark Magician in the Graveyard to
// add, Boo Koo in the hand to Normal Summon and Sogen in the hand to send.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (68963107, MZONE),
    (68963107, MZONE),
    (68963107, MZONE),
    (36996508, GRAVE),
    (36996508, GRAVE),
    (68963107, HAND),
    (68963107, HAND),
    (86318356, HAND),
    (86318356, HAND),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_link_summoned(duel));
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2a = duel.create_effect(c);
    e2a.set_description(duel, aux::stringid(ID, 1));
    e2a.set_category(duel, CATEGORY_SUMMON);
    e2a.set_type(duel, EFFECT_TYPE_IGNITION);
    e2a.set_range(duel, LOCATION_MZONE);
    e2a.set_count_limit(duel, 1, (ID, 1));
    e2a.set_cost(duel, sumcost);
    e2a.set_target(duel, sumtg);
    e2a.set_operation(duel, sumop);
    c.register_effect(duel, e2a);
    let e2b = duel.create_effect(c);
    e2b.set_type(duel, EFFECT_TYPE_FIELD);
    e2b.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    e2b.set_code(duel, EFFECT_UNRELEASABLE_SUM);
    e2b.set_range(duel, LOCATION_MZONE);
    e2b.set_target_range(duel, LOCATION_MZONE, 0);
    e2b.set_target_card(duel, |duel, e, c| {
        e.get_label_object_group(duel).and_then(|g| g.get_first()) == Some(c)
    });
    e2b.set_value(duel, 1);
    c.register_effect(duel, e2b);
    e2a.set_label_object(duel, Some(e2b));
}

fn matfilter(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
    c.is_attribute_full(duel, ATTRIBUTE_DARK, Some(lc), sumtype, tp) && c.is_race_full(duel, RACE_SPELLCASTER, Some(lc), sumtype, tp)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_level_above(duel, 5) && c.is_attribute(duel, ATTRIBUTE_DARK) && c.is_race(duel, RACE_SPELLCASTER) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_GRAVE, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let Some(sc) = duel
        .select_matching_card(tp, thfilter, tp, LOCATION_GRAVE, 0, 1, 1, None)?
        .get_first()
    else {
        return Ok(());
    };
    duel.hint_selection(sc);
    if duel.send_to_hand(sc, PLAYER_NONE, REASON_EFFECT)? > 0 && sc.is_location(duel, LOCATION_HAND) {
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_description(duel, aux::stringid(ID, 2));
        e1.set_type(duel, EFFECT_TYPE_FIELD);
        e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
        e1.set_code(duel, EFFECT_CANNOT_ACTIVATE);
        e1.set_target_range(duel, 1, 0);
        e1.set_label(duel, i64::from(sc.get_code(duel)));
        e1.set_value(
            duel,
            Value::EffectPlayer(|duel, e, re, _| re.get_handler(duel).is_code(duel, e.get_label(duel) as u32)),
        );
        e1.set_reset(duel, RESET_PHASE | PHASE_END);
        duel.register_effect(e1, tp);
    }
    Ok(())
}

fn sumfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_SPELLCASTER) && c.is_summonable(duel, true, None)
}

fn sumcostfilter(c: CardId, duel: &Duel, tp: u8, eff: EffectId) -> bool {
    c.is_spell_trap(duel)
        && c.is_able_to_grave_as_cost(duel)
        && duel.check_mut(|duel| {
            eff.set_label_object_group(duel, Some(c.into()));
            duel.is_existing_matching_card(sumfilter, tp, LOCATION_HAND | LOCATION_MZONE, 0, 1, Some(c.into()))
        })
}

fn sumcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    let Some(eff) = a.e.get_label_object(duel) else { return Ok(false) };
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(
            |c, duel| sumcostfilter(c, duel, tp, eff),
            tp,
            LOCATION_HAND | LOCATION_ONFIELD,
            0,
            1,
            None,
        ));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(
        tp,
        |c, duel| sumcostfilter(c, duel, tp, eff),
        tp,
        LOCATION_HAND | LOCATION_ONFIELD,
        0,
        1,
        1,
        None,
    )?;
    duel.send_to_grave(g, REASON_COST)?;
    Ok(false)
}

fn sumtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, sumfilter, LOCATION_HAND | LOCATION_MZONE, CATEGORY_SUMMON)
}

fn sumop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SUMMON);
    if let Some(sc) = duel
        .select_matching_card(tp, sumfilter, tp, LOCATION_HAND | LOCATION_MZONE, 0, 1, 1, None)?
        .get_first()
    {
        duel.summon(tp, sc, true, None)?;
    }
    Ok(())
}
