// Agave Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE, OPPONENT};

const ID: u32 = 2411269;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        !c.is_type_full(duel, TYPE_TOKEN, Some(lc), sumtype, tp)
    }
}

// Digitron and Bitron to Link Summon with, a Dragon (Petit Dragon), Dinosaur (Tomozaurus), Sea Serpent (Atlantean Pikeman) and Wyrm
// (Mystery Shell Dragon) in the Graveyard, and a monster of the opponent's to weaken.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (32295838, MZONE),
    (36211150, MZONE),
    (32295838, MZONE),
    (36211150, MZONE),
    (75356564, GRAVE),
    (46457856, GRAVE),
    (26976414, GRAVE),
    (18108166, GRAVE),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DAMAGE + CATEGORY_RECOVER + CATEGORY_ATKCHANGE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_link_summoned(duel));
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn graveyard_group(duel: &Duel, tp: u8) -> CardSet {
    let filter = |c: CardId, duel: &Duel| c.is_race(duel, RACE_DRAGON | RACE_DINOSAUR | RACE_SEASERPENT | RACE_WYRM);
    duel.get_matching_group(filter, tp, LOCATION_GRAVE, LOCATION_GRAVE, None)
}

fn count(duel: &Duel, g: CardSet, race: u64) -> usize {
    g.filter_count(duel, |c, duel| c.is_race(duel, race), None)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let g = graveyard_group(duel, tp);
    if chk == 0 {
        return Ok(!g.is_empty()
            && (count(duel, g, RACE_SEASERPENT) < g.len()
                || duel.is_existing_matching_card(CardId::is_faceup, tp, 0, LOCATION_MZONE, 1, None)));
    }
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, count(duel, g, RACE_DRAGON) as i32 * 100);
    duel.set_operation_info(0, CATEGORY_RECOVER, None, 0, tp, count(duel, g, RACE_WYRM) as i32 * 400);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let g = graveyard_group(duel, tp);
    if g.is_empty() {
        return Ok(());
    }
    let c = e.get_handler(duel);
    let (ct1, ct2, ct3, ct4) = (
        count(duel, g, RACE_DRAGON),
        count(duel, g, RACE_DINOSAUR),
        count(duel, g, RACE_SEASERPENT),
        count(duel, g, RACE_WYRM),
    );
    if ct1 > 0 {
        duel.damage(1 - tp, ct1 as u32 * 100, REASON_EFFECT)?;
    }
    if ct2 > 0 && c.is_faceup(duel) && c.is_relate_to_effect(duel, e) {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_value(duel, ct2 as i32 * 200);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        c.register_effect(duel, e1);
    }
    let og = duel.get_matching_group(CardId::is_faceup, tp, 0, LOCATION_MZONE, None);
    if ct3 > 0 && !og.is_empty() {
        for tc in og.iter() {
            let e2 = duel.create_effect(c);
            e2.set_type(duel, EFFECT_TYPE_SINGLE);
            e2.set_code(duel, EFFECT_UPDATE_ATTACK);
            e2.set_value(duel, ct3 as i32 * -300);
            e2.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
            tc.register_effect(duel, e2);
        }
    }
    if ct4 > 0 {
        duel.recover(tp, ct4 as u32 * 400, REASON_EFFECT)?;
    }
    Ok(())
}
