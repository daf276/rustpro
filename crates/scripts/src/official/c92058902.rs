// Future Fusion Nova
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, OPPONENT};

const ID: u32 = 92058902;

pub const LISTED_NAMES: &[u32] = &[CARD_CYBER_DRAGON];

struct Params;

impl FusionSummonParams for Params {
    fn fusfilter(c: CardId, duel: &Duel) -> bool {
        c.is_attribute(duel, ATTRIBUTE_LIGHT) && c.is_race(duel, RACE_MACHINE)
    }

    fn matfilter(_: CardId, _: &Duel, _: EffectId, _: u8) -> bool {
        false
    }

    fn extrafil(duel: &Duel, e: EffectId, tp: u8, _: CardSet) -> (Option<CardSet>, Option<FusionCheck>) {
        let filter = |c, duel: &Duel| Fusion::is_monster_filter(c, duel, CardId::is_able_to_grave);
        (
            Some(duel.get_matching_group(filter, tp, LOCATION_DECK, 0, None)),
            Some((fcheck, e.get_handler(duel))),
        )
    }

    fn stage2(duel: &mut Duel, e: EffectId, tc: Option<CardId>, tp: u8, _: Option<CardSet>, chk: i32) -> Script<()> {
        let (Some(fc), 1, true) = (tc, chk, e.is_has_type(duel, EFFECT_TYPE_ACTIVATE)) else {
            return Ok(());
        };
        let c = e.get_handler(duel);
        c.set_card_target(duel, fc);
        let e1 = duel.create_effect(c);
        e1.set_description(duel, aux::stringid(ID, 1));
        e1.set_type(duel, EFFECT_TYPE_FIELD);
        e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
        e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
        e1.set_target_range(duel, 1, 0);
        e1.set_target_card(duel, |duel, _, c| c.is_race_except(duel, RACE_MACHINE));
        e1.set_reset(duel, RESET_PHASE | PHASE_END);
        duel.register_effect(e1, tp);
        let fid = fc.get_field_id(duel);
        aux::register_client_hint(duel, c, 0, tp, 1, 0, Some(aux::stringid(ID, 2)));
        let e2 = duel.create_effect(c);
        e2.set_type(duel, EFFECT_TYPE_FIELD);
        e2.set_code(duel, EFFECT_CANNOT_ATTACK_ANNOUNCE);
        e2.set_target_range(duel, LOCATION_MZONE, 0);
        e2.set_target_card(duel, |duel, e, c| i64::from(c.get_field_id(duel)) != e.get_label(duel));
        e2.set_label(duel, i64::from(fid));
        e2.set_reset(duel, RESET_PHASE | PHASE_END);
        duel.register_effect(e2, tp);
        Ok(())
    }

    fn extratg(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
        if chk != 0 {
            duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, a.tp, LOCATION_DECK as i32);
        }
        Ok(true)
    }
}

// Cyber Twin Dragon in the Extra Deck, Fusion Summoned from two Cyber Dragons in the Deck, with Dark Hole in the opponent's hand to destroy it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (74157028, EXTRA),
    (CARD_CYBER_DRAGON, DECK),
    (CARD_CYBER_DRAGON, DECK),
    (CARD_CYBER_DRAGON, DECK),
    (53129443, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummonedCode(74157028)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = Fusion::create_summon_eff::<Params>(duel, c);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e2.set_code(duel, EVENT_LEAVE_FIELD_P);
    e2.set_operation(duel, |duel, a| {
        let label = i64::from(a.e.get_handler(duel).is_disabled(duel));
        a.e.set_label(duel, label);
        Ok(())
    });
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e3.set_code(duel, EVENT_LEAVE_FIELD);
    e3.set_operation(duel, mondesop);
    e3.set_label_object(duel, Some(e2));
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e4.set_code(duel, EVENT_LEAVE_FIELD);
    e4.set_range(duel, LOCATION_SZONE);
    e4.set_condition(duel, selfdescon);
    e4.set_operation(duel, |duel, a| {
        duel.destroy(a.e.get_handler(duel), REASON_EFFECT)?;
        Ok(())
    });
    c.register_effect(duel, e4);
}

fn fcheck(duel: &Duel, _: CardId, _: u8, sg: CardSet, _: CardId) -> bool {
    sg.is_exists(duel, |c, duel| c.is_code(duel, CARD_CYBER_DRAGON), 1, None)
}

fn mondesop(duel: &mut Duel, a: &Args) -> Script<()> {
    if a.e.get_label_object(duel).is_some_and(|e2| e2.get_label(duel) != 0) {
        return Ok(());
    }
    if let Some(tc) = a.e.get_handler(duel).get_first_card_target(duel)
        && tc.is_location(duel, LOCATION_MZONE)
    {
        duel.destroy(tc, REASON_EFFECT)?;
    }
    Ok(())
}

fn selfdescon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel)
        .get_first_card_target(duel)
        .is_some_and(|tc| a.eg.is_some_and(|eg| eg.contains(tc)) && tc.is_reason(duel, REASON_DESTROY))
}
