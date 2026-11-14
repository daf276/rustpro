// Jurrac Meteor
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 17548456;

pub const LISTED_SERIES: &[u16] = &[SET_JURRAC];
// Synchro Summoned from Jurrac Megalo, a Level 1 Tuner, with Frostosaurus (Level 6) and Little D (Level 3), Dinosaur Normal
// Monsters, and Megalo in the Graveyard afterwards to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (93170499, MZONE),
    (93170499, MZONE),
    (6631034, MZONE),
    (42625254, MZONE),
    (42625254, MZONE),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 2;
    const MAX2: usize = 99;

    fn f1(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_set_card_full(duel, &[SET_JURRAC], Some(sc), sumtype, tp)
    }

    fn f2(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, |c, duel| {
            c.is_race_full(duel, RACE_DINOSAUR, Some(sc), sumtype, tp)
        })
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY + CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_synchro_summoned(duel));
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    c.register_effect(duel, e1);
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(true);
    }
    let g = duel.get_matching_group(aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, None);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), g.len() as u8, 0, 0);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_GRAVE as i32);
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_type(duel, TYPE_TUNER) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let g = duel.get_matching_group(aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, None);
    if g.is_empty() || duel.destroy(g, REASON_EFFECT)? == 0 {
        return Ok(());
    }
    let filter = aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp));
    let sg = duel.get_matching_group(filter, tp, LOCATION_GRAVE, 0, None);
    if !sg.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 1))? {
        duel.break_effect()?;
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        if let Some(sp) = sg.select(duel, tp, 1, 1, None)?.get_first() {
            duel.special_summon(sp, 0, tp, tp, false, false, POS_FACEUP)?;
        }
    }
    Ok(())
}
