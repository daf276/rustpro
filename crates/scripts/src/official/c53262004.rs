// Odd-Eyes Vortex Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 53262004;

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_set_card_full(duel, &[SET_ODD_EYES], Some(fc), sumtype, tp)),
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_type_full(duel, TYPE_PENDULUM, Some(fc), sumtype, tp)),
    ];
}

pub const MATERIAL_SETCODE: &[u16] = &[SET_ODD_EYES];

// In the Extra Deck to Fusion Summon with Polymerization from Odd-Eyes Absolute Dragon and Flash Knight (Pendulum Normal
// Monster) on the field, and the opponent's Raigeki to send a Pendulum Monster to the Extra Deck face-up.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (16691074, MZONE),
    (17390179, MZONE),
    (CARD_POLYMERIZATION, HAND),
    (CARD_POLYMERIZATION, HAND),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_TOHAND);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_category(duel, CATEGORY_NEGATE + CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, discon);
    e2.set_target(duel, distg);
    e2.set_operation(duel, disop);
    c.register_effect(duel, e2);
    aux::double_snare_validity(duel, c, LOCATION_MZONE);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_position(duel, POS_FACEUP_ATTACK) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, 1 - tp) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
    let g = duel.select_target(tp, filter, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOHAND, g, 1, 0, 0);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.send_to_hand(tc, PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}

fn discon(duel: &Duel, a: &Args) -> bool {
    let (c, Some(re)) = (a.e.get_handler(duel), a.re) else {
        return false;
    };
    re.get_handler(duel) != c
        && !c.is_status(duel, STATUS_BATTLE_DESTROYED)
        && (re.is_monster_effect(duel) || re.is_has_type(duel, EFFECT_TYPE_ACTIVATE))
        && duel.is_chain_negatable(a.ev as u8)
}

fn disfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_type(duel, TYPE_PENDULUM) && c.is_able_to_deck(duel)
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(disfilter, tp, LOCATION_EXTRA, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TODECK, None, 1, tp, LOCATION_EXTRA as i32);
    duel.set_operation_info(0, CATEGORY_NEGATE, a.eg, 1, 0, 0);
    if let Some(re) = a.re {
        let rc = re.get_handler(duel);
        if rc.is_destructable(duel) && rc.is_relate_to_effect(duel, re) {
            duel.set_operation_info(0, CATEGORY_DESTROY, a.eg, 1, 0, 0);
        }
    }
    Ok(false)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let g = duel.select_matching_card(tp, disfilter, tp, LOCATION_EXTRA, 0, 1, 1, None)?;
    if duel.send_to_deck(g, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)? != 0
        && duel.negate_activation(a.ev as u8)
        && a.re.is_some_and(|re| re.get_handler(duel).is_relate_to_effect(duel, re))
    {
        duel.destroy(a.eg.unwrap_or_default(), REASON_EFFECT)?;
    }
    Ok(())
}
