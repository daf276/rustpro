// Performapal Odd-Eyes Dissolver
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT, PZONE};

const ID: u32 = 46136942;

struct DragonParams;

impl FusionSummonParams for DragonParams {
    fn fusfilter(c: CardId, duel: &Duel) -> bool {
        c.is_race(duel, RACE_DRAGON)
    }
}

struct HandlerParams;

impl FusionSummonParams for HandlerParams {
    fn matfilter(c: CardId, duel: &Duel, e: EffectId, _: u8) -> bool {
        Fusion::check_with_handler(c, duel, e, Fusion::on_field_mat)
    }

    fn extrafil(duel: &Duel, _: EffectId, tp: u8, _: CardSet) -> (Option<CardSet>, Option<FusionCheck>) {
        (
            Some(duel.get_matching_group(CardId::is_able_to_grave, tp, LOCATION_PZONE, 0, None)),
            None,
        )
    }

    fn gc(duel: &Duel, e: EffectId, _: u8) -> Option<CardSet> {
        Some(Fusion::forced_handler(duel, e))
    }
}

// In a Pendulum Zone with Firegrass and Petit Dragon for Darkfire Dragon; on the field with DARK monsters (Dark
// Magician) for Starving Venom Fusion Dragon; in hand with Pendulums (Dragong) on the field to battle Sangan and the
// opponent's Elfnote Lucina.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, PZONE),
    (ID, MZONE),
    (ID, HAND),
    (ID, HAND),
    (83980492, MZONE),
    (83980492, MZONE),
    (83980492, MZONE),
    (13597785, MZONE | OPPONENT),
    (53293545, HAND),
    (75356564, HAND),
    (46986414, MZONE),
    (46986414, MZONE),
    (17881964, EXTRA),
    (17881964, EXTRA),
    (41209827, EXTRA),
    (41209827, EXTRA),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::SpecialSummonedCode(41209827),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Pendulum::add_procedure(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_FUSION_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_PZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_target(duel, Fusion::summon_eff_tg::<DragonParams>);
    e1.set_operation(duel, Fusion::summon_eff_op::<DragonParams>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_BATTLE_START);
    e2.set_range(duel, LOCATION_HAND);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition_mut(duel, spcon);
    e2.set_target(duel, stock::special_summon_self_target_player_0);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_FUSION_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_target(duel, Fusion::summon_eff_tg::<HandlerParams>);
    e3.set_operation(duel, Fusion::summon_eff_op::<HandlerParams>);
    c.register_effect(duel, e3);
}

fn spcon(duel: &mut Duel, a: &Args) -> bool {
    let tp = a.tp;
    let Some(attacker) = duel.get_attacker() else { return false };
    let Some(target) = attacker.get_battle_target(duel) else {
        return false;
    };
    let (a_card, d_card) = if attacker.is_controler(duel, 1 - tp) {
        (target, attacker)
    } else {
        (attacker, target)
    };
    a.e.set_label_object_group(duel, Some(a_card.into()));
    a_card.is_controler(duel, tp)
        && a_card.is_faceup(duel)
        && a_card.is_type(duel, TYPE_PENDULUM)
        && a_card.get_controler(duel) != d_card.get_controler(duel)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) && duel.special_summon(c, 0, a.tp, a.tp, false, false, POS_FACEUP)? != 0 {
        let Some(tc) = a.e.get_label_object_group(duel).and_then(|g| g.get_first()) else {
            return Ok(());
        };
        if !tc.is_relate_to_battle(duel) {
            return Ok(());
        }
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
        e1.set_value(duel, 1);
        e1.set_reset(duel, RESET_PHASE | PHASE_DAMAGE);
        tc.register_effect(duel, e1);
    }
    Ok(())
}
