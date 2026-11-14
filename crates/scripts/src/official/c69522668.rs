// Magikey Fiend - Transfurlmine
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, MZONE, OPPONENT};

const ID: u32 = 69522668;

pub const LISTED_SERIES: &[u16] = &[SET_MAGIKEY];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f1(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_set_card_full(duel, &[SET_MAGIKEY], Some(sc), sumtype, tp)
    }

    fn f2(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, |c, duel| c.is_type_full(duel, TYPE_NORMAL, Some(sc), sumtype, tp))
    }
}

// In the Extra Deck, Synchro Summoned with Maginificent Magikey Mafteal (a LIGHT Tuner) and Clavkiys (a DARK Normal Monster), with
// Magikey Battle in the Deck to Set and the opponent's Special Summons (Sangan, Iron Chain Dragon) to destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (15983048, MZONE),
    (98234196, MZONE),
    (98234196, MZONE),
    (54408264, DECK),
    (54408264, DECK),
    (54408264, DECK),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_EXTRA_ATTACK_MONSTER);
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2a = duel.create_effect(c);
    e2a.set_type(duel, EFFECT_TYPE_SINGLE);
    e2a.set_code(duel, EFFECT_MATERIAL_CHECK);
    e2a.set_value(duel, Value::MaterialCheck(matcheck));
    c.register_effect(duel, e2a);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_label_object(duel, Some(e2a));
    e2.set_condition(duel, setcon);
    e2.set_target(duel, settg);
    e2.set_operation(duel, setop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_DESTROY);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_SUMMON_SUCCESS);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_condition(duel, descon);
    e3.set_target(duel, destg);
    e3.set_operation(duel, desop);
    c.register_effect(duel, e3);
    let e4 = e3.clone_effect(duel);
    e4.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e4);
}

fn matcheck(duel: &mut Duel, e: EffectId, c: CardId) {
    let classes = c.get_material(duel).get_class_count(duel, |c, duel| c.get_attribute(duel).into());
    e.set_label(duel, classes as i64);
}

fn setcon(duel: &Duel, a: &Args) -> bool {
    let obj = a.e.get_label_object(duel);
    a.e.get_handler(duel).is_synchro_summoned(duel) && obj.is_some_and(|obj| obj.get_label(duel) > 1)
}

fn setfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_MAGIKEY]) && c.is_spell_trap(duel) && !c.is_type(duel, TYPE_FIELD) && c.is_ssetable(duel)
}

fn settg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(
            duel.get_location_count(a.tp, LOCATION_SZONE) > 0 && duel.is_existing_matching_card(setfilter, a.tp, LOCATION_DECK, 0, 1, None)
        );
    }
    Ok(false)
}

fn setop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SET);
    let g = duel.select_matching_card(tp, setfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if let Some(tc) = g.get_first() {
        duel.sset(tp, tc)?;
    }
    Ok(())
}

fn get_attr(duel: &Duel, tp: u8) -> u32 {
    duel.get_matching_group(CardId::is_monster, tp, LOCATION_GRAVE, 0, None)
        .get_bitwise_or(duel, |tc, duel| tc.get_attribute(duel).into()) as u32
}

fn desfilter(c: CardId, duel: &Duel, tp: u8, att: u32) -> bool {
    c.is_summon_player(duel, 1 - tp) && c.get_attribute(duel) & att > 0
}

fn descon(duel: &Duel, a: &Args) -> bool {
    let (tp, att) = (a.tp, get_attr(duel, a.tp));
    a.eg.is_some_and(|eg| eg.is_exists(duel, |c, duel| desfilter(c, duel, tp, att), 1, None))
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let (tp, att) = (a.tp, get_attr(duel, a.tp));
    let g = a.eg.unwrap_or_default().filter(duel, |c, duel| desfilter(c, duel, tp, att), None);
    duel.set_target_card(g);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), g.len() as u8, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp, att) = (a.e, a.tp, get_attr(duel, a.tp));
    let g =
        a.eg.unwrap_or_default()
            .filter(duel, |c, duel| desfilter(c, duel, tp, att), None)
            .filter(duel, |c, duel| c.is_relate_to_effect(duel, e), None);
    if !g.is_empty() {
        duel.destroy(g, REASON_EFFECT)?;
    }
    Ok(())
}
