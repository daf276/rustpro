// Dark Highlander
use crate::prelude::*;
use crate::testing::{EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 75326861;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f1(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_race_full(duel, RACE_FIEND, Some(sc), sumtype, tp)
    }

    fn f2(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_not_tuner(duel, sc, tp) && c.is_race_full(duel, RACE_FIEND, Some(sc), sumtype, tp)
    }
}

// In the Extra Deck, Synchro Summoned with Hallohallo (a Level 3 Tuner) and Archfiend Soldier (Level 4), with the opponent's Demotion
// in hand to equip to a monster.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (77994337, MZONE),
    (77994337, MZONE),
    (49881766, MZONE),
    (49881766, MZONE),
    (72575145, HAND | OPPONENT),
    (72575145, HAND | OPPONENT),
    (72575145, HAND | OPPONENT),
    (4035199, MZONE | OPPONENT),
    (4035199, MZONE | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_target_range(duel, 1, 1);
    e1.set_target_spsummon(duel, |_, _, _, _, sumtp, _, _, _| {
        sumtp & SUMMON_TYPE_SYNCHRO == SUMMON_TYPE_SYNCHRO
    });
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_DESTROY + CATEGORY_DAMAGE);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_count_limit(duel, 1, 0);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target(duel, target);
    e2.set_operation(duel, operation);
    c.register_effect(duel, e2);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.get_equip_count(duel) > 0
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, 1 - tp) && chkc.is_location(duel, LOCATION_MZONE) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_OPPO);
    let g = duel.select_target(tp, filter, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    let eqg = g.and_then(|g| g.get_first()).map(|tc| tc.get_equip_group(duel)).unwrap_or_default();
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(eqg), eqg.len() as u8, 0, 0);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, eqg.len() as i32 * 400);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
    {
        let eqg = tc.get_equip_group(duel);
        if !eqg.is_empty() {
            let des = duel.destroy(eqg, REASON_EFFECT)?;
            duel.damage(1 - tp, des as u32 * 400, REASON_EFFECT)?;
        }
    }
    Ok(())
}
