// Copy Plant
use crate::prelude::*;
use crate::testing::{Check, MZONE};

const ID: u32 = 66457407;

// On the field with Plant Normal Monsters (Level 1 Dark Plant, Level 2 Firegrass) of other Levels.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (13193642, MZONE), (53293545, MZONE)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_LVCHANGE);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_target(duel, lvtg);
    e1.set_operation(duel, lvop);
    c.register_effect(duel, e1);
}

fn lvfilter(c: CardId, duel: &Duel, lvl: u32) -> bool {
    c.is_faceup(duel) && c.is_race(duel, RACE_PLANT) && c.has_level(duel) && !c.is_level(duel, &[lvl])
}

fn lvtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    let lvl = c.get_level(duel);
    if let Some(chkc) = chkc {
        return Ok(chkc != c && chkc.is_location(duel, LOCATION_MZONE) && lvfilter(chkc, duel, lvl));
    }
    if chk == 0 {
        return Ok(lvl > 0
            && duel.is_existing_target(
                |c, duel| lvfilter(c, duel, lvl),
                tp,
                LOCATION_MZONE,
                LOCATION_MZONE,
                1,
                Some(c.into()),
            ));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    duel.select_target(
        tp,
        |c, duel| lvfilter(c, duel, lvl),
        tp,
        LOCATION_MZONE,
        LOCATION_MZONE,
        1,
        1,
        Some(c.into()),
    )?;
    Ok(false)
}

fn lvop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if !(c.is_faceup(duel) && c.is_relate_to_effect(duel, a.e)) {
        return Ok(());
    }
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, a.e) && tc.is_faceup(duel) {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_CHANGE_LEVEL);
        e1.set_value(duel, tc.get_level(duel) as i32);
        e1.set_reset(duel, RESETS_STANDARD_DISABLE_PHASE_END);
        c.register_effect(duel, e1);
    }
    Ok(())
}
