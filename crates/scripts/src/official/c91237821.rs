// Clown Crew Fiends
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE};

const ID: u32 = 91237821;

pub const LISTED_SERIES: &[u16] = &[SET_CLOWN_CREW];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(3);
    const CT: usize = 2;
}

// In the Extra Deck and on the field, Xyz Summoned with Dark Gray and Burglar (Level 3), with Clown Crew Biancaviso in the hand to
// Tribute it and a Clown Crew Spell in the Deck to Set.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (9159938, MZONE),
    (6297941, MZONE),
    (82159583, HAND),
    (83232904, DECK),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, LOCATION_MZONE, 0);
    e1.set_target_card(duel, |duel, _, c| c.is_tribute_summoned(duel));
    e1.set_value(duel, 1000);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_RELEASE);
    e2.set_target(duel, efftg);
    e2.set_operation(duel, effop);
    c.register_effect(duel, e2);
}

fn setfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_CLOWN_CREW]) && c.is_spell(duel) && c.is_ssetable(duel)
}

fn xyzfilter(c: CardId, duel: &Duel) -> bool {
    c.is_xyz_monster(duel) && c.is_faceup(duel) && c.is_able_to_extra(duel)
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let b1 = !duel.has_flag_effect(tp, ID)
        && duel.is_existing_matching_card(
            xyzfilter,
            tp,
            LOCATION_MZONE | LOCATION_GRAVE,
            LOCATION_MZONE | LOCATION_GRAVE,
            1,
            None,
        );
    let b2 = !duel.has_flag_effect(tp, ID + 1) && duel.is_existing_matching_card(setfilter, tp, LOCATION_DECK, 0, 1, None);
    if chk == 0 {
        return Ok(b1 || b2);
    }
    let op = duel.select_effect(tp, &[(b1, aux::stringid(ID, 1)), (b2, aux::stringid(ID, 2))])?;
    e.set_label(duel, op.map_or(0, |op| op as i64));
    if op == Some(1) {
        duel.register_flag_effect(tp, ID, RESET_PHASE | PHASE_END, 0, 1);
        e.set_category(duel, CATEGORY_TOEXTRA);
        duel.set_operation_info(0, CATEGORY_TOEXTRA, None, 1, PLAYER_ALL, (LOCATION_MZONE | LOCATION_GRAVE) as i32);
    } else if op == Some(2) {
        duel.register_flag_effect(tp, ID + 1, RESET_PHASE | PHASE_END, 0, 1);
        e.set_category(duel, CATEGORY_SET);
    }
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    match a.e.get_label(duel) {
        1 => {
            let g = duel.get_matching_group(
                xyzfilter,
                tp,
                LOCATION_MZONE | LOCATION_GRAVE,
                LOCATION_MZONE | LOCATION_GRAVE,
                None,
            );
            if !g.is_empty() {
                duel.send_to_deck(g, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)?;
            }
        }
        2 => {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_SET);
            let g = duel.select_matching_card(tp, setfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
            if !g.is_empty() {
                duel.sset(tp, g)?;
            }
        }
        _ => {}
    }
    Ok(())
}
