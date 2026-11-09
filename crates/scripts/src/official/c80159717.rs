// Dragunity Knight - Trident
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 80159717;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f1(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_race_full(duel, RACE_DRAGON, Some(sc), sumtype, tp)
    }

    fn f2(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, |c, duel| {
            c.is_race_full(duel, RACE_WINGEDBEAST, Some(sc), sumtype, tp)
        })
    }
}

// Synchro Summoned with Guardragon Justicia (a Dragon Tuner) and Peacock (a Level 5 Winged Beast Normal Monster).
pub const SUPPORT: &[(u32, u16)] = &[(ID, EXTRA), (ID, EXTRA), (11012154, MZONE), (20624263, MZONE), (20624263, MZONE)];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::Resolves];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOGRAVE);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_count_limit(duel, 1, 0);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_cost(duel, excost);
    e1.set_operation(duel, exop);
    c.register_effect(duel, e1);
}

fn excost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let ct = duel.get_field_group_count(tp, 0, LOCATION_EXTRA).min(3) as u8;
    if chk == 0 {
        return Ok(ct > 0 && duel.is_existing_matching_card(CardId::is_able_to_grave_as_cost, tp, LOCATION_ONFIELD, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, CardId::is_able_to_grave_as_cost, tp, LOCATION_ONFIELD, 0, 1, ct, None)?;
    duel.send_to_grave(g, REASON_COST)?;
    e.set_label(duel, g.len() as i64);
    Ok(false)
}

fn exop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let g = duel.get_field_group(tp, 0, LOCATION_EXTRA);
    let ct = e.get_label(duel) as u8;
    if g.len() < usize::from(ct) {
        return Ok(());
    }
    duel.confirm_cards(tp, g)?;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let sg = duel.select_matching_card(tp, CardId::is_able_to_grave, tp, 0, LOCATION_EXTRA, ct, ct, None)?;
    duel.send_to_grave(sg, REASON_EFFECT)?;
    duel.shuffle_extra(1 - tp)?;
    Ok(())
}
