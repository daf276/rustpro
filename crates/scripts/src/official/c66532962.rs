// Magistus Chorozo
use crate::prelude::*;

const ID: u32 = 66532962;

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| {
            c.is_type_full(duel, TYPE_FUSION | TYPE_SYNCHRO | TYPE_XYZ | TYPE_LINK, Some(fc), sumtype, tp)
        }),
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_race_full(duel, RACE_SPELLCASTER, Some(fc), sumtype, tp)),
    ];
}

struct Contact;

impl ContactFusion for Contact {
    const CANNOT_BE_LIZARD: bool = false;

    fn group(duel: &Duel, tp: u8) -> CardSet {
        duel.get_matching_group(|c, duel| c.is_able_to_grave_as_cost(duel), tp, LOCATION_ONFIELD, 0, None)
    }

    fn op(duel: &mut Duel, g: CardSet, _: u8, _: CardId) -> Script<()> {
        duel.send_to_grave(g, REASON_COST | REASON_MATERIAL).map(|_| ())
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    c.add_must_first_be_fusion_summoned(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    Fusion::add_contact_proc::<Contact>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_ATKCHANGE + CATEGORY_TOHAND);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_target(duel, atktg);
    e1.set_operation(duel, atkop);
    c.register_effect(duel, e1);
}

fn atktg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let Some(ac) = duel.get_attacker() else { return Ok(false) };
    if let Some(chkc) = chkc {
        return Ok(chkc == ac);
    }
    if chk == 0 {
        return Ok(ac.is_on_field(duel) && ac.is_can_be_effect_target(duel, a.e));
    }
    duel.set_target_card(ac);
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, Some(ac.into()), 1, a.tp, 0);
    Ok(false)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let Some(tc) = duel.get_attacker() else { return Ok(()) };
    if !(tc.is_relate_to_effect(duel, e)
        && tc.is_faceup(duel)
        && duel.negate_attack()?
        && c.is_relate_to_effect(duel, e)
        && c.is_faceup(duel))
    {
        return Ok(());
    }
    let atk = tc.get_attack(duel);
    if atk <= 0 {
        return Ok(());
    }
    let prev_atk = c.get_attack(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_value(duel, atk);
    e1.set_reset(duel, RESETS_STANDARD_DISABLE_PHASE_END);
    c.register_effect(duel, e1);
    duel.adjust_instantly(Some(c));
    if prev_atk >= c.get_attack(duel) {
        return Ok(());
    }
    if tc.is_able_to_hand(duel) && duel.select_yes_no(tp, aux::stringid(ID, 1))? {
        duel.break_effect()?;
        duel.send_to_hand(tc, PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}
