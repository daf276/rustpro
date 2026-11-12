// Shiranui Squiresaga
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE, OPPONENT, SZONE};

const ID: u32 = 52711246;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

// In the Extra Deck, Synchro Summoned with Assault Lich (a Level 4 Tuner) and Armored Zombie (Level 3), with a Zombie in the GY to banish and
// Spells/Traps and monsters on the field to destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (86098176, MZONE),
    (86098176, MZONE),
    (20277860, MZONE),
    (20277860, MZONE),
    (41562624, GRAVE),
    (6021033, GRAVE),
    (5318639, SZONE | OPPONENT),
    (5318639, SZONE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    c.set_sp_summon_once(duel, ID);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_able_to_remove(duel) && c.is_monster(duel) && (c.is_location(duel, LOCATION_GRAVE) || c.is_faceup(duel))
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(filter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_REMOVE, None, 1, tp, (LOCATION_MZONE | LOCATION_GRAVE) as i32);
    duel.set_possible_operation_info(0, CATEGORY_ATKCHANGE, None, 1, tp, 300);
    duel.set_possible_operation_info(0, CATEGORY_DESTROY, None, 1, tp, LOCATION_ONFIELD as i32);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, 1, 1, None)?;
    let Some(tc) = g.get_first() else { return Ok(()) };
    if duel.remove(tc, POS_FACEUP, REASON_EFFECT)? == 0 {
        return Ok(());
    }
    let g1 = duel.get_matching_group(CardId::is_faceup, tp, LOCATION_MZONE, 0, None);
    let g2 = duel.get_matching_group(aux::TRUE, tp, LOCATION_SZONE, LOCATION_SZONE, None);
    let g3 = duel.get_matching_group(aux::TRUE, tp, LOCATION_MZONE, LOCATION_MZONE, None);
    if tc.is_race(duel, RACE_ZOMBIE) && !g1.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 1))? {
        for t1 in g1 {
            let e1 = duel.create_effect(e.get_handler(duel));
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_code(duel, EFFECT_UPDATE_ATTACK);
            e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
            e1.set_value(duel, 300);
            t1.register_effect(duel, e1);
        }
    }
    if tc.is_attribute(duel, ATTRIBUTE_FIRE) && !g2.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 2))? {
        destroy_one(duel, tp, g2)?;
    }
    if tc.is_type(duel, TYPE_SYNCHRO) && !g3.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 3))? {
        destroy_one(duel, tp, g3)?;
    }
    Ok(())
}

fn destroy_one(duel: &mut Duel, tp: u8, g: CardSet) -> Script<()> {
    duel.break_effect()?;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let t = g.select(duel, tp, 1, 1, None)?;
    duel.hint_selection(t);
    duel.destroy(t, REASON_EFFECT)?;
    Ok(())
}
