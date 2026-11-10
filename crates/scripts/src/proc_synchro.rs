// Port of proc_synchro.lua: Synchro.AddProcedure and Synchro.AddMajesticProcedure with their conditions, targets and
// operation, and Synchro.AddHandMaterialEffect. The Dark Synchro procedure and Synchro.Send come with the cards that use them.
use engine::{Args, CardId, CardSet, ConditionFn, Duel, EffectId, Operation, Script, Target, TargetFn, Value};

use crate::constant::*;
use crate::utility::aux;

pub struct Synchro;

// The parameters of Synchro.AddProcedure (f1, min1, max1, f2, min2, max2, sub1, sub2, req1, req2, reqm): a filter left
// nil passes every card, a nil substitute none.
pub trait SynchroProcedure {
    const MIN1: usize = 1;
    const MAX1: usize = 1;
    const MIN2: usize;
    const MAX2: usize;
    const REQ1: Option<Requirement> = None;
    const REQ2: Option<Requirement> = None;
    const REQM: Option<Requirement> = None;

    fn f1(_c: CardId, _duel: &Duel, _sc: CardId, _sumtype: u64, _tp: u8) -> bool {
        true
    }

    fn f2(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool;

    fn sub1(_c: CardId, _duel: &Duel, _sc: CardId, _sumtype: u64, _tp: u8) -> bool {
        false
    }

    fn sub2(_c: CardId, _duel: &Duel, _sc: CardId, _sumtype: u64, _tp: u8) -> bool {
        false
    }
}

// The parameters of Synchro.AddMajesticProcedure (f1, cbt1, f2, cbt2, f3, cbt3, reqm...): the filter of each of the three
// materials in the order chosen, and whether it can be the Tuner.
pub trait MajesticProcedure {
    const CBT1: bool;
    const CBT2: bool;
    const CBT3: bool;
    const REQM: &'static [Requirement] = &[];

    fn f1(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool;

    fn f2(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool;

    fn f3(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool;
}

// The parameters of Synchro.AddHandMaterialEffect (id, material_filter, synchro_filter); no official script passes
// banish_mats or rc.
pub trait HandMaterial {
    const ID: u32;

    fn material_filter(_c: CardId, _duel: &Duel) -> bool {
        true
    }

    fn synchro_filter(_sc: CardId, _duel: &Duel) -> bool {
        true
    }
}

// req1, req2 or reqm: whether the chosen Tuners, non-Tuners or all materials (g, sc, tp) may be used.
pub type Requirement = fn(&Duel, CardSet, CardId, u8) -> bool;

fn req(requirement: Option<Requirement>, duel: &Duel, g: CardSet, sc: CardId, tp: u8) -> bool {
    requirement.is_none_or(|requirement| requirement(duel, g, sc, tp))
}

const SYNCHRO_MATERIAL: u64 = SUMMON_TYPE_SYNCHRO as u64 | MATERIAL_SYNCHRO;
const HAND_CHECK: u32 = EFFECT_HAND_SYNCHRO + EFFECT_SYNCHRO_CHECK;

// What the recursive checks share: the Synchro Monster's level, the monster and its controller, the materials it must
// use, whether only given materials may be used, and the forced minimum and maximum (0 for nil).
struct Search {
    lv: u32,
    sc: CardId,
    tp: u8,
    pg: CardSet,
    mgchk: bool,
    min: usize,
    max: usize,
    levels: Levels,
    plain: bool,
}

// The Synchro levels of up to 16 materials, read once per condition or target call instead of at every CheckP43: neither
// changes a level before its last read, and a repeated read looks up the same effects in the same state. Other cards are
// read when asked.
struct Levels([(Option<CardId>, u32); 16]);

impl Levels {
    fn new(duel: &Duel, g: CardSet, sc: CardId) -> Self {
        let mut levels = [(None, 0); 16];
        levels
            .iter_mut()
            .zip(g)
            .for_each(|(slot, c)| *slot = (Some(c), c.get_synchro_level(duel, sc)));
        Self(levels)
    }

    fn get(&self, duel: &Duel, c: CardId, sc: CardId) -> u32 {
        self.0
            .iter()
            .find(|(card, _)| *card == Some(c))
            .map_or_else(|| c.get_synchro_level(duel, sc), |&(_, level)| level)
    }
}

impl Synchro {
    // Synchro.NonTuner(f, ...): `filter` is f with its arguments.
    pub fn non_tuner(target: CardId, duel: &Duel, scard: CardId, tp: u8, filter: impl Fn(CardId, &Duel) -> bool) -> bool {
        target.is_not_tuner(duel, scard, tp) && filter(target, duel)
    }

    pub fn add_procedure<P: SynchroProcedure>(duel: &mut Duel, c: CardId) -> EffectId {
        Synchro::register_procedure(duel, c, Synchro::condition::<P>, Synchro::target::<P>)
    }

    pub fn add_majestic_procedure<P: MajesticProcedure>(duel: &mut Duel, c: CardId) -> EffectId {
        Synchro::register_procedure(duel, c, Synchro::majestic_condition::<P>, Synchro::majestic_target::<P>)
    }

    fn register_procedure(duel: &mut Duel, c: CardId, condition: ConditionFn, target: TargetFn) -> EffectId {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_FIELD);
        e1.set_description(duel, 1172);
        e1.set_code(duel, EFFECT_SPSUMMON_PROC);
        e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE + EFFECT_FLAG_IGNORE_IMMUNE);
        e1.set_range(duel, LOCATION_EXTRA);
        e1.set_condition(duel, condition);
        e1.set_target(duel, target);
        e1.set_operation(duel, Synchro::operation);
        e1.set_value(duel, SUMMON_TYPE_SYNCHRO as i32);
        c.register_effect(duel, e1);
        e1
    }

    // Synchro.AddHandMaterialEffect: while c is on the field and a Synchro material, a hand card that passes the filters
    // can be another.
    pub fn add_hand_material_effect<H: HandMaterial>(duel: &mut Duel, c: CardId) -> EffectId {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE + EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
        e1.set_code(duel, EFFECT_HAND_SYNCHRO);
        e1.set_range(duel, LOCATION_MZONE);
        e1.set_label(duel, H::ID.into());
        e1.set_value(duel, Value::HandSynchro(Synchro::hand_material_value::<H>));
        c.register_effect(duel, e1);
        e1
    }

    // synval of Synchro.CreateHandMaterialEffect.
    fn hand_material_value<H: HandMaterial>(duel: &mut Duel, e: EffectId, c: CardId, sc: CardId) -> bool {
        if !c.is_location(duel, LOCATION_HAND) || !H::material_filter(c, duel) || !H::synchro_filter(sc, duel) {
            return false;
        }
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, HAND_CHECK);
        e1.set_label(duel, H::ID.into());
        e1.set_synchro_check_target(duel, Synchro::hand_material_check::<H>);
        c.register_effect(duel, e1);
        true
    }

    // synchktg of Synchro.CreateHandMaterialEffect.
    fn hand_material_check<H: HandMaterial>(
        duel: &Duel,
        _e: EffectId,
        c: Option<CardId>,
        sg: CardSet,
        tg: CardSet,
        ntg: CardSet,
        _tsg: CardSet,
        _ntsg: CardSet,
    ) -> (bool, CardSet, CardSet) {
        let none = CardSet::default();
        let Some(c) = c else { return (true, none, none) };
        let id = i64::from(H::ID);
        let matfilter1 = |c: CardId, duel: &Duel| {
            let effs = c.get_card_effect(duel, HAND_CHECK);
            effs.iter().all(|&te| te.get_label(duel) == id) && !effs.is_empty()
        };
        let matfilter2 = |c: CardId, duel: &Duel| {
            c.is_has_effect(duel, HAND_CHECK).is_none()
                && c.get_card_effect(duel, EFFECT_HAND_SYNCHRO)
                    .iter()
                    .any(|&te| te.get_label(duel) == id)
        };
        let others = Some(c.into());
        if sg.is_exists(duel, matfilter1, 1, others)
            || !(tg.is_exists(duel, matfilter2, 1, others)
                || ntg.is_exists(duel, matfilter2, 1, others)
                || sg.is_exists(duel, matfilter2, 1, others))
        {
            return (false, none, none);
        }
        (true, tg.filter(duel, matfilter1, None), ntg.filter(duel, matfilter1, None))
    }

    // sub1 as the condition and target extend it: also a card that can be treated as a Tuner and passes f1.
    fn sub1<P: SynchroProcedure>(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        P::sub1(c, duel, sc, sumtype, tp)
            || (c.is_has_effect(duel, EFFECT_CAN_BE_TUNER).is_some() && P::f1(c, duel, sc, SYNCHRO_MATERIAL, tp))
    }

    fn tuner_filter<P: SynchroProcedure>(c: CardId, duel: &Duel, sc: CardId, tp: u8) -> bool {
        (c.is_type_full(duel, TYPE_TUNER, Some(sc), SYNCHRO_MATERIAL, tp) && P::f1(c, duel, sc, SYNCHRO_MATERIAL, tp))
            || Synchro::sub1::<P>(c, duel, sc, SYNCHRO_MATERIAL, tp)
    }

    fn non_tuner_filter<P: SynchroProcedure>(c: CardId, duel: &Duel, sc: CardId, tp: u8) -> bool {
        P::f2(c, duel, sc, SYNCHRO_MATERIAL, tp) || P::sub2(c, duel, sc, SYNCHRO_MATERIAL, tp)
    }

    // The materials both the condition and the target start from: (g, pg, mgchk) and their levels.
    fn materials(duel: &Duel, c: CardId, tp: u8, mg: Option<CardSet>) -> (CardSet, CardSet, bool, Levels) {
        let synchmatfilter = |mc: CardId, duel: &Duel| {
            let handmatfilter = mc.is_has_effect(duel, EFFECT_SYNCHRO_MAT_FROM_HAND);
            (mc.is_location(duel, LOCATION_MZONE)
                && mc.is_faceup(duel)
                && (mc.is_controler(duel, tp) || mc.is_can_be_synchro_material(duel, c)))
                || handmatfilter.is_some_and(|te| {
                    te.check_count_limit(duel, tp) && matches!(te.get_value(duel), Value::MaterialFor(value) if value(duel, te, mc, c))
                })
        };
        let dg = mg
            .unwrap_or_else(|| duel.get_matching_group(synchmatfilter, tp, LOCATION_MZONE | LOCATION_HAND, LOCATION_MZONE, Some(c.into())));
        Synchro::usable_materials(duel, c, tp, dg, mg.is_some())
    }

    // The materials of Synchro.MajesticCondition and Synchro.MajesticTarget, as [Synchro::materials] has them.
    fn majestic_materials(duel: &Duel, c: CardId, tp: u8, mg: Option<CardSet>) -> (CardSet, CardSet, bool, Levels) {
        let matfilter =
            |mc: CardId, duel: &Duel| mc.is_faceup(duel) && (mc.is_controler(duel, tp) || mc.is_can_be_synchro_material(duel, c));
        let dg = mg.unwrap_or_else(|| duel.get_matching_group(matfilter, tp, LOCATION_MZONE, LOCATION_MZONE, Some(c.into())));
        Synchro::usable_materials(duel, c, tp, dg, mg.is_some())
    }

    fn usable_materials(duel: &Duel, c: CardId, tp: u8, dg: CardSet, mgchk: bool) -> (CardSet, CardSet, bool, Levels) {
        let mut g = dg.filter(duel, |mc, duel| mc.is_can_be_synchro_material(duel, c), Some(c.into()));
        let levels = Levels::new(duel, g, c);
        g.remove_matching(duel, |mc, duel| levels.get(duel, mc, c) < 1, None);
        let pg = aux::get_must_be_material_group(duel, tp, Some(dg), tp, Some(c), Some(g), REASON_SYNCHRO);
        (g, pg, mgchk, levels)
    }

    // The condition's check that the materials it must use can be, and g and pg with smat.
    fn must_materials(duel: &Duel, c: CardId, g: CardSet, pg: CardSet, smat: Option<CardSet>) -> Option<(CardSet, CardSet)> {
        let cannot = |mc: CardId, duel: &Duel| !mc.is_can_be_synchro_material(duel, c);
        if !g.includes(pg) || pg.is_exists(duel, cannot, 1, None) || smat.is_some_and(|smat| smat.is_exists(duel, cannot, 1, None)) {
            return None;
        }
        let smat = smat.unwrap_or_default();
        Some((g.union(smat), pg.union(smat)))
    }

    // The hand materials the EFFECT_HAND_SYNCHRO effects of the cards of g add, unfiltered.
    fn hand_materials(duel: &mut Duel, c: CardId, tp: u8, g: CardSet) -> CardSet {
        let hg = Synchro::hand_group(duel, c, tp);
        let mut thg = CardSet::default();
        for thc in g {
            if let Some(&te) = thc.get_card_effect(duel, EFFECT_HAND_SYNCHRO).first() {
                thg.merge(Synchro::hand_synchro(duel, te, hg, c, |_, _| true));
            }
        }
        thg
    }

    // Whether an effect may have EFFECT_HAND_SYNCHRO, or EFFECT_HAND_SYNCHRO+EFFECT_SYNCHRO_CHECK (which a cancelled
    // Synchro.Target leaves on hand cards); without them a check registers and resets nothing.
    fn hand_synchro_possible(duel: &Duel) -> bool {
        duel.may_have_effect(EFFECT_HAND_SYNCHRO) || duel.may_have_effect(HAND_CHECK)
    }

    fn hand_group(duel: &Duel, c: CardId, tp: u8) -> CardSet {
        let can_be_material = |mc: CardId, duel: &Duel| mc.is_can_be_synchro_material(duel, c);
        duel.get_matching_group(can_be_material, tp, LOCATION_HAND + LOCATION_GRAVE, 0, Some(c.into()))
    }

    // The cards of hg that pass `filter` and value(te, mc, sc) of te, an EFFECT_HAND_SYNCHRO effect, which registers
    // EFFECT_HAND_SYNCHRO+EFFECT_SYNCHRO_CHECK on each hand material.
    fn hand_synchro(duel: &mut Duel, te: EffectId, hg: CardSet, sc: CardId, filter: impl Fn(CardId, &Duel) -> bool) -> CardSet {
        let Value::HandSynchro(val) = te.get_value(duel) else {
            return CardSet::default();
        };
        let mut ag = CardSet::default();
        for mc in hg {
            if filter(mc, duel) && val(duel, te, mc, sc) {
                ag.insert(mc);
            }
        }
        ag
    }

    // The cards of tg and ntg with EFFECT_HAND_SYNCHRO. Not in the Lua: none are looked for while no effect has the code.
    fn hand_synchro_cards(duel: &Duel, tg: CardSet, ntg: CardSet) -> CardSet {
        if !duel.may_have_effect(EFFECT_HAND_SYNCHRO) {
            return CardSet::default();
        }
        let has = |mc: CardId, duel: &Duel| mc.is_has_effect(duel, EFFECT_HAND_SYNCHRO).is_some();
        tg.filter(duel, has, None).union(ntg.filter(duel, has, None))
    }

    // tg and ntg with the hand materials the EFFECT_HAND_SYNCHRO effects of the cards of thg add.
    fn add_hand_materials<P: SynchroProcedure>(
        duel: &mut Duel,
        c: CardId,
        tp: u8,
        thg: CardSet,
        tg: CardSet,
        ntg: CardSet,
    ) -> (CardSet, CardSet) {
        let hg = Synchro::hand_group(duel, c, tp);
        let (mut tg, mut ntg) = (tg, ntg);
        for thc in thg {
            let Some(&te) = thc.get_card_effect(duel, EFFECT_HAND_SYNCHRO).first() else {
                continue;
            };
            let thag = Synchro::hand_synchro(duel, te, hg, c, |mc, duel| Synchro::tuner_filter::<P>(mc, duel, c, tp));
            let nthag = Synchro::hand_synchro(duel, te, hg, c, |mc, duel| Synchro::non_tuner_filter::<P>(mc, duel, c, tp));
            tg.merge(thag);
            ntg.merge(nthag);
        }
        (tg, ntg)
    }

    // aux.ResetEffects over the hand and GY cards with EFFECT_HAND_SYNCHRO+EFFECT_SYNCHRO_CHECK. Not in the Lua: the cards
    // are not searched while no effect has the code.
    fn reset_hand_synchro(duel: &mut Duel, tp: u8) {
        if !duel.may_have_effect(HAND_CHECK) {
            return;
        }
        let has = |c: CardId, duel: &Duel| c.is_has_effect(duel, HAND_CHECK).is_some();
        for c in duel.get_matching_group(has, tp, LOCATION_HAND + LOCATION_GRAVE, 0, None) {
            for te in c.get_card_effect(duel, HAND_CHECK) {
                te.reset(duel);
            }
        }
    }

    // Synchro.CheckFilterChk: a material with EFFECT_SYNCHRO_CHECK that passes some filter once the effect's value, which
    // may assume properties, ran.
    fn check_filter_chk<P: SynchroProcedure>(c: CardId, duel: &Duel, sc: CardId, tp: u8) -> bool {
        let Some(&te) = c.get_card_effect(duel, EFFECT_SYNCHRO_CHECK).first() else {
            return false;
        };
        let reset = Synchro::synchro_check(duel, te, c);
        let res = (c.is_type_full(duel, TYPE_TUNER, Some(sc), SYNCHRO_MATERIAL, tp) && P::f1(c, duel, sc, SYNCHRO_MATERIAL, tp))
            || P::f2(c, duel, sc, SYNCHRO_MATERIAL, tp)
            || Synchro::sub1::<P>(c, duel, sc, SYNCHRO_MATERIAL, tp)
            || P::sub2(c, duel, sc, SYNCHRO_MATERIAL, tp);
        if reset {
            duel.assume_reset();
        }
        res
    }

    // Not in the Lua: the cards are not searched while no effect has the code.
    fn any_synchro_check<P: SynchroProcedure>(duel: &Duel, g: CardSet, sc: CardId, tp: u8) -> bool {
        duel.may_have_effect(EFFECT_SYNCHRO_CHECK) && g.is_exists(duel, |c, duel| Synchro::check_filter_chk::<P>(c, duel, sc, tp), 1, None)
    }

    // value(e, c) of EFFECT_SYNCHRO_CHECK.
    fn synchro_check(duel: &Duel, te: EffectId, c: CardId) -> bool {
        matches!(te.get_value(duel), Value::Card(val) if val(duel, te, c) != 0)
    }

    // The end of CheckP31, CheckP32 and the target's deselection: the assumptions stay while a chosen material has
    // EFFECT_SYNCHRO_CHECK.
    fn assume_reset_unless_checked(duel: &Duel, sg: CardSet) {
        if !sg.is_exists(duel, |c, duel| c.is_has_effect(duel, EFFECT_SYNCHRO_CHECK).is_some(), 1, None) {
            duel.assume_reset();
        }
    }

    fn condition<P: SynchroProcedure>(duel: &Duel, a: &Args) -> bool {
        let (smat, mg) = (a.must, a.materials);
        let Some(c) = a.c else { return true };
        if c.is_type(duel, TYPE_PENDULUM) && c.is_faceup(duel) {
            return false;
        }
        let tp = c.get_controler(duel);
        let (g, pg, mgchk, levels) = Synchro::materials(duel, c, tp, mg);
        let Some((g, pg)) = Synchro::must_materials(duel, c, g, pg, smat) else {
            return false;
        };
        let mut s = Search {
            lv: c.get_level(duel),
            sc: c,
            tp,
            pg,
            mgchk,
            min: usize::from(a.min),
            max: usize::from(a.max),
            levels,
            plain: false,
        };
        let none = CardSet::default();
        let hand_synchro = Synchro::hand_synchro_possible(duel);
        if Synchro::any_synchro_check::<P>(duel, g, c, tp) {
            let search = |duel: &Duel, g2: CardSet| {
                g2.is_exists(
                    duel,
                    |mc, duel| Synchro::check_p31::<P>(duel, mc, g2, none, none, none, &s),
                    1,
                    None,
                )
            };
            let res = if hand_synchro {
                duel.check_mut(|duel| {
                    let g2 = if mgchk {
                        g
                    } else {
                        g.union(Synchro::hand_materials(duel, c, tp, g))
                    };
                    let res = search(duel, g2);
                    Synchro::reset_hand_synchro(duel, tp);
                    res
                })
            } else {
                search(duel, g)
            };
            duel.assume_reset();
            return res;
        }
        let tg = g.filter(duel, |mc, duel| Synchro::tuner_filter::<P>(mc, duel, c, tp), None);
        let ntg = g.filter(duel, |mc, duel| Synchro::non_tuner_filter::<P>(mc, duel, c, tp), None);
        let search = |duel: &Duel, tg: CardSet, ntg: CardSet, s: &mut Search| {
            s.plain = Synchro::is_plain::<P>(duel);
            Synchro::level_reachable::<P>(duel, tg.union(ntg), s)
                && tg.is_exists(
                    duel,
                    |tc, duel| Synchro::check_p41::<P>(duel, tc, tg, ntg, none, none, none, s),
                    1,
                    None,
                )
        };
        let thg = if mgchk { none } else { Synchro::hand_synchro_cards(duel, tg, ntg) };
        // Not in the Lua: the copy of the duel the Lua needs to register and reset effects is made only when it does.
        if thg.is_empty() && !duel.may_have_effect(HAND_CHECK) {
            return search(duel, tg, ntg, &mut s);
        }
        duel.check_mut(|duel| {
            let (tg, ntg) = if mgchk {
                (tg, ntg)
            } else {
                Synchro::add_hand_materials::<P>(duel, c, tp, thg, tg, ntg)
            };
            let res = search(duel, tg, ntg, &mut s);
            Synchro::reset_hand_synchro(duel, tp);
            res
        })
    }

    // Not in the Lua: CheckP43 needs all of a selection to add up to the level and more materials only raise the sum, so
    // nothing is found from materials that cannot reach the level or from a selection already past it. The search is cut
    // short only when what it skips looks up no effect, since is_available numbers an effect that becomes available: no
    // requirements, and no effect of the codes CheckP41 to CheckP43 look up.
    fn is_plain<P: SynchroProcedure>(duel: &Duel) -> bool {
        let codes = [EFFECT_SYNCHRO_MAT_RESTRICTION, HAND_CHECK, EFFECT_SYNCHRO_MATERIAL_CUSTOM];
        P::REQ1.is_none() && P::REQ2.is_none() && P::REQM.is_none() && !codes.iter().any(|&code| duel.may_have_effect(code))
    }

    fn level_reachable<P: SynchroProcedure>(duel: &Duel, g: CardSet, s: &Search) -> bool {
        !s.plain || g.check_with_sum_equal(duel, |c, duel| s.levels.get(duel, c, s.sc), s.lv, P::MIN1 + P::MIN2, g.len())
    }

    fn level_exceeded(duel: &Duel, sg: CardSet, s: &Search) -> bool {
        let least = |level: u32| match level >> 16 {
            0 => level,
            high => high.min(level & 0xffff),
        };
        s.plain && sg.iter().map(|c| least(s.levels.get(duel, c, s.sc))).sum::<u32>() > s.lv
    }

    // The restrictions on adding c to sg, false if one rules it out, else the cards they rule out (trg, ntrg): those of
    // rtg and rntg that the EFFECT_SYNCHRO_MAT_RESTRICTION effects of c reject, and those the first passing
    // EFFECT_HAND_SYNCHRO+EFFECT_SYNCHRO_CHECK check of c returns for ctg and ntg. Not in the Lua: the lookups are
    // skipped while no effect has the code, as they would find nothing.
    fn restrictions(
        duel: &Duel,
        c: CardId,
        sg: CardSet,
        [rtg, rntg]: [CardSet; 2],
        [ctg, cntg]: [CardSet; 2],
        tsg: CardSet,
        ntsg: CardSet,
        mgchk: bool,
    ) -> Option<(CardSet, CardSet)> {
        // Auxiliary.HarmonizingMagFilter(c, f, f:GetValue()).
        let rejects = |f: EffectId| move |c: CardId, duel: &Duel| matches!(f.get_value(duel), Value::Card(value) if value(duel, f, c) == 0);
        let (mut trg, mut ntrg) = (CardSet::default(), CardSet::default());
        if duel.may_have_effect(EFFECT_SYNCHRO_MAT_RESTRICTION) {
            for f in c.get_card_effect(duel, EFFECT_SYNCHRO_MAT_RESTRICTION) {
                if sg.is_exists(duel, rejects(f), 1, Some(c.into())) {
                    return None;
                }
                trg.merge(rtg.filter(duel, rejects(f), None));
                ntrg.merge(rntg.filter(duel, rejects(f), None));
            }
            for tc in sg {
                if tc
                    .get_card_effect(duel, EFFECT_SYNCHRO_MAT_RESTRICTION)
                    .iter()
                    .any(|&f| rejects(f)(c, duel))
                {
                    return None;
                }
            }
        }
        if !mgchk && duel.may_have_effect(HAND_CHECK) {
            let check = |card: CardId, c: Option<CardId>| {
                let mut result = Some((CardSet::default(), CardSet::default()));
                for te in card.get_card_effect(duel, HAND_CHECK) {
                    let Target::SynchroCheck(check) = te.get_target(duel) else {
                        continue;
                    };
                    let (ok, trg, ntrg) = check(duel, te, c, sg, ctg, cntg, tsg, ntsg);
                    result = ok.then_some((trg, ntrg));
                    if ok {
                        break;
                    }
                }
                result
            };
            let (trg2, ntrg2) = check(c, Some(c))?;
            trg.merge(trg2);
            ntrg.merge(ntrg2);
            if sg.iter().any(|tc| check(tc, None).is_none()) {
                return None;
            }
        }
        Some((trg, ntrg))
    }

    fn check_p31<P: SynchroProcedure>(duel: &Duel, c: CardId, g: CardSet, tsg: CardSet, ntsg: CardSet, sg: CardSet, s: &Search) -> bool {
        let (sc, tp) = (s.sc, s.tp);
        let mut rg = CardSet::default();
        for te in c.get_card_effect(duel, EFFECT_SYNCHRO_CHECK) {
            let tg = g.filter(duel, |mc, duel| Synchro::synchro_check(duel, te, mc), None);
            rg = tg.filter(
                duel,
                |mc, duel| !Synchro::tuner_filter::<P>(mc, duel, sc, tp) && !Synchro::non_tuner_filter::<P>(mc, duel, sc, tp),
                None,
            );
        }
        let none = CardSet::default();
        let Some((trg, _)) = Synchro::restrictions(duel, c, sg, [g, none], [g, g], tsg, ntsg, s.mgchk) else {
            return false;
        };
        rg.merge(trg);
        let (g, tsg, sg) = (g.difference(rg), tsg.union(c.into()), sg.union(c.into()));
        let tsg_count = tsg.len();
        let tuners = |duel: &Duel| {
            tsg.is_exists(duel, |tc, duel| Synchro::tuner_filter::<P>(tc, duel, sc, tp), tsg_count, None) && req(P::REQ1, duel, tsg, sc, tp)
        };
        let p31 = |tc, duel: &Duel| Synchro::check_p31::<P>(duel, tc, g, tsg, ntsg, sg, s);
        let p32 = |tc, duel: &Duel| Synchro::check_p32::<P>(duel, tc, g, tsg, ntsg, sg, s);
        let res = if s.max != 0 && (tsg_count > s.max || s.max - tsg_count < P::MIN2) {
            false
        } else if s.max != 0 && s.max - tsg_count == P::MIN2 {
            tuners(duel) && g.is_exists(duel, p32, 1, Some(sg))
        } else if tsg_count < P::MIN1 {
            g.is_exists(duel, p31, 1, Some(sg))
        } else if tsg_count < P::MAX1 {
            g.is_exists(duel, p31, 1, Some(sg)) || (tuners(duel) && g.is_exists(duel, p32, 1, Some(sg)))
        } else {
            tuners(duel) && g.is_exists(duel, p32, 1, Some(sg))
        };
        Synchro::assume_reset_unless_checked(duel, sg.difference(c.into()));
        res
    }

    fn check_p32<P: SynchroProcedure>(duel: &Duel, c: CardId, g: CardSet, tsg: CardSet, ntsg: CardSet, sg: CardSet, s: &Search) -> bool {
        let (sc, tp) = (s.sc, s.tp);
        let mut rg = CardSet::default();
        for te in c.get_card_effect(duel, EFFECT_SYNCHRO_CHECK) {
            let tg = g.filter(duel, |mc, duel| Synchro::synchro_check(duel, te, mc), None);
            rg = tg.filter(duel, |mc, duel| !Synchro::non_tuner_filter::<P>(mc, duel, sc, tp), None);
        }
        let none = CardSet::default();
        let Some((_, ntrg)) = Synchro::restrictions(duel, c, sg, [none, g], [none, g], tsg, ntsg, s.mgchk) else {
            return false;
        };
        rg.merge(ntrg);
        let (g, ntsg, sg) = (g.difference(rg), ntsg.union(c.into()), sg.union(c.into()));
        let (tsg_count, ntsg_count) = (tsg.len(), ntsg.len());
        let p32 = |tc, duel: &Duel| Synchro::check_p32::<P>(duel, tc, g, tsg, ntsg, sg, s);
        let complete = || {
            (s.min == 0 || tsg_count + ntsg_count >= s.min)
                && req(P::REQ2, duel, ntsg, sc, tp)
                && req(P::REQM, duel, sg, sc, tp)
                && ntsg.is_exists(duel, |tc, duel| Synchro::non_tuner_filter::<P>(tc, duel, sc, tp), ntsg_count, None)
                && sg.includes(s.pg)
                && Synchro::check_p43(duel, tsg, ntsg, sg, s)
        };
        let res = if s.max != 0 && tsg_count + ntsg_count > s.max {
            false
        } else if ntsg_count < P::MIN2 {
            g.is_exists(duel, p32, 1, Some(sg))
        } else if ntsg_count < P::MAX2 {
            g.is_exists(duel, p32, 1, Some(sg)) || complete()
        } else {
            complete()
        };
        Synchro::assume_reset_unless_checked(duel, sg.difference(c.into()));
        res
    }

    fn check_p41<P: SynchroProcedure>(
        duel: &Duel,
        c: CardId,
        tg: CardSet,
        ntg: CardSet,
        tsg: CardSet,
        ntsg: CardSet,
        sg: CardSet,
        s: &Search,
    ) -> bool {
        let Some((trg, ntrg)) = Synchro::restrictions(duel, c, sg, [tg, ntg], [tg, ntg], tsg, ntsg, s.mgchk) else {
            return false;
        };
        let (tg, ntg) = (tg.difference(trg), ntg.difference(ntrg));
        let (tsg, sg) = (tsg.union(c.into()), sg.union(c.into()));
        let tsg_count = tsg.len();
        let p41 = |tc, duel: &Duel| Synchro::check_p41::<P>(duel, tc, tg, ntg, tsg, ntsg, sg, s);
        let p42 = |tc, duel: &Duel| Synchro::check_p42::<P>(duel, tc, ntg, tsg, ntsg, sg, s);
        if s.max != 0 && (tsg_count > s.max || s.max - tsg_count < P::MIN2) {
            false
        } else if s.max != 0 && s.max - tsg_count == P::MIN2 {
            req(P::REQ1, duel, tsg, s.sc, s.tp) && ntg.is_exists(duel, p42, 1, Some(sg))
        } else if tsg_count < P::MIN1 {
            tg.is_exists(duel, p41, 1, Some(sg))
        } else if tsg_count < P::MAX1 {
            tg.is_exists(duel, p41, 1, Some(sg)) || (req(P::REQ1, duel, tsg, s.sc, s.tp) && ntg.is_exists(duel, p42, 1, Some(sg)))
        } else {
            req(P::REQ1, duel, tsg, s.sc, s.tp) && ntg.is_exists(duel, p42, 1, Some(sg))
        }
    }

    fn check_p42<P: SynchroProcedure>(duel: &Duel, c: CardId, ntg: CardSet, tsg: CardSet, ntsg: CardSet, sg: CardSet, s: &Search) -> bool {
        let none = CardSet::default();
        let Some((_, ntrg)) = Synchro::restrictions(duel, c, sg, [none, ntg], [none, ntg], tsg, ntsg, s.mgchk) else {
            return false;
        };
        let ntg = ntg.difference(ntrg);
        let (ntsg, sg) = (ntsg.union(c.into()), sg.union(c.into()));
        if Synchro::level_exceeded(duel, sg, s) {
            return false;
        }
        let (tsg_count, ntsg_count) = (tsg.len(), ntsg.len());
        let p42 = |tc, duel: &Duel| Synchro::check_p42::<P>(duel, tc, ntg, tsg, ntsg, sg, s);
        let complete = || {
            (s.min == 0 || tsg_count + ntsg_count >= s.min)
                && req(P::REQ2, duel, ntsg, s.sc, s.tp)
                && req(P::REQM, duel, sg, s.sc, s.tp)
                && sg.includes(s.pg)
                && Synchro::check_p43(duel, tsg, ntsg, sg, s)
        };
        if s.max != 0 && tsg_count + ntsg_count > s.max {
            false
        } else if ntsg_count < P::MIN2 {
            ntg.is_exists(duel, p42, 1, Some(sg))
        } else if ntsg_count < P::MAX2 {
            ntg.is_exists(duel, p42, 1, Some(sg)) || complete()
        } else {
            complete()
        }
    }

    fn check_p43(duel: &Duel, tsg: CardSet, ntsg: CardSet, sg: CardSet, s: &Search) -> bool {
        !Synchro::hand_unmatched(duel, sg) && Synchro::level_matches(duel, tsg, ntsg, sg, s)
    }

    // Synchro.CheckHand over sg: a hand material whose EFFECT_HAND_SYNCHRO card (by label) is not among the materials.
    fn hand_unmatched(duel: &Duel, sg: CardSet) -> bool {
        let check_hand = |c: CardId, duel: &Duel| {
            let teg = c.get_card_effect(duel, HAND_CHECK);
            let check_label = |tc: CardId, duel: &Duel, label: i64| {
                tc.get_card_effect(duel, EFFECT_HAND_SYNCHRO)
                    .first()
                    .is_some_and(|&eff| eff.get_label(duel) == label)
            };
            !teg.is_empty()
                && teg
                    .iter()
                    .all(|&te| !sg.is_exists(duel, |tc, duel| check_label(tc, duel, te.get_label(duel)), 1, Some(c.into())))
        };
        duel.may_have_effect(HAND_CHECK) && sg.is_exists(duel, check_hand, 1, None)
    }

    // The rest of CheckP43: the EFFECT_SYNCHRO_MATERIAL_CUSTOM checks, the level sum and a free zone.
    fn level_matches(duel: &Duel, tsg: CardSet, ntsg: CardSet, sg: CardSet, s: &Search) -> bool {
        let (lv, sc, tp) = (s.lv, s.sc, s.tp);
        let mut lvchk = false;
        let custom = if duel.may_have_effect(EFFECT_SYNCHRO_MATERIAL_CUSTOM) {
            sg
        } else {
            CardSet::default()
        };
        for tc in custom {
            for te in tc.get_card_effect(duel, EFFECT_SYNCHRO_MATERIAL_CUSTOM) {
                let Some(Operation::SynchroMaterial(op)) = te.get_operation(duel) else {
                    continue;
                };
                let (ok, tlvchk) = op(duel, te, tsg, ntsg, sg, lv, sc, tp);
                if !ok {
                    return false;
                }
                lvchk |= tlvchk;
            }
        }
        (lvchk || sg.check_with_sum_equal(duel, |c, duel| s.levels.get(duel, c, sc), lv, sg.len(), sg.len()))
            && ((sc.is_location(duel, LOCATION_EXTRA) && duel.get_location_count_from_ex(tp, tp, Some(sg), sc) > 0)
                || (!sc.is_location(duel, LOCATION_EXTRA) && duel.get_mzone_count(tp, Some(sg)) > 0))
    }

    // Runs value(e, mc) of every EFFECT_SYNCHRO_CHECK of a chosen material for every card of g.
    fn synchro_check_all(duel: &Duel, tc: CardId, g: CardSet) {
        for te in tc.get_card_effect(duel, EFFECT_SYNCHRO_CHECK) {
            for mc in g {
                Synchro::synchro_check(duel, te, mc);
            }
        }
    }

    fn target<P: SynchroProcedure>(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
        let (e, tp, smat, mg) = (a.e, a.tp, a.must, a.materials);
        let Some(c) = a.c else { return Ok(false) };
        let mut sg = CardSet::default();
        let (mut g, mut pg, mgchk, levels) = Synchro::materials(duel, c, tp, mg);
        if let Some(smat) = smat {
            pg.merge(smat);
            g.merge(smat);
        }
        let tg = g.filter(duel, |mc, duel| Synchro::tuner_filter::<P>(mc, duel, c, tp), None);
        let ntg = g.filter(duel, |mc, duel| Synchro::non_tuner_filter::<P>(mc, duel, c, tp), None);
        let (tg, ntg) = if mgchk {
            (tg, ntg)
        } else {
            let thg = Synchro::hand_synchro_cards(duel, tg, ntg);
            Synchro::add_hand_materials::<P>(duel, c, tp, thg, tg, ntg)
        };
        let s = Search {
            lv: c.get_level(duel),
            sc: c,
            tp,
            pg,
            mgchk,
            min: usize::from(a.min),
            max: usize::from(a.max),
            levels,
            plain: Synchro::is_plain::<P>(duel),
        };
        let mut tsg = CardSet::default();
        let mut selectedastuner = CardSet::default();
        let mut ntsg = CardSet::default();
        let mut tune = true;
        if Synchro::any_synchro_check::<P>(duel, g, c, tp) {
            let tuners = |duel: &Duel, tsg: CardSet| {
                tsg.is_exists(duel, |tc, duel| Synchro::tuner_filter::<P>(tc, duel, c, tp), tsg.len(), None)
                    && req(P::REQ1, duel, tsg, c, tp)
            };
            while ntsg.len() < P::MAX2 {
                let p31 = |tc, duel: &Duel| Synchro::check_p31::<P>(duel, tc, g, tsg, ntsg, sg, &s);
                let p32 = |tc, duel: &Duel| Synchro::check_p32::<P>(duel, tc, g, tsg, ntsg, sg, &s);
                if tune {
                    let cancel = !mgchk && duel.is_summon_cancelable() && tsg.is_empty();
                    let g3 = ntg.filter(duel, p32, Some(sg));
                    let mut g2 = g.filter(duel, p31, Some(sg));
                    if !g3.is_empty() && tsg.len() >= P::MIN1 && tuners(duel, tsg) {
                        g2.merge(g3);
                    }
                    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SMATERIAL);
                    // Cancelling is offered only before a Tuner is chosen, when the Lua cannot go on with a nil card.
                    let Some(tc) = g2.select_unselect(duel, Some(sg), tp, false, cancel, 1, 1)? else {
                        return Ok(false);
                    };
                    if !sg.is_contains(tc) {
                        if g3.is_contains(tc) {
                            ntsg.insert(tc);
                            tune = false;
                        } else {
                            tsg.insert(tc);
                        }
                        selectedastuner.insert(tc);
                        sg.insert(tc);
                        Synchro::synchro_check_all(duel, tc, g);
                    } else {
                        selectedastuner.remove(tc);
                        tsg.remove(tc);
                        sg.remove(tc);
                        Synchro::assume_reset_unless_checked(duel, sg);
                    }
                    let p31 = |tc, duel: &Duel| Synchro::check_p31::<P>(duel, tc, g, tsg, ntsg, sg, &s);
                    if g.filter_count(duel, p31, Some(sg)) == 0 || tsg.len() >= P::MAX1 {
                        tune = false;
                    }
                } else {
                    let complete = |duel: &Duel| {
                        ntsg.len() >= P::MIN2
                            && req(P::REQ2, duel, ntsg, c, tp)
                            && req(P::REQM, duel, sg, c, tp)
                            && sg.includes(pg)
                            && Synchro::check_p43(duel, tsg, ntsg, sg, &s)
                    };
                    let finish = ntsg.len() >= P::MIN2
                        && req(P::REQ2, duel, ntsg, c, tp)
                        && req(P::REQM, duel, sg, c, tp)
                        && ntsg.is_exists(duel, |tc, duel| Synchro::non_tuner_filter::<P>(tc, duel, c, tp), ntsg.len(), None)
                        && sg.includes(pg)
                        && Synchro::check_p43(duel, tsg, ntsg, sg, &s);
                    let cancel = !mgchk && duel.is_summon_cancelable() && sg.is_empty();
                    let mut g2 = g.filter(duel, p32, Some(sg));
                    if g2.is_empty() {
                        break;
                    }
                    let g3 = g.filter(duel, p31, Some(sg));
                    if !g3.is_empty() && ntsg.difference(selectedastuner).is_empty() && tsg.len() < P::MAX1 {
                        g2.merge(g3);
                    }
                    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SMATERIAL);
                    let Some(tc) = g2.select_unselect(duel, Some(sg), tp, finish, cancel, 1, 1)? else {
                        if complete(duel) {
                            break;
                        }
                        return Ok(false);
                    };
                    if !selectedastuner.is_contains(tc) {
                        if !sg.is_contains(tc) {
                            ntsg.insert(tc);
                            sg.insert(tc);
                            Synchro::synchro_check_all(duel, tc, g);
                        } else {
                            ntsg.remove(tc);
                            sg.remove(tc);
                            Synchro::assume_reset_unless_checked(duel, sg);
                        }
                    } else if ntsg.difference(selectedastuner).is_empty() {
                        tune = true;
                        selectedastuner.remove(tc);
                        ntsg.remove(tc);
                        tsg.remove(tc);
                        sg.remove(tc);
                    }
                }
            }
            duel.assume_reset();
        } else {
            while ntsg.len() < P::MAX2 {
                let p41 = |tc, duel: &Duel| Synchro::check_p41::<P>(duel, tc, tg, ntg, tsg, ntsg, sg, &s);
                let p42 = |tc, duel: &Duel| Synchro::check_p42::<P>(duel, tc, ntg, tsg, ntsg, sg, &s);
                if tune {
                    let cancel = !mgchk && duel.is_summon_cancelable() && tsg.is_empty();
                    let g3 = ntg.filter(duel, p42, Some(sg));
                    let mut g2 = tg.filter(duel, p41, Some(sg));
                    if !g3.is_empty() && tsg.len() >= P::MIN1 && req(P::REQ1, duel, tsg, c, tp) {
                        g2.merge(g3);
                    }
                    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SMATERIAL);
                    match g2.select_unselect(duel, Some(sg), tp, false, cancel, 1, 1)? {
                        None => {
                            if tsg.len() >= P::MIN1 && req(P::REQ1, duel, tsg, c, tp) && ntg.filter_count(duel, p42, Some(sg)) > 0 {
                                tune = false;
                            } else {
                                return Ok(false);
                            }
                        }
                        Some(tc) if !sg.is_contains(tc) => {
                            if g3.is_contains(tc) {
                                ntsg.insert(tc);
                                tune = false;
                            } else {
                                tsg.insert(tc);
                            }
                            selectedastuner.insert(tc);
                            sg.insert(tc);
                        }
                        Some(tc) => {
                            selectedastuner.remove(tc);
                            tsg.remove(tc);
                            sg.remove(tc);
                        }
                    }
                    let p41 = |tc, duel: &Duel| Synchro::check_p41::<P>(duel, tc, tg, ntg, tsg, ntsg, sg, &s);
                    if tg.filter_count(duel, p41, Some(sg)) == 0 || tsg.len() >= P::MAX1 {
                        tune = false;
                    }
                } else {
                    let complete = |duel: &Duel| {
                        ntsg.len() >= P::MIN2
                            && req(P::REQ2, duel, ntsg, c, tp)
                            && req(P::REQM, duel, sg, c, tp)
                            && sg.includes(pg)
                            && Synchro::check_p43(duel, tsg, ntsg, sg, &s)
                    };
                    let finish = complete(duel);
                    let cancel = !mgchk && duel.is_summon_cancelable() && sg.is_empty();
                    let mut g2 = ntg.filter(duel, p42, Some(sg));
                    if g2.is_empty() {
                        break;
                    }
                    let g3 = tg.filter(duel, p41, Some(sg));
                    if !g3.is_empty() && ntsg.difference(selectedastuner).is_empty() && tsg.len() < P::MAX1 {
                        g2.merge(g3);
                    }
                    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SMATERIAL);
                    let Some(tc) = g2.select_unselect(duel, Some(sg), tp, finish, cancel, 1, 1)? else {
                        if complete(duel) {
                            break;
                        }
                        return Ok(false);
                    };
                    if !selectedastuner.is_contains(tc) {
                        if !sg.is_contains(tc) {
                            ntsg.insert(tc);
                            sg.insert(tc);
                        } else {
                            ntsg.remove(tc);
                            sg.remove(tc);
                        }
                    } else if ntsg.difference(selectedastuner).is_empty() {
                        tune = true;
                        selectedastuner.remove(tc);
                        ntsg.remove(tc);
                        tsg.remove(tc);
                        sg.remove(tc);
                    }
                }
            }
        }
        Synchro::reset_hand_synchro(duel, tp);
        let subtsg = tsg.filter(
            duel,
            |sc, duel| {
                Synchro::sub1::<P>(sc, duel, c, SYNCHRO_MATERIAL, tp)
                    && (!P::f1(sc, duel, c, SYNCHRO_MATERIAL, tp) || !sc.is_type(duel, TYPE_TUNER))
            },
            None,
        );
        for subc in subtsg {
            let e1 = duel.create_effect(c);
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_code(duel, EFFECT_ADD_TYPE);
            e1.set_value(duel, TYPE_TUNER as i32);
            e1.set_reset(duel, RESET_EVENT + RESETS_STANDARD);
            subc.register_effect_full(duel, e1, true);
        }
        e.set_label_object_group(duel, Some(sg));
        Ok(true)
    }

    // Synchro.MajesticCheck1: whether c can be chosen after the materials of `cards`, in their order.
    fn majestic_check1<P: MajesticProcedure>(
        duel: &Duel,
        c: CardId,
        g: CardSet,
        sg: CardSet,
        cards: [Option<CardId>; 3],
        s: &Search,
    ) -> bool {
        Synchro::synchro_check_all(duel, c, g);
        let Some((rg, _)) = Synchro::restrictions(duel, c, sg, [g, CardSet::default()], [g, g], sg, sg, false) else {
            return false;
        };
        let (g, sg, cards) = (g.difference(rg), sg.union(c.into()), Synchro::chosen(cards, c));
        let res = if sg.len() < 3 {
            g.is_exists(
                duel,
                |mc, duel| Synchro::majestic_check1::<P>(duel, mc, g, sg, cards, s),
                1,
                Some(sg),
            )
        } else {
            sg.includes(s.pg) && Synchro::majestic_check2::<P>(duel, sg, cards, s)
        };
        Synchro::assume_reset_unless_checked(duel, sg.difference(c.into()));
        res
    }

    fn majestic_check2<P: MajesticProcedure>(duel: &Duel, sg: CardSet, cards: [Option<CardId>; 3], s: &Search) -> bool {
        let (sc, tp) = (s.sc, s.tp);
        let [Some(card1), Some(card2), Some(card3)] = cards else {
            return false;
        };
        if Synchro::hand_unmatched(duel, sg)
            || !P::f1(card1, duel, sc, SYNCHRO_MATERIAL, tp)
            || !P::f2(card2, duel, sc, SYNCHRO_MATERIAL, tp)
            || !P::f3(card3, duel, sc, SYNCHRO_MATERIAL, tp)
        {
            return false;
        }
        let tuner = |cbt: bool, card: CardId| cbt && card.is_type_full(duel, TYPE_TUNER, Some(sc), SYNCHRO_MATERIAL, tp);
        let tunechk = [tuner(P::CBT1, card1), tuner(P::CBT2, card2), tuner(P::CBT3, card3)];
        tunechk.contains(&true)
            && P::REQM.iter().all(|reqmat| reqmat(duel, sg, sc, tp))
            && Synchro::level_matches(duel, CardSet::default(), CardSet::default(), sg, s)
    }

    fn chosen(mut cards: [Option<CardId>; 3], c: CardId) -> [Option<CardId>; 3] {
        if let Some(slot) = cards.iter_mut().find(|slot| slot.is_none()) {
            *slot = Some(c);
        }
        cards
    }

    fn majestic_search(a: &Args, lv: u32, sc: CardId, tp: u8, pg: CardSet, mgchk: bool, levels: Levels) -> Search {
        Search {
            lv,
            sc,
            tp,
            pg,
            mgchk,
            min: usize::from(a.min),
            max: usize::from(a.max),
            levels,
            plain: false,
        }
    }

    fn majestic_condition<P: MajesticProcedure>(duel: &Duel, a: &Args) -> bool {
        let Some(c) = a.c else { return true };
        if a.min > 3 || (a.max != 0 && a.max < 3) {
            return false;
        }
        let (tp, lv) = (c.get_controler(duel), c.get_level(duel));
        let (g, pg, mgchk, levels) = Synchro::majestic_materials(duel, c, tp, a.materials);
        let Some((g, pg)) = Synchro::must_materials(duel, c, g, pg, a.must) else {
            return false;
        };
        let s = Synchro::majestic_search(a, lv, c, tp, pg, mgchk, levels);
        let none = CardSet::default();
        let search = |duel: &Duel, g: CardSet| {
            g.is_exists(
                duel,
                |mc, duel| Synchro::majestic_check1::<P>(duel, mc, g, none, [None; 3], &s),
                1,
                None,
            )
        };
        let res = if Synchro::hand_synchro_possible(duel) {
            duel.check_mut(|duel| {
                let g = if mgchk {
                    g
                } else {
                    g.union(Synchro::hand_materials(duel, c, tp, g))
                };
                let res = search(duel, g);
                Synchro::reset_hand_synchro(duel, tp);
                res
            })
        } else {
            search(duel, g)
        };
        duel.assume_reset();
        res
    }

    fn majestic_target<P: MajesticProcedure>(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
        let (e, tp) = (a.e, a.tp);
        let Some(c) = a.c else { return Ok(false) };
        if a.min > 3 || (a.max != 0 && a.max < 3) {
            return Ok(false);
        }
        // The Lua reads the level again once it has the materials.
        c.get_level(duel);
        let (mut g, mut pg, mgchk, levels) = Synchro::majestic_materials(duel, c, tp, a.materials);
        if let Some(smat) = a.must {
            pg.merge(smat);
            g.merge(smat);
        }
        if !mgchk {
            g.merge(Synchro::hand_materials(duel, c, tp, g));
        }
        let s = Synchro::majestic_search(a, c.get_level(duel), c, tp, pg, mgchk, levels);
        let mut sg = CardSet::default();
        let mut cards = [None; 3];
        let cancel = !mgchk && duel.is_summon_cancelable();
        while sg.len() < 3 {
            let g2 = g.filter(duel, |mc, duel| Synchro::majestic_check1::<P>(duel, mc, g, sg, cards, &s), Some(sg));
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_SMATERIAL);
            let Some(tc) = g2.select_unselect(duel, Some(sg), tp, false, cancel, 1, 1)? else {
                return Ok(false);
            };
            if !sg.is_contains(tc) {
                sg.insert(tc);
                Synchro::synchro_check_all(duel, tc, g);
                cards = Synchro::chosen(cards, tc);
            } else if let Some(last) = cards.iter_mut().rev().find(|slot| slot.is_some())
                && *last == Some(tc)
            {
                *last = None;
                sg.remove(tc);
                Synchro::assume_reset_unless_checked(duel, sg);
            }
        }
        duel.assume_reset();
        Synchro::reset_hand_synchro(duel, tp);
        e.set_label_object_group(duel, Some(sg));
        Ok(true)
    }

    fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
        let (e, c) = (a.e, a.c);
        let Some(c) = c else { return Ok(()) };
        let g = e.get_label_object_group(duel).unwrap_or_default();
        c.set_material(duel, Some(g));
        if let Some(eff) = c.is_has_effect(duel, EFFECT_MATERIAL_CHECK) {
            let operation = eff.get_operation(duel);
            unported!(
                duel,
                !matches!(operation, None | Some(Operation::Event(_) | Operation::Promptless(_)))
            );
            if let Some(Operation::Event(op) | Operation::Promptless(op)) = operation {
                op(
                    duel,
                    &Args {
                        e: eff,
                        c: Some(c),
                        ..Args::default()
                    },
                )?;
            }
        }
        // GetValue with arguments returns the function itself, so every hand material uses up its count.
        for mc in g {
            if let Some(handmatfilter) = mc.is_has_effect(duel, EFFECT_SYNCHRO_MAT_FROM_HAND) {
                handmatfilter.use_count_limit(duel, a.tp);
            }
        }
        unported!(duel, g.is_exists(duel, |mc, duel| mc.get_flag_effect(duel, 3096468) != 0, 1, None));
        duel.send_to_grave(g, REASON_MATERIAL + REASON_SYNCHRO)?;
        Ok(())
    }
}
