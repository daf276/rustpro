// Zenna's Deceiving Doll Maidens
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 7594154;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_type_full(duel, TYPE_EFFECT, Some(lc), sumtype, tp)
    }
}

// Link Summoned with a Sangan on the field and one from the hand (extra material), then sent to the Graveyard by Raigeki.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (26202165, MZONE),
    (26202165, HAND),
    (26202165, HAND),
    (12580477, HAND | OPPONENT),
];
// No MaterialFrom check: the hand material is a Sangan, and the check counts materials by the card under test.
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_FIELD);
    e0.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CANNOT_DISABLE);
    e0.set_code(duel, EFFECT_EXTRA_MATERIAL);
    e0.set_range(duel, LOCATION_EXTRA);
    e0.set_target_range(duel, 1, 0);
    e0.set_extra_material_operation(duel, |duel, _, _, _, sg, _, _, _, _| {
        sg.filter_count(duel, |c, duel| c.has_flag_effect(duel, ID), None) <= 1
    });
    e0.set_value(duel, Value::ExtraMaterial(extraval));
    c.register_effect(duel, e0);
    let e1a = duel.create_effect(c);
    e1a.set_description(duel, aux::stringid(ID, 0));
    e1a.set_category(duel, CATEGORY_TOGRAVE);
    e1a.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1a.set_property(duel, EFFECT_FLAG_DELAY);
    e1a.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1a.set_count_limit(duel, 1, (ID, 0));
    e1a.set_condition(duel, |duel, a| a.e.get_handler(duel).is_link_summoned(duel));
    e1a.set_target(duel, tgtg);
    e1a.set_operation(duel, tgop);
    c.register_effect(duel, e1a);
    let e1b = duel.create_effect(c);
    e1b.set_type(duel, EFFECT_TYPE_SINGLE);
    e1b.set_code(duel, EFFECT_MATERIAL_CHECK);
    e1b.set_value(duel, Value::MaterialCheck(matcheck));
    e1b.set_label_object(duel, Some(e1a));
    c.register_effect(duel, e1b);
    let e2a = duel.create_effect(c);
    e2a.set_description(duel, aux::stringid(ID, 1));
    e2a.set_category(duel, CATEGORY_TOHAND);
    e2a.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2a.set_code(duel, EVENT_PHASE + PHASE_STANDBY);
    e2a.set_range(duel, LOCATION_GRAVE);
    e2a.set_count_limit(duel, 1, (ID, 1));
    e2a.set_condition(duel, |duel, a| {
        let c = a.e.get_handler(duel);
        duel.is_turn_player(a.tp)
            && c.has_flag_effect(duel, ID)
            && (c.get_flag_effect_label(duel, ID) == Some(1) || c.get_turn_id(duel) != duel.get_turn_count())
    });
    e2a.set_target(duel, thtg);
    e2a.set_operation(duel, thop);
    c.register_effect(duel, e2a);
    let e2b = duel.create_effect(c);
    e2b.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e2b.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e2b.set_code(duel, EVENT_TO_GRAVE);
    e2b.set_condition(duel, |duel, a| a.e.get_handler(duel).is_previous_location(duel, LOCATION_ONFIELD));
    e2b.set_operation(duel, |duel, a| {
        let ct = if duel.is_turn_player(a.tp) && duel.is_phase(PHASE_STANDBY) {
            2
        } else {
            1
        };
        let reset = RESET_EVENT | RESETS_STANDARD | RESET_PHASE | PHASE_STANDBY | RESET_SELF_TURN;
        a.e.get_handler(duel)
            .register_flag_effect_full(duel, ID, reset, 0, ct, i64::from(ct), 0);
        Ok(())
    });
    c.register_effect(duel, e2b);
}

fn extraval(duel: &mut Duel, e: EffectId, call: ExtraMaterialCall) -> CardSet {
    match call {
        ExtraMaterialCall::Get { summon_type, tp, sc } => {
            if summon_type != SUMMON_TYPE_LINK || sc != e.get_handler(duel) {
                return CardSet::default();
            }
            let g = duel.get_matching_group(|c, duel| !c.has_flag_effect(duel, ID), tp, LOCATION_HAND, 0, None);
            for mc in g {
                mc.register_flag_effect(duel, ID, 0, 0, 1);
            }
            g
        }
        ExtraMaterialCall::Use { .. } => CardSet::default(),
        ExtraMaterialCall::Delete { .. } => {
            let tp = e.get_handler_player(duel);
            let g = duel.get_matching_group(|c, duel| c.has_flag_effect(duel, ID), tp, LOCATION_HAND, 0, None);
            for mc in g {
                mc.reset_flag_effect(duel, ID);
            }
            CardSet::default()
        }
    }
}

fn tglocations(duel: &Duel, e: EffectId) -> u32 {
    if e.get_label(duel) == 0 {
        LOCATION_DECK
    } else {
        LOCATION_DECK | LOCATION_EXTRA
    }
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel) && c.is_able_to_grave(duel)
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let locations = tglocations(duel, a.e);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(tgfilter, tp, locations, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, locations as i32);
    Ok(false)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let locations = tglocations(duel, a.e);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, tgfilter, tp, locations, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_grave(g, REASON_EFFECT)?;
    }
    Ok(())
}

fn matfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_location(duel, LOCATION_MZONE) && c.is_controler(duel, tp)
}

fn matcheck(duel: &mut Duel, e: EffectId, c: CardId) {
    let mg = c.get_material(duel);
    let tp = e.get_handler_player(duel);
    let label = i64::from(mg.filter_count(duel, |mc, duel| matfilter(mc, duel, tp), None) == mg.len());
    if let Some(e1) = e.get_label_object(duel) {
        e1.set_label(duel, label);
    }
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_GRAVE, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.hint_selection(g);
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}
