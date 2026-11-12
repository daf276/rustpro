// Kerzebs, the Gaoled Glutton
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 94503794;

// Twice in the Extra Deck, Xyz Summoned with Level 7 Normal Monsters (Stone Dragon, Gem-Knight Crystal), with Normal Spells to
// attach activated from both hands.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (68171737, MZONE),
    (76908448, MZONE),
    (68171737, MZONE),
    (55144522, HAND),
    (83764718, HAND),
    (12580477, HAND | OPPONENT),
    (53129443, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(7);
    const CT: usize = 2;
    const MAXCT: usize = Xyz::INFINITE_MATS;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(duel, Value::Card(|duel, _, c| c.get_overlay_count(duel) as i32 * 700));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit_full(duel, 1, 0, EFFECT_COUNT_CODE_CHAIN);
    e2.set_condition(duel, attachcon);
    e2.set_operation(duel, attachop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_SET);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_PHASE + PHASE_END);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, ID);
    e3.set_target(duel, settg);
    e3.set_operation(duel, setop);
    c.register_effect(duel, e3);
}

fn attachcon(duel: &Duel, a: &Args) -> bool {
    let Some(re) = a.re else { return false };
    let rc = re.get_handler(duel);
    re.is_has_type(duel, EFFECT_TYPE_ACTIVATE)
        && (rc.is_normal_spell(duel) || rc.is_quick_play_spell(duel))
        && rc.is_on_field(duel)
        && rc.is_can_be_xyz_material(duel, Some(a.e.get_handler(duel)), a.tp, REASON_EFFECT)
}

fn attachop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(re) = a.re else { return Ok(()) };
    let (c, rc) = (e.get_handler(duel), re.get_handler(duel));
    if c.is_relate_to_effect(duel, e)
        && c.is_faceup(duel)
        && rc.is_relate_to_effect(duel, re)
        && !c.is_immune_to_effect(duel, e)
        && !rc.is_immune_to_effect(duel, e)
        && rc.is_can_be_xyz_material(duel, Some(c), tp, REASON_EFFECT)
    {
        duel.overlay(c, rc)?;
        if !c.get_overlay_group(duel).is_contains(rc) {
            return Ok(());
        }
        rc.cancel_to_grave(duel);
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_code(duel, EFFECT_IMMUNE_EFFECT);
        e1.set_value(duel, Value::Effect(|duel, e, te| e.get_label_object(duel) == Some(te)));
        e1.set_label_object(duel, Some(re));
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD | RESET_CHAIN);
        c.register_effect(duel, e1);
    }
    Ok(())
}

fn setfilter(c: CardId, duel: &Duel) -> bool {
    c.is_spell(duel) && c.is_ssetable(duel)
}

fn settg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(a.e.get_handler(duel).get_overlay_group(duel).is_exists(duel, setfilter, 1, None));
    }
    Ok(false)
}

fn setop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SET);
    let g = c.get_overlay_group(duel).filter_select(duel, tp, setfilter, 1, 1, None)?;
    if !g.is_empty() {
        duel.sset(tp, g)?;
    }
    Ok(())
}
