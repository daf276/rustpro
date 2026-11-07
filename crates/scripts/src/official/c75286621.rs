// Invoked Mechaba
use crate::prelude::*;
use crate::testing::{HAND, MZONE, OPPONENT};

const ID: u32 = 75286621;
const ALEISTER_THE_INVOKER: u32 = 86120751;

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Code(ALEISTER_THE_INVOKER),
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_attribute_full(duel, ATTRIBUTE_LIGHT, Some(fc), sumtype, tp)),
    ];
}

// On the field to negate the opponent's Elfnote Lucina, a monster effect, with another Lucina in hand to discard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (13597785, MZONE | OPPONENT),
    (13597785, HAND | OPPONENT),
    (13597785, HAND),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_NEGATE + CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_condition(duel, negcon);
    e1.set_cost(duel, negcost);
    e1.set_target(duel, negtg);
    e1.set_operation(duel, negop);
    c.register_effect(duel, e1);
    aux::double_snare_validity(duel, c, LOCATION_MZONE);
}

fn negcon(duel: &Duel, a: &Args) -> bool {
    let Some(re) = a.re else { return false };
    !a.e.get_handler(duel).is_status(duel, STATUS_BATTLE_DESTROYED)
        && (re.is_monster_effect(duel) || re.is_has_type(duel, EFFECT_TYPE_ACTIVATE))
        && duel.is_chain_negatable(a.ev as u8)
}

fn cfilter(c: CardId, duel: &Duel, rtype: u32) -> bool {
    c.is_type(duel, rtype) && c.is_able_to_grave_as_cost(duel)
}

fn negcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    let rtype = a.re.map_or(0, |re| re.get_active_type(duel)) & (TYPE_MONSTER | TYPE_SPELL | TYPE_TRAP);
    let filter = |c, duel: &Duel| cfilter(c, duel, rtype);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(filter, tp, LOCATION_HAND, 0, 1, None));
    }
    duel.discard_hand(tp, filter, 1, 1, REASON_COST)?;
    Ok(false)
}

fn negtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let Some(re) = a.re else { return Ok(false) };
    let rc = re.get_handler(duel);
    let relation = rc.is_relate_to_effect(duel, re);
    if chk == 0 {
        return Ok(
            rc.is_able_to_remove_full(duel, Some(tp), POS_FACEUP, REASON_EFFECT) || (!relation && duel.is_player_can_remove(tp, None))
        );
    }
    duel.set_operation_info(0, CATEGORY_NEGATE, a.eg, 1, 0, 0);
    if relation {
        duel.set_operation_info(
            0,
            CATEGORY_REMOVE,
            Some(rc.into()),
            1,
            rc.get_controler(duel),
            rc.get_location(duel) as i32,
        );
    } else {
        duel.set_operation_info(0, CATEGORY_REMOVE, None, 0, 0, rc.get_previous_location(duel) as i32);
    }
    Ok(false)
}

fn negop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(re) = a.re else { return Ok(()) };
    if duel.negate_activation(a.ev as u8)
        && re.get_handler(duel).is_relate_to_effect(duel, re)
        && let Some(eg) = a.eg
    {
        duel.remove(eg, POS_FACEUP, REASON_EFFECT)?;
    }
    Ok(())
}
