// Cupid Pitch
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, MZONE};

const ID: u32 = 21915012;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

// In the Extra Deck, Synchro Summoned with Soul Resonator (Tuner) and Skull Servant, and on the field with Feral Imp for Red Dragon
// Archfiend, with Firegrass (600 DEF) in the Deck to search.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (62991792, MZONE),
    (32274490, MZONE),
    (ID, MZONE),
    (41392891, MZONE),
    (70902743, EXTRA),
    (70902743, EXTRA),
    (70902743, EXTRA),
    (53293545, DECK),
    (53293545, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_SINGLE);
    e0.set_code(duel, EFFECT_MATERIAL_CHECK);
    e0.set_label(duel, 0);
    e0.set_value(duel, Value::MaterialCheck(valcheck));
    e0.set_operation(duel, matop);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_LVCHANGE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_label_object(duel, Some(e0));
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_synchro_summoned(duel));
    e1.set_operation(duel, lvop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_value(duel, Value::Card(|duel, _, c| c.get_level(duel) as i32 * 400));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_DAMAGE + CATEGORY_SEARCH + CATEGORY_TOHAND);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e3.set_code(duel, EVENT_BE_MATERIAL);
    e3.set_count_limit(duel, 1, ID);
    e3.set_condition(duel, damcon);
    e3.set_target(duel, damtg);
    e3.set_operation(duel, damop);
    c.register_effect(duel, e3);
}

const SYNCHRO_MATERIAL: u64 = SUMMON_TYPE_SYNCHRO as u64 | MATERIAL_SYNCHRO;

fn puretuner(c: CardId, duel: &Duel, sc: CardId, tp: u8) -> bool {
    c.is_has_effect(duel, EFFECT_NONTUNER).is_none() && c.is_type_full(duel, TYPE_TUNER, Some(sc), SYNCHRO_MATERIAL, tp)
}

fn othertuner(c: CardId, duel: &Duel, sc: CardId, tp: u8) -> bool {
    (c.is_has_effect(duel, EFFECT_NONTUNER).is_some() && c.is_type_full(duel, TYPE_TUNER, Some(sc), SYNCHRO_MATERIAL, tp))
        || c.is_has_effect(duel, EFFECT_CAN_BE_TUNER).is_some()
}

fn valcheck(duel: &mut Duel, e: EffectId, c: CardId) {
    let mg = c.get_material(duel);
    if mg.is_empty() {
        e.set_label(duel, 0);
        return;
    }
    let tp = e.get_handler_player(duel);
    if let Some(tuner) = mg.filter(duel, |mc, duel| puretuner(mc, duel, c, tp), None).get_first() {
        e.set_label_object_group(duel, None);
        let level = tuner.get_synchro_level(duel, c);
        e.set_label(duel, i64::from(level));
        return;
    }
    let other_tuner_g = mg.filter(duel, |mc, duel| othertuner(mc, duel, c, tp), None);
    if other_tuner_g.len() > 1 {
        e.set_label_object_group(duel, Some(other_tuner_g));
    } else {
        e.set_label_object_group(duel, None);
        // An empty group is a nil index in the Lua.
        unported!(duel, other_tuner_g.is_empty());
        if let Some(tuner) = other_tuner_g.get_first() {
            let level = tuner.get_synchro_level(duel, c);
            e.set_label(duel, i64::from(level));
        }
    }
}

fn matop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, Some(c)) = (a.e, a.c) else { return Ok(()) };
    let Some(g) = e.get_label_object_group(duel) else { return Ok(()) };
    let tp = e.get_handler_player(duel);
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 5));
    if let Some(tuner) = g.select(duel, tp, 1, 1, None)?.get_first() {
        duel.hint_selection(tuner);
        let level = tuner.get_synchro_level(duel, c);
        e.set_label(duel, i64::from(level));
    }
    Ok(())
}

fn lvop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let level = e.get_label_object(duel).map_or(0, |e0| e0.get_label(duel));
    if level == 0 {
        return Ok(());
    }
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) && c.is_faceup(duel) {
        let opt = duel.select_option(tp, &[aux::stringid(ID, 2), aux::stringid(ID, 3)])?;
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_LEVEL);
        e1.set_value(duel, if opt == 0 { level } else { -level } as i32);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD_DISABLE);
        c.register_effect(duel, e1);
    }
    Ok(())
}

fn damcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_previous_location(duel, LOCATION_MZONE)
        && c.is_synchro_summoned(duel)
        && c.is_location(duel, LOCATION_GRAVE)
        && a.r & REASON_SYNCHRO > 0
}

fn damtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(true);
    }
    let Some(sc) = a.e.get_handler(duel).get_reason_card(duel) else {
        // The Lua errors on the nil reason card.
        unported!(duel, true);
        return Ok(false);
    };
    duel.set_target_card(sc);
    let damage = sc.get_level(duel) as i32 * 100;
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, tp, damage);
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_level_below(duel, 8) && c.is_defense(duel, &[600]) && c.is_able_to_hand(duel)
}

fn damop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(sc) = duel.get_first_target() else { return Ok(()) };
    if !(sc.is_relate_to_effect(duel, e) && sc.is_faceup(duel)) {
        return Ok(());
    }
    let damage = sc.get_level(duel) * 100;
    if duel.damage(1 - tp, damage, REASON_EFFECT)? > 0
        && duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None)
        && duel.select_yes_no(tp, aux::stringid(ID, 4))?
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
        let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
        if g.is_empty() {
            return Ok(());
        }
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
    }
    Ok(())
}
