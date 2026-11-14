// Herald of the Abyss
use crate::prelude::*;
use crate::testing::{HAND, SZONE};

const ID: u32 = 94016752;

pub const SUPPORT: &[(u32, u16)] = &[(ID, SZONE), (ID, HAND)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOGRAVE);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_condition(duel, |duel, a| {
        duel.is_existing_matching_card(CardId::is_faceup, a.tp, 0, LOCATION_MZONE, 1, None)
    });
    e1.set_cost(duel, Cost::pay_lp::<1500>);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(true);
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RACE);
    let race = duel.announce_race(tp, 1, RACE_ALL)?;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATTRIBUTE);
    let attr = duel.announce_attribute(tp, 1, ATTRIBUTE_ALL)?;
    e.set_labels(duel, &[race as i64, attr.into()]);
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, 1 - tp, LOCATION_MZONE as i32);
    Ok(false)
}

fn tgfilter(c: CardId, duel: &Duel, race: u64, attr: u32) -> bool {
    c.is_faceup(duel) && c.is_race(duel, race) && c.is_attribute(duel, attr) && c.is_able_to_grave(duel)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let &[race, attr] = e.get_labels(duel) else { return Ok(()) };
    duel.hint(HINT_SELECTMSG, 1 - tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(
        1 - tp,
        |c, duel| tgfilter(c, duel, race as u64, attr as u32),
        tp,
        0,
        LOCATION_MZONE,
        1,
        1,
        None,
    )?;
    let Some(sc) = g.get_first() else { return Ok(()) };
    duel.hint_selection_full(sc, true);
    let code = sc.get_code(duel);
    if duel.send_to_grave_full(sc, REASON_RULE, PLAYER_NONE, 1 - tp)? > 0 {
        sc.register_flag_effect(duel, ID, RESETS_STANDARD_PHASE_END, 0, 1);
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_FIELD);
        e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
        e1.set_code(duel, EFFECT_CANNOT_ACTIVATE);
        e1.set_target_range(duel, 0, 1);
        e1.set_value(duel, Value::EffectPlayer(aclimit));
        e1.set_label(duel, code.into());
        e1.set_reset(duel, RESET_PHASE | PHASE_END);
        duel.register_effect(e1, tp);
    }
    Ok(())
}

fn aclimit(duel: &Duel, e: EffectId, re: EffectId, _: u8) -> bool {
    let rc = re.get_handler(duel);
    re.is_monster_effect(duel) && (rc.has_flag_effect(duel, ID) || rc.is_code(duel, e.get_label(duel) as u32))
}
