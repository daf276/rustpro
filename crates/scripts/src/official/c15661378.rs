// Trishula, the Dragon of Icy Imprisonment
use crate::prelude::*;
use crate::testing::{EXTRA, HAND, MZONE};

const ID: u32 = 15661378;

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[FusionMaterial::GroupFilter(ffilter); 3];
    const MATCHECK: Option<MatCheck> = Some(matfilter);
}

struct Contact;

impl ContactFusion for Contact {
    const SUMCON: Option<Value> = Some(Value::SpSummonCondition(splimit));
    const CANNOT_BE_LIZARD: bool = false;

    fn group(duel: &Duel, tp: u8) -> CardSet {
        duel.get_matching_group(filteraux, tp, LOCATION_ONFIELD, 0, None)
    }

    fn op(duel: &mut Duel, g: CardSet, _: u8, _: CardId) -> Script<()> {
        duel.remove(g, POS_FACEUP, REASON_COST | REASON_MATERIAL)?;
        Ok(())
    }
}

// Two copies in the Extra Deck, Summoned with Dragons of different names on the field, and Polymerization in the hand.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (CARD_POLYMERIZATION, HAND),
    (11091375, MZONE),
    (17658803, MZONE),
    (20315854, MZONE),
    (28279543, MZONE),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix_n::<Materials>(duel, c);
    Fusion::add_contact_proc::<Contact>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, remcon);
    e1.set_target(duel, remtg);
    e1.set_operation(duel, remop);
    c.register_effect(duel, e1);
}

fn ffilter(c: CardId, duel: &Duel, fc: CardId, sumtype: u64, tp: u8, sg: Option<CardSet>) -> bool {
    let code = c.get_codes_full(duel, Some(fc), sumtype, tp).into_iter().min().unwrap_or(0);
    sg.is_none_or(|sg| !sg.is_exists(duel, |mc, duel| fusfilter(mc, duel, code, fc, tp), 1, Some(c.into())))
}

fn fusfilter(c: CardId, duel: &Duel, code: u32, fc: CardId, tp: u8) -> bool {
    c.is_summon_code(duel, Some(fc), SUMMON_TYPE_FUSION as u64, tp, &[code]) && c.is_has_effect(duel, 511002961).is_none()
}

fn matfilter(duel: &Duel, c: CardId, fc: CardId, tp: u8, contact: bool, sumtype: u64) -> bool {
    if sumtype & SUMMON_TYPE_FUSION as u64 != 0 && fc.is_location(duel, LOCATION_EXTRA) && !contact {
        return c.is_location(duel, LOCATION_ONFIELD | LOCATION_HAND) && c.is_controler(duel, tp);
    }
    true
}

fn filteraux(c: CardId, duel: &Duel) -> bool {
    c.is_able_to_remove_as_cost(duel) && c.is_monster(duel)
}

fn splimit(duel: &Duel, e: EffectId, _: Option<EffectId>, _: u8, st: u32, _: u8, _: u8) -> bool {
    e.get_handler(duel).get_location(duel) != LOCATION_EXTRA || st & SUMMON_TYPE_FUSION == SUMMON_TYPE_FUSION
}

fn remcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_summon_location(duel, LOCATION_EXTRA)
        && !c
            .get_material(duel)
            .is_exists(duel, |mc, duel| mc.get_original_race(duel) != RACE_DRAGON, 1, None)
}

fn remtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(
            duel.is_existing_matching_card(CardId::is_able_to_remove, tp, LOCATION_DECK, 0, 1, None)
                && duel.is_existing_matching_card(CardId::is_able_to_remove, tp, 0, LOCATION_DECK, 1, None)
                && duel.is_existing_matching_card(CardId::is_able_to_remove, tp, 0, LOCATION_EXTRA, 1, None),
        );
    }
    duel.set_operation_info(0, CATEGORY_REMOVE, None, 1, 0, (LOCATION_DECK | LOCATION_EXTRA) as i32);
    Ok(false)
}

fn remop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g1 = duel.get_matching_group(CardId::is_able_to_remove, tp, LOCATION_DECK, 0, None);
    let g2 = duel.get_decktop_group(1 - tp, 1);
    let g3 = duel.get_matching_group(CardId::is_able_to_remove, tp, 0, LOCATION_EXTRA, None);
    if !g1.is_empty() && !g2.is_empty() && !g3.is_empty() {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
        let mut sg1 = g1.select(duel, tp, 1, 1, None)?;
        duel.confirm_decktop(1 - tp, 1)?;
        duel.confirm_cards(tp, g3)?;
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
        let sg3 = g3.select(duel, tp, 1, 1, None)?;
        sg1.merge(g2);
        sg1.merge(sg3);
        duel.remove(sg1, POS_FACEUP, REASON_EFFECT)?;
        duel.shuffle_extra(1 - tp)?;
    }
    Ok(())
}
