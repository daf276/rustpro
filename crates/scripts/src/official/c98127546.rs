// Underworld Goddess of the Closed World
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};
use engine::OpTarget;

const ID: u32 = 98127546;
const CLOSED_SKY_FLAG: u32 = 71818935;
// s.curgroup, the opponent's monsters offered as extra Link Material, kept as a script group.
const CURGROUP: u64 = (ID as u64) << 16;

// Sangan (an Effect Monster) as Link Material, one of them the opponent's, and the opponent's Monster Reborn with Sangan in its
// Graveyard to negate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (26202165, MZONE),
    (26202165, MZONE),
    (26202165, MZONE),
    (26202165, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, GRAVE | OPPONENT),
    (83764718, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::MaterialFrom(MZONE | OPPONENT),
];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 4;

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_type_full(duel, TYPE_EFFECT, Some(lc), sumtype, tp)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_FIELD);
    e0.set_property(
        duel,
        EFFECT_FLAG_PLAYER_TARGET | EFFECT_FLAG_CANNOT_DISABLE | EFFECT_FLAG_SET_AVAILABLE,
    );
    e0.set_code(duel, EFFECT_EXTRA_MATERIAL);
    e0.set_range(duel, LOCATION_EXTRA);
    e0.set_target_range(duel, 1, 1);
    e0.set_extra_material_operation(duel, extracon);
    e0.set_value(duel, Value::ExtraMaterial(extraval));
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DISABLE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_link_summoned(duel));
    e1.set_target(duel, distg);
    e1.set_operation(duel, disop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_code(duel, EFFECT_IMMUNE_EFFECT);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_value(duel, Value::Effect(immval));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_NEGATE);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e3.set_code(duel, EVENT_CHAINING);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_condition(duel, spnegcon);
    e3.set_target(duel, spnegtg);
    e3.set_operation(duel, |duel, a| {
        duel.negate_activation(a.ev as u8);
        Ok(())
    });
    c.register_effect(duel, e3);
}

fn closed_sky_filter(c: CardId, duel: &Duel) -> bool {
    !(c.has_flag_effect(duel, CLOSED_SKY_FLAG) && !c.get_card_target(duel).is_empty())
}

fn extracon(duel: &Duel, _: CardId, _: EffectId, tp: u8, sg: CardSet, _: CardSet, _: CardId, _: CardSet, _: i32) -> bool {
    let Some(curgroup) = duel.get_script_group(CURGROUP) else {
        return true;
    };
    let g = curgroup.filter(duel, closed_sky_filter, None);
    let effs = duel.get_player_effect(tp, EFFECT_EXTRA_MATERIAL);
    // Calling another card's extra material value registers effects, which a condition cannot.
    unported!(duel, effs.iter().any(|eff| !eff.get_owner(duel).is_code(duel, ID)));
    sg.intersection(g).len() <= 1
}

fn extraval(duel: &mut Duel, e: EffectId, call: ExtraMaterialCall) -> CardSet {
    match call {
        ExtraMaterialCall::Get { summon_type, tp, sc } => {
            if summon_type != SUMMON_TYPE_LINK || sc != e.get_handler(duel) {
                return CardSet::default();
            }
            let g = duel.get_matching_group(CardId::is_faceup, tp, 0, LOCATION_MZONE, None);
            duel.set_script_group(CURGROUP, Some(g));
            g
        }
        ExtraMaterialCall::Use { .. } => CardSet::default(),
        ExtraMaterialCall::Delete { .. } => {
            duel.set_script_group(CURGROUP, None);
            CardSet::default()
        }
    }
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_negatable_monster, a.tp, 0, LOCATION_MZONE, 1, None));
    }
    Ok(false)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_matching_group(CardId::is_negatable_monster, a.tp, 0, LOCATION_MZONE, None);
    if g.is_empty() {
        return Ok(());
    }
    let c = a.e.get_handler(duel);
    for tc in g {
        tc.negate_effects(duel, c, 0, false);
    }
    Ok(())
}

fn immval(duel: &Duel, e: EffectId, re: EffectId) -> bool {
    let c = e.get_handler(duel);
    if !(re.is_activated(duel) && c.is_link_summoned(duel) && e.get_owner_player(duel) == 1 - re.get_owner_player(duel)) {
        return false;
    }
    if !re.is_has_property(duel, EFFECT_FLAG_CARD_TARGET) {
        return true;
    }
    duel.get_chain_info::<Option<CardSet>>(0, CHAININFO_TARGET_CARDS)
        .is_none_or(|g| !g.is_contains(c))
}

fn spnegcon(duel: &Duel, a: &Args) -> bool {
    let ev = a.ev as u8;
    if a.e.get_handler(duel).is_status(duel, STATUS_BATTLE_DESTROYED) || a.rp == a.tp || !duel.is_chain_negatable(ev) {
        return false;
    }
    let from_grave = |info: Option<OpTarget>| {
        info.is_some_and(|info| {
            info.op_param as u32 & LOCATION_GRAVE > 0
                || info
                    .op_cards
                    .is_some_and(|tg| tg.is_exists(duel, |c, duel| c.is_location(duel, LOCATION_GRAVE) && c.is_monster(duel), 1, None))
        })
    };
    from_grave(duel.get_operation_info(ev, CATEGORY_SPECIAL_SUMMON))
        || from_grave(duel.get_possible_operation_info(ev, CATEGORY_SPECIAL_SUMMON))
}

fn spnegtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_NEGATE, a.eg, 1, a.tp, 0);
    Ok(false)
}
