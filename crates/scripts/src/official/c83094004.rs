// Aeropixthree
use crate::prelude::*;
use crate::testing::{MZONE, OPPONENT};

const ID: u32 = 83094004;

pub const COUNTER_PLACE_LIST: &[u16] = &[0x1207];
const COUNTER: u16 = 0x1207;

// On the field with Sangan on the opponent's side to move into its column and put a counter on.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_COUNTER);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER_E);
    e1.set_target(duel, seqtg);
    e1.set_operation(duel, seqop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e2.set_value(duel, Value::Card(val));
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EFFECT_UPDATE_DEFENSE);
    c.register_effect(duel, e3);
}

fn seqfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    let g = e.get_handler(duel).get_column_group(duel);
    c.is_faceup(duel)
        && g.is_contains(c)
        && (0..5).any(|i| duel.check_location(tp, LOCATION_MZONE, i) && duel.check_location(1 - tp, LOCATION_MZONE, 4 - i))
}

fn seqtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, 1 - tp) && seqfilter(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.get_location_count(1 - tp, LOCATION_MZONE) > 0
            && duel.is_existing_target(|c, duel| seqfilter(c, duel, e, tp), tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    duel.select_target(tp, |c, duel| seqfilter(c, duel, e, tp), tp, 0, LOCATION_MZONE, 1, 1, None)?;
    Ok(false)
}

fn seqop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !c.is_relate_to_effect(duel, e)
        || c.is_controler(duel, 1 - tp)
        || !tc.is_relate_to_effect(duel, e)
        || tc.is_controler(duel, tp)
        || c.is_immune_to_effect(duel, e)
        || tc.is_immune_to_effect(duel, e)
        || duel.get_location_count(tp, LOCATION_MZONE) <= 0
        || duel.get_location_count(1 - tp, LOCATION_MZONE) <= 0
    {
        return Ok(());
    }
    let zone = (0..5)
        .filter(|&i| duel.check_location(tp, LOCATION_MZONE, i) && duel.check_location(1 - tp, LOCATION_MZONE, 4 - i))
        .fold(0u32, |zone, i| zone | 1 << i);
    if zone == 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOZONE);
    let selzone = duel.select_disable_field(tp, 1, LOCATION_MZONE as u16, 0, 0xff_ffff & !zone)?;
    unported!(duel, selzone.is_none());
    let Some(sequence) = selzone.map(|selzone| selzone.trailing_zeros() as u8) else {
        return Ok(());
    };
    duel.move_sequence(c, sequence)?;
    duel.move_sequence(tc, 4 - sequence)?;
    if tc.is_faceup(duel) {
        duel.break_effect()?;
        tc.add_counter(duel, COUNTER, 1);
    }
    Ok(())
}

fn val(duel: &Duel, _: EffectId, c: CardId) -> i32 {
    i32::from(c.get_counter(duel, COUNTER)) * -200
}
