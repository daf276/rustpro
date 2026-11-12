// Pain Painter
use crate::prelude::*;
use crate::testing::MZONE;

const ID: u32 = 21620076;

// On the field with Skull Servant and Bone Mouse (Zombies) to change the Level of.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (32274490, MZONE), (21239280, MZONE)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_CHANGE_CODE);
    e1.set_value(duel, 33420078);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_LVCHANGE);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_target(duel, lvtg);
    e2.set_operation(duel, lvop);
    c.register_effect(duel, e2);
}

fn lvfilter(c: CardId, duel: &Duel) -> bool {
    let lv = c.get_level(duel);
    c.is_faceup(duel) && c.is_race(duel, RACE_ZOMBIE) && lv > 0 && lv != 2
}

fn lvtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && lvfilter(chkc, duel));
    }
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.is_existing_target(lvfilter, tp, LOCATION_MZONE, 0, 1, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    duel.select_target(tp, lvfilter, tp, LOCATION_MZONE, 0, 1, 2, Some(c.into()))?;
    Ok(false)
}

fn lvop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    for tc in duel.get_target_cards(a.e).unwrap_or_default() {
        if tc.is_faceup(duel) {
            let e1 = duel.create_effect(c);
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_code(duel, EFFECT_CHANGE_LEVEL);
            e1.set_value(duel, 2);
            e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
            tc.register_effect(duel, e1);
            let e2 = duel.create_effect(c);
            e2.set_type(duel, EFFECT_TYPE_SINGLE);
            e2.set_code(duel, EFFECT_CANNOT_BE_SYNCHRO_MATERIAL);
            e2.set_value(duel, Value::Card(|duel, _, c| i32::from(!c.is_race(duel, RACE_ZOMBIE))));
            e2.set_reset(duel, RESETS_STANDARD_PHASE_END);
            tc.register_effect(duel, e2);
        }
    }
    Ok(())
}
