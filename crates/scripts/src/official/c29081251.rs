// Hubolt the Dark Directory of Death
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND};

const ID: u32 = 29081251;

pub const LISTED_NAMES: &[u32] = &[ID];

// Level 6 Zombies (Ryu Kokki) in the Deck and Graveyard, Monster Reborn to Special Summon one from the Graveyard, and Pilgrim Reaper
// (Rank 6 Zombie Xyz) in the Extra Deck to Xyz Summon with the two of them.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (57281778, DECK),
    (57281778, DECK),
    (57281778, GRAVE),
    (57281778, GRAVE),
    (83764718, HAND),
    (45742626, EXTRA),
    (45742626, EXTRA),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EFFECT_SUMMON_PROC);
    e1.set_condition(duel, |duel, a| {
        let Some(c) = a.c else { return true };
        a.min == 0 && c.is_level_above(duel, 4) && duel.get_location_count(c.get_controler(duel), LOCATION_MZONE) > 0
    });
    c.register_effect(duel, e1);
    let e2a = duel.create_effect(c);
    e2a.set_description(duel, aux::stringid(ID, 1));
    e2a.set_category(duel, CATEGORY_TOGRAVE + CATEGORY_TOHAND);
    e2a.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2a.set_property(duel, EFFECT_FLAG_DELAY);
    e2a.set_code(duel, EVENT_SUMMON_SUCCESS);
    e2a.set_count_limit(duel, 1, (ID, 0));
    e2a.set_target(duel, tgtg);
    e2a.set_operation(duel, tgop);
    c.register_effect(duel, e2a);
    let e2b = e2a.clone_effect(duel);
    e2b.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e2b);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_property_full(duel, EFFECT_FLAG_DELAY, EFFECT_FLAG2_CHECK_SIMULTANEOUS);
    e3.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_condition(duel, |duel, a| {
        a.eg.is_some_and(|eg| {
            !eg.is_contains(a.e.get_handler(duel)) && eg.is_exists(duel, |c, duel| c.is_summon_location(duel, LOCATION_GRAVE), 1, None)
        })
    });
    e3.set_target(duel, xyztg);
    e3.set_operation(duel, xyzop);
    c.register_effect(duel, e3);
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_level_above(duel, 6) && c.is_race(duel, RACE_ZOMBIE) && !c.is_code(duel, ID) && c.is_able_to_grave(duel)
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(tgfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, LOCATION_DECK as i32);
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_GRAVE as i32);
    Ok(false)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_level_above(duel, 6) && c.is_race(duel, RACE_ZOMBIE) && c.is_able_to_hand(duel)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let sc = duel
        .select_matching_card(tp, tgfilter, tp, LOCATION_DECK, 0, 1, 1, None)?
        .get_first();
    if let Some(sc) = sc
        && duel.send_to_grave(sc, REASON_EFFECT)? > 0
        && sc.is_location(duel, LOCATION_GRAVE)
    {
        duel.shuffle_deck(tp)?;
        let filter = aux::necro_valley_filter(thfilter);
        if duel.is_existing_matching_card(&filter, tp, LOCATION_GRAVE, 0, 1, None) && duel.select_yes_no(tp, aux::stringid(ID, 3))? {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
            let g = duel.select_matching_card(tp, &filter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
            if !g.is_empty() {
                duel.hint_selection(g);
                duel.break_effect()?;
                duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
            }
        }
    }
    Ok(())
}

fn xyzfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_ZOMBIE) && c.is_xyz_summonable(duel, None, None)
}

fn xyztg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, xyzfilter, LOCATION_EXTRA, CATEGORY_SPECIAL_SUMMON)
}

fn xyzop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    if let Some(sc) = duel
        .select_matching_card(tp, xyzfilter, tp, LOCATION_EXTRA, 0, 1, 1, None)?
        .get_first()
    {
        duel.xyz_summon(tp, sc, None, None)?;
    }
    Ok(())
}
