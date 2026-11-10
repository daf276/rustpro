// Lightstorm Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 10515412;

// In the Extra Deck, Synchro Summoned with Angel Trumpeter (Level 4 Tuner) and Peacock (Level 5), or Dragon Core Hexer (Level 8
// Tuner) and Key Mace (Level 1), with Normal Monsters and a Spell in the Graveyard and Spells/Traps and Dark Hole on the other side.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (87979586, MZONE),
    (20624263, MZONE),
    (38955728, MZONE),
    (1929294, MZONE),
    (20624263, GRAVE),
    (5318639, GRAVE),
    (5318639, SZONE | OPPONENT),
    (53129443, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SET);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_DESTROYED);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).is_reason(duel, REASON_BATTLE | REASON_EFFECT));
    e2.set_target(duel, settg);
    e2.set_operation(duel, setop);
    c.register_effect(duel, e2);
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_spell_trap(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_spell_trap, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None));
    }
    let ct = duel.get_matching_group_count(|c, duel| c.is_type(duel, TYPE_NORMAL), tp, LOCATION_GRAVE, 0, None);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(
        tp,
        CardId::is_spell_trap,
        tp,
        LOCATION_ONFIELD,
        LOCATION_ONFIELD,
        1,
        (ct + 1) as u8,
        None,
    )?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, g.map_or(0, |g| g.len() as u8), tp, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tg = duel.get_target_cards(a.e).unwrap_or_default();
    if !tg.is_empty() {
        duel.destroy(tg, REASON_EFFECT)?;
    }
    Ok(())
}

fn setfilter(c: CardId, duel: &Duel) -> bool {
    c.is_spell_trap(duel) && c.is_ssetable(duel)
}

fn settg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && setfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(setfilter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SET);
    let g = duel.select_target(tp, setfilter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, g, 1, tp, 0);
    Ok(false)
}

fn setop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, e) && duel.sset(tp, tc)? > 0 {
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_description(duel, aux::stringid(ID, 2));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
        e1.set_code(duel, EFFECT_CANNOT_TRIGGER);
        e1.set_label(duel, i64::from(tp));
        e1.set_condition(duel, |duel, a| {
            let tp = a.e.get_label(duel) as u8;
            !duel.is_existing_matching_card(
                aux::faceup_filter(|c, duel| c.is_type(duel, TYPE_NORMAL)),
                tp,
                LOCATION_MZONE,
                0,
                1,
                None,
            )
        });
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        tc.register_effect(duel, e1);
        let e2 = duel.create_effect(e.get_handler(duel));
        e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        e2.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
        e2.set_code(duel, EVENT_CHAINING);
        e2.set_count_limit(duel, 1, 0);
        e2.set_condition(duel, |duel, a| {
            a.re.is_some_and(|re| Some(re.get_handler(duel)) == a.e.get_label_object_group(duel).and_then(|g| g.get_first()))
        });
        e2.set_operation(duel, |duel, a| {
            if let Some(e1) = a.e.get_label_object(duel) {
                e1.reset(duel);
            }
            a.e.reset(duel);
            Ok(())
        });
        e2.set_label_object(duel, Some(e1));
        e2.set_label_object_group(duel, Some(tc.into()));
        duel.register_effect(e2, tp);
        let e3 = duel.create_effect(e.get_handler(duel));
        e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
        e3.set_code(duel, EVENT_LEAVE_FIELD_P);
        e3.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e3.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        e3.set_operation(duel, reseteffect);
        e3.set_label_object(duel, Some(e2));
        tc.register_effect(duel, e3);
    }
    Ok(())
}

fn reseteffect(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(on_act_eff) = a.e.get_label_object(duel).filter(|eff| !eff.is_deleted(duel)) {
        on_act_eff.reset(duel);
    }
    Ok(())
}
