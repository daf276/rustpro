// Port of proc_fusion.lua and proc_fusion_spell.lua: the Fusion.AddProc* procedures (Mix and the variants built on it,
// MixRep, contact fusion) and the Fusion Summon effect (Fusion.CreateSummonEff, SummonEffTG/OP) with their helpers.
// Material lists given as Lua tables, Fusion.AddProcMixRepUnfix and the effect value a card may set on the procedure
// (matcheck) have no caller in the TCG pool. The paths behind EFFECT_FUSION_MAT_RESTRICTION, EFFECT_EXTRA_FUSION_MATERIAL,
// EFFECT_CHAIN_MATERIAL and unofficial effect 511004008 are `unported!` checks; Fusion.SummonEffect and the metatable
// fields (material, material_count, contactfus) are read only by cards that are not ported.
use engine::{Args, CardId, CardSet, ConditionFn, Duel, EffectId, FusionCheck, OperationFn, Script, Value};

use crate::constant::*;
use crate::proc_workaround::CardWorkaround;
use crate::utility::{CardUtility, aux};

pub struct Fusion;

// f(c, fc, sumtype, tp).
pub type MaterialFilter = fn(CardId, &Duel, CardId, u64, u8) -> bool;

// f(c, fc, sumtype, tp, sub, mg, sg) for a filter that reads sg, the materials chosen with c (nil in the first pass, as
// the Lua's globals are); sub and mg have no caller.
pub type GroupMaterialFilter = fn(CardId, &Duel, CardId, u64, u8, Option<CardSet>) -> bool;

// preselect(e, tc): false stops the summon.
pub type Preselect = fn(&mut Duel, EffectId, CardId) -> Script<bool>;

#[derive(Debug, Clone, Copy)]
pub enum FusionMaterial {
    Code(u32),
    Filter(MaterialFilter),
    GroupFilter(GroupMaterialFilter),
}

// Fusion.AddProcMix(c, sub, insf, ...) and the procedures that call it (AddProcMixN, AddProcCode2, ...), with each
// material written out.
pub trait FusionMix {
    const SUB: bool;
    const INSF: bool;
    const MATERIALS: &'static [FusionMaterial];
}

// Fusion.AddProcMixRep(c, sub, insf, fun1, minc, maxc, ...).
pub trait FusionMixRep {
    const SUB: bool;
    const INSF: bool;
    const FUN1: FusionMaterial;
    const MINC: i32;
    const MAXC: i32;
    const MATERIALS: &'static [FusionMaterial] = &[];
}

// Fusion.AddContactProc(c, group, op, sumcon, condition, sumtype, desc, cannotBeLizard): sumcon None for nil or false,
// Value::Int(0) for true. summonToPlayer and summonToZones have no caller among the ported cards.
pub trait ContactFusion {
    const SUMCON: Option<Value> = None;
    const CONDITION: Option<fn(&Duel, u8) -> bool> = None;
    const SUMTYPE: Option<u32> = None;
    const DESC: u64 = 2;
    const CANNOT_BE_LIZARD: bool = true;

    fn group(duel: &Duel, tp: u8) -> CardSet;

    fn op(duel: &mut Duel, g: CardSet, tp: u8, c: CardId) -> Script<()>;
}

// The named parameters of Fusion.CreateSummonEff, SummonEffTG and SummonEffOP; nil ones keep the defaults below.
pub trait FusionSummonParams {
    const DESC: u64 = 1170;
    const EXACTCOUNT: Option<usize> = None;
    const MINCOUNT: Option<usize> = None;
    const MAXCOUNT: Option<usize> = None;
    const VALUE: u64 = 0;
    const LOCATION: u32 = LOCATION_EXTRA;
    const CHKF: Option<u64> = None;
    const NOSUMMONCHECK: bool = false;
    const SUMPOS: u32 = POS_FACEUP;
    const PRESELECT: Option<Preselect> = None;

    fn fusfilter(_c: CardId, _duel: &Duel) -> bool {
        true
    }

    // matfilter(c, e, tp, chk); nil is aux.LimitParams(Card.IsAbleToGrave).
    fn matfilter(c: CardId, duel: &Duel, _e: EffectId, _tp: u8) -> bool {
        c.is_able_to_grave(duel)
    }

    // extrafil(e, tp, mg): the extra materials and the additional check.
    fn extrafil(_duel: &Duel, _e: EffectId, _tp: u8, _mg: CardSet) -> (Option<CardSet>, Option<FusionCheck>) {
        (None, None)
    }

    fn extraop(_duel: &mut Duel, _e: EffectId, _tc: CardId, _tp: u8, _sg: &mut CardSet) -> Script<()> {
        Ok(())
    }

    // gc(e, tp, ...): the materials that must be used.
    fn gc(_duel: &Duel, _e: EffectId, _tp: u8) -> Option<CardSet> {
        None
    }

    fn stage2(_duel: &mut Duel, _e: EffectId, _tc: Option<CardId>, _tp: u8, _mg: Option<CardSet>, _chk: i32) -> Script<()> {
        Ok(())
    }

    fn extratg(_duel: &mut Duel, _a: &Args, _chk: i32) -> Script<bool> {
        Ok(true)
    }
}

// Effect codes of unofficial cards the procedures look up.
const EFFECT_ANY_MATERIAL: u32 = 511002961;
const EFFECT_NOT_MATERIAL: u32 = 6205579;
const EFFECT_OPPONENT_SELECTS: u32 = 511004008;

// Fusion.CheckExact, CheckMin and CheckMax as script state, 0 for nil; keys 1-3 are taken.
const CHECK_EXACT: u64 = 4;
const CHECK_MIN: u64 = 5;
const CHECK_MAX: u64 = 6;
// The chkf upvalue of a SummonEffTG or SummonEffOP closure, per card that created it; SET marks a value.
const UPVALUE_CHKF: u64 = 1 << 62;
const SET: i64 = 1 << 40;

fn check(duel: &Duel, key: u64) -> Option<usize> {
    usize::try_from(duel.get_script_state(key)).ok().filter(|&count| count != 0)
}

fn set_check(duel: &mut Duel, key: u64, count: Option<usize>) {
    duel.set_script_state(key, count.map_or(0, |count| count as i64));
}

fn check_additional(duel: &Duel, tp: u8, sg: CardSet, fc: CardId) -> bool {
    duel.get_fusion_check_additional()
        .is_none_or(|(check, c)| check(duel, c, tp, sg, fc))
}

impl FusionMaterial {
    // The material function the procedures build: fun(c, fc, sub, sub2, mg, sg, tp, contact, sumtype).
    fn check(self, duel: &Duel, c: CardId, fc: CardId, sub: bool, sub2: bool, tp: u8, sumtype: u64, sg: Option<CardSet>) -> bool {
        let any = |duel| sub2 && c.is_has_effect(duel, EFFECT_ANY_MATERIAL).is_some();
        match self {
            Self::Filter(_) | Self::GroupFilter(_) => {
                let passes = match self {
                    Self::Filter(f) => f(c, duel, fc, sumtype, tp),
                    Self::GroupFilter(f) => f(c, duel, fc, sumtype, tp, sg),
                    Self::Code(_) => false,
                };
                (passes || any(duel)) && c.is_has_effect(duel, EFFECT_NOT_MATERIAL).is_none()
            }
            Self::Code(code) => {
                c.is_summon_code(duel, Some(fc), sumtype, fc.get_controler(duel), &[code])
                    || (sub && c.check_fusion_substitute(duel, fc))
                    || any(duel)
            }
        }
    }
}

// What the procedures read from chkfnf and the procedure: the Fusion Monster and its controller, the player whose field
// must fit it, the forced zones, whether this is a contact fusion and the summon type the materials are checked for.
#[derive(Clone, Copy)]
struct Proc {
    fc: CardId,
    tp: u8,
    chkf: u8,
    zone: u64,
    contact: bool,
    cancelable: bool,
    sumtype: u64,
}

impl Proc {
    // Also returns sub as the condition and operation narrow it.
    fn new(duel: &Duel, e: EffectId, chkfnf: u64, sub: bool) -> (Self, bool) {
        let fc = e.get_handler(duel);
        let notfusion = chkfnf & FUSPROC_NOTFUSION != 0;
        let contact = chkfnf & FUSPROC_CONTACTFUS != 0;
        let listedmats = chkfnf & FUSPROC_LISTEDMATS != 0;
        let sumtype = if listedmats {
            0
        } else if contact || notfusion {
            MATERIAL_FUSION
        } else {
            u64::from(SUMMON_TYPE_FUSION) | MATERIAL_FUSION
        };
        let proc = Self {
            fc,
            tp: fc.get_controler(duel),
            chkf: chkfnf as u8,
            zone: match (chkfnf >> 40) & 0xff {
                0 => 0xff,
                zone => zone,
            },
            contact,
            cancelable: chkfnf & (FUSPROC_CONTACTFUS | FUSPROC_CANCELABLE) != 0,
            sumtype,
        };
        (proc, !listedmats && (sub || notfusion) && !contact)
    }

    // Fusion.ConditionFilterMix.
    fn condition_filter(self, duel: &Duel, c: CardId, sub: bool, funs: impl IntoIterator<Item = FusionMaterial>) -> bool {
        c.is_can_be_fusion_material(duel, Some(self.fc), self.sumtype)
            && funs
                .into_iter()
                .any(|f| f.check(duel, c, self.fc, sub, sub, self.tp, self.sumtype, None))
    }

    fn cannot_be_material(self, duel: &Duel, g: CardSet) -> bool {
        g.is_exists(
            duel,
            |c, duel| !c.is_can_be_fusion_material(duel, Some(self.fc), self.sumtype),
            1,
            None,
        )
    }

    // The field check of CheckMixGoal (`mzone`) and of the MixRep goals, which ask the Extra Deck count only.
    fn location_count(self, duel: &Duel, sg: CardSet, mzone: bool) -> bool {
        if self.chkf == PLAYER_NONE {
            return true;
        }
        unported!(duel, self.zone != 0xff);
        let count = match mzone && !self.fc.is_location(duel, LOCATION_EXTRA) {
            true => duel.get_mzone_count(self.chkf, Some(sg)),
            false => duel.get_location_count_from_ex(self.chkf, self.tp, Some(sg), self.fc),
        };
        count > 0
    }

    // The EFFECT_FUSION_MAT_RESTRICTION checks of SelectMix and SelectMixRep.
    fn restrictions(self, duel: &Duel, c: CardId, sg: CardSet) {
        if self.contact {
            return;
        }
        let restricted = |c: CardId, duel: &Duel| c.is_has_effect(duel, EFFECT_FUSION_MAT_RESTRICTION).is_some();
        unported!(duel, restricted(c, duel));
        unported!(duel, sg.is_exists(duel, restricted, 1, None));
    }

    // The player who selects the materials, the opponent under unofficial effect 511004008.
    fn selecting_player(self, duel: &Duel) -> u8 {
        unported!(
            duel,
            !self.contact && duel.is_player_affected_by_effect(self.tp, EFFECT_OPPONENT_SELECTS).is_some()
        );
        self.tp
    }

    // Toggles tc in the selection sg unless it is one of the materials that must be used.
    fn toggle(sg: &mut CardSet, mustg: CardSet, tc: CardId) {
        if !mustg.is_empty() && mustg.is_contains(tc) {
            return;
        }
        match sg.is_contains(tc) {
            false => sg.insert(tc),
            true => sg.remove(tc),
        }
    }
}

impl Fusion {
    fn register_material_effect(duel: &mut Duel, c: CardId, condition: ConditionFn, operation: OperationFn) -> EffectId {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
        e1.set_code(duel, EFFECT_FUSION_MATERIAL);
        e1.set_condition(duel, condition);
        e1.set_operation(duel, operation);
        c.register_effect(duel, e1);
        e1
    }

    pub fn add_proc_mix<M: FusionMix>(duel: &mut Duel, c: CardId) -> Option<EffectId> {
        if c.is_status(duel, STATUS_COPYING_EFFECT) {
            return None;
        }
        Some(Fusion::register_material_effect(
            duel,
            c,
            Fusion::condition_mix::<M>,
            Fusion::operation_mix::<M>,
        ))
    }

    pub fn add_proc_mix_n<M: FusionMix>(duel: &mut Duel, c: CardId) -> Option<EffectId> {
        Fusion::add_proc_mix::<M>(duel, c)
    }

    pub fn add_proc_mix_rep<M: FusionMixRep>(duel: &mut Duel, c: CardId) -> Option<EffectId> {
        if c.is_status(duel, STATUS_COPYING_EFFECT) {
            return None;
        }
        Some(Fusion::register_material_effect(
            duel,
            c,
            Fusion::condition_mix_rep::<M>,
            Fusion::operation_mix_rep::<M>,
        ))
    }

    fn condition_mix<M: FusionMix>(duel: &Duel, a: &Args) -> bool {
        let c = a.e.get_handler(duel);
        let tp = c.get_controler(duel);
        let Some(g) = a.materials else {
            let mustg = aux::get_must_be_material_group(duel, tp, None, tp, Some(c), None, REASON_FUSION);
            return M::INSF && mustg.is_empty();
        };
        let (s, sub) = Proc::new(duel, a.e, a.chkf, M::SUB);
        let funs = M::MATERIALS;
        let filter = |mc, duel: &Duel| s.condition_filter(duel, mc, sub, funs.iter().copied());
        let mut mg = g.filter(duel, filter, Some(c.into()));
        let mut mustg = aux::get_must_be_material_group(duel, tp, Some(g), tp, Some(c), Some(mg), REASON_FUSION);
        if s.contact {
            mustg.clear();
        }
        if !mg.includes(mustg) || s.cannot_be_material(duel, mustg) {
            return false;
        }
        if let Some(gc) = a.must {
            if s.cannot_be_material(duel, gc) || gc.is_exists(duel, |mc, duel| !filter(mc, duel), 1, None) {
                return false;
            }
            mustg.merge(gc);
        }
        mg.merge(mustg);
        let sg = CardSet::default();
        mg.is_exists(
            duel,
            |mc, duel| Fusion::select_mix(duel, mc, s, mg, sg, mustg, sub, sub, funs),
            1,
            None,
        )
    }

    fn operation_mix<M: FusionMix>(duel: &mut Duel, a: &Args) -> Script<()> {
        let (s, sub) = Proc::new(duel, a.e, a.chkf, M::SUB);
        let (c, tp, funs) = (s.fc, s.tp, M::MATERIALS);
        let eg = a.eg.unwrap_or_default();
        let mg = eg.filter(
            duel,
            |mc, duel| s.condition_filter(duel, mc, sub, funs.iter().copied()),
            Some(c.into()),
        );
        let mut mustg = aux::get_must_be_material_group(duel, tp, Some(eg), tp, Some(c), Some(mg), REASON_FUSION);
        if s.contact {
            mustg.clear();
        }
        let mut sg = CardSet::default();
        if let Some(gc) = a.must {
            mustg.merge(gc);
        }
        for tc in mustg {
            sg.insert(tc);
            unported!(duel, !s.contact && tc.is_has_effect(duel, EFFECT_FUSION_MAT_RESTRICTION).is_some());
        }
        let p = s.selecting_player(duel);
        let count = funs.len() as u8;
        while sg.len() < funs.len() {
            duel.hint(HINT_SELECTMSG, p, HINTMSG_FMATERIAL);
            let mustg_left = mustg.difference(sg);
            let cg = mg.filter(
                duel,
                |mc, duel| Fusion::select_mix(duel, mc, s, mg, sg, mustg_left, sub, sub, funs),
                Some(sg),
            );
            let Some(tc) = cg.select_unselect(duel, Some(sg), p, false, s.cancelable && sg.is_empty(), count, count)? else {
                break;
            };
            Proc::toggle(&mut sg, mustg, tc);
        }
        duel.set_fusion_material(sg);
        Ok(())
    }

    // Fusion.CheckMix: whether c, then the rest of mg, fill funs in order.
    fn check_mix(duel: &Duel, c: CardId, mg: CardSet, sg: CardSet, s: Proc, sub: bool, sub2: bool, funs: &[FusionMaterial]) -> bool {
        let (fc, tp, sumtype) = (s.fc, s.tp, s.sumtype);
        let [fun1, rest @ ..] = funs else { return false };
        if rest.is_empty() {
            return fun1.check(duel, c, fc, sub, sub2, tp, sumtype, Some(sg));
        }
        let sg = sg.union(c.into());
        let next = |sub| {
            mg.is_exists(
                duel,
                |tc, duel| Fusion::check_mix(duel, tc, mg, sg, s, sub, sub2, rest),
                1,
                Some(sg),
            )
        };
        if fun1.check(duel, c, fc, false, sub2, tp, sumtype, Some(sg)) {
            next(sub)
        } else if sub && fun1.check(duel, c, fc, true, sub2, tp, sumtype, Some(sg)) {
            next(false)
        } else {
            false
        }
    }

    fn check_mix_goal(duel: &Duel, sg: CardSet, s: Proc, sub: bool, sub2: bool, funs: &[FusionMaterial]) -> bool {
        let g = CardSet::default();
        sg.is_exists(duel, |c, duel| Fusion::check_mix(duel, c, sg, g, s, sub, sub2, funs), 1, None)
            && s.location_count(duel, sg, true)
            && check_additional(duel, s.tp, sg, s.fc)
    }

    fn select_mix(
        duel: &Duel,
        c: CardId,
        s: Proc,
        mg: CardSet,
        sg: CardSet,
        mustg: CardSet,
        sub: bool,
        sub2: bool,
        funs: &[FusionMaterial],
    ) -> bool {
        let (totalcount, mustgcount) = (funs.len(), mustg.len());
        if mustgcount > totalcount
            || check(duel, CHECK_EXACT).is_some_and(|exact| exact != totalcount || mustgcount > exact)
            || check(duel, CHECK_MAX).is_some_and(|max| max < totalcount || mustgcount > max)
            || check(duel, CHECK_MIN).is_some_and(|min| min > totalcount)
        {
            return false;
        }
        s.restrictions(duel, c, sg);
        let sg = sg.union(c.into());
        if sg.len() < totalcount {
            let mustg = mustg.difference(sg);
            mg.is_exists(
                duel,
                |tc, duel| Fusion::select_mix(duel, tc, s, mg, sg, mustg, sub, sub2, funs),
                1,
                Some(sg),
            )
        } else {
            Fusion::check_mix_goal(duel, sg, s, sub, sub2, funs) && sg.includes(mustg)
        }
    }
}

// Which of the fixed materials of Fusion.AddProcMixRep (FusionMixRep::MATERIALS) are still to fill, by index: the Lua's
// varargs after table.remove, in their order.
type Rest = u8;

fn rest_iter<M: FusionMixRep>(rest: Rest) -> impl Iterator<Item = (usize, FusionMaterial)> {
    M::MATERIALS.iter().copied().enumerate().filter(move |&(i, _)| rest & (1 << i) != 0)
}

// The groups the MixRep selection checks share: the procedure, the material pool, the selection and the materials it
// must use.
#[derive(Clone, Copy)]
struct Search {
    proc: Proc,
    mg: CardSet,
    sg: CardSet,
    mustg: CardSet,
}

// The state the MixRep recursion carries: whether substitutes may still be used (sub and sub2), the counts of fun1
// still possible, and the fixed materials still to fill.
#[derive(Clone, Copy)]
struct Rep {
    sub: bool,
    sub2: bool,
    minc: i32,
    maxc: i32,
    rest: Rest,
}

impl Rep {
    fn fixed(self) -> i32 {
        self.rest.count_ones() as i32
    }

    // A card given to fixed material i, or to fun1 when i is None; sub stays only if the card also passes without it.
    fn after(self, duel: &Duel, s: Proc, c: CardId, i: Option<usize>, f: FusionMaterial, g: CardSet) -> Self {
        let sub = self.sub && f.check(duel, c, s.fc, false, self.sub2, s.tp, s.sumtype, Some(g));
        match i {
            Some(i) => Self {
                sub,
                rest: self.rest & !(1 << i),
                ..self
            },
            None => Self {
                sub,
                minc: self.minc - 1,
                maxc: self.maxc - 1,
                ..self
            },
        }
    }

    fn passes(self, duel: &Duel, s: Proc, c: CardId, f: FusionMaterial, g: CardSet) -> bool {
        f.check(duel, c, s.fc, self.sub, self.sub2, s.tp, s.sumtype, Some(g))
    }
}

impl Fusion {
    fn rep_funs<M: FusionMixRep>() -> impl Iterator<Item = FusionMaterial> {
        std::iter::once(M::FUN1).chain(M::MATERIALS.iter().copied())
    }

    fn rep<M: FusionMixRep>(sub: bool) -> Rep {
        Rep {
            sub,
            sub2: sub,
            minc: M::MINC,
            maxc: M::MAXC,
            rest: ((1u16 << M::MATERIALS.len()) - 1) as Rest,
        }
    }

    fn condition_mix_rep<M: FusionMixRep>(duel: &Duel, a: &Args) -> bool {
        let c = a.e.get_handler(duel);
        let tp = c.get_controler(duel);
        let Some(g) = a.materials else {
            return M::INSF && aux::get_must_be_material_group(duel, tp, None, tp, Some(c), None, REASON_FUSION).is_empty();
        };
        let (s, sub) = Proc::new(duel, a.e, a.chkf, M::SUB);
        // The Lua passes the globals eg and mg here, which are nil.
        let mut mustg = aux::get_must_be_material_group(duel, tp, None, tp, Some(c), None, REASON_FUSION);
        if s.contact {
            mustg.clear();
        }
        let filter = |mc, duel: &Duel| s.condition_filter(duel, mc, sub, Fusion::rep_funs::<M>());
        let mut mg = g.filter(duel, filter, Some(c.into()));
        if !mg.includes(mustg) || s.cannot_be_material(duel, mustg) {
            return false;
        }
        if let Some(gc) = a.must {
            if s.cannot_be_material(duel, gc) || gc.is_exists(duel, |mc, duel| !filter(mc, duel), 1, None) {
                return false;
            }
            mustg.merge(gc);
        }
        mg.merge(mustg);
        let search = Search {
            proc: s,
            mg,
            sg: CardSet::default(),
            mustg,
        };
        mg.is_exists(
            duel,
            |mc, duel| Fusion::select_mix_rep::<M>(duel, mc, search, Fusion::rep::<M>(sub)),
            1,
            None,
        )
    }

    fn operation_mix_rep<M: FusionMixRep>(duel: &mut Duel, a: &Args) -> Script<()> {
        let (s, sub) = Proc::new(duel, a.e, a.chkf, M::SUB);
        let (c, tp) = (s.fc, s.tp);
        let mut sg = CardSet::default();
        let eg = a.eg.unwrap_or_default();
        let mg = eg.filter(
            duel,
            |mc, duel| s.condition_filter(duel, mc, sub, Fusion::rep_funs::<M>()),
            Some(c.into()),
        );
        let mut mustg = aux::get_must_be_material_group(duel, tp, Some(eg), tp, Some(c), Some(mg), REASON_FUSION);
        if s.contact {
            mustg.clear();
        }
        if !mg.includes(mustg) || s.cannot_be_material(duel, mustg) {
            return Ok(());
        }
        if let Some(gc) = a.must {
            mustg.merge(gc);
        }
        sg.merge(mustg);
        let p = s.selecting_player(duel);
        let rep = Fusion::rep::<M>(sub);
        while sg.len() < M::MAXC as usize + M::MATERIALS.len() {
            let search = Search { proc: s, mg, sg, mustg };
            let cg = mg.filter(duel, |mc, duel| Fusion::select_mix_rep::<M>(duel, mc, search, rep), Some(sg));
            if cg.is_empty() {
                break;
            }
            let finish = Fusion::check_mix_rep_goal::<M>(duel, sg, s, rep)
                && check(duel, CHECK_EXACT).is_none()
                && !check(duel, CHECK_MIN).is_some_and(|min| sg.len() < min);
            let cancel = s.cancelable && sg.is_empty();
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_FMATERIAL);
            let Some(tc) = cg.select_unselect(duel, Some(sg), p, finish, cancel, 1, 1)? else {
                break;
            };
            Proc::toggle(&mut sg, mustg, tc);
        }
        duel.set_fusion_material(sg);
        Ok(())
    }

    // Fusion.CheckMixRep: whether sg fills the fixed materials still to fill and fun1 with the others; g holds the cards
    // given to fixed materials so far.
    fn check_mix_rep<M: FusionMixRep>(duel: &Duel, sg: CardSet, g: CardSet, s: Proc, rep: Rep) -> bool {
        if let Some((i, fun2)) = rest_iter::<M>(rep.rest).next() {
            let filter = |c: CardId, duel: &Duel| {
                let g = g.union(c.into());
                rep.passes(duel, s, c, fun2, g) && Fusion::check_mix_rep::<M>(duel, sg, g, s, rep.after(duel, s, c, Some(i), fun2, g))
            };
            return sg.is_exists(duel, filter, 1, Some(g));
        }
        let ct1 = sg.filter_count(duel, |c, duel| rep.passes(duel, s, c, M::FUN1, sg), Some(g));
        let ct2 = sg.filter_count(duel, |c, duel| Rep { sub: false, ..rep }.passes(duel, s, c, M::FUN1, sg), Some(g));
        ct1 + g.len() == sg.len() && ct1 <= ct2 + 1
    }

    fn check_mix_rep_goal<M: FusionMixRep>(duel: &Duel, sg: CardSet, s: Proc, rep: Rep) -> bool {
        let count = sg.len() as i32;
        if count < rep.minc + rep.fixed() || count > rep.maxc + rep.fixed() {
            return false;
        }
        Fusion::check_mix_rep::<M>(duel, sg, CardSet::default(), s, rep)
            && s.location_count(duel, sg, false)
            && check_additional(duel, s.tp, sg, s.fc)
    }

    // Fusion.CheckMixRepTemplate with cond CheckMixRepSelectedCond when `selected`, else CheckSelectMixRep.
    fn check_mix_rep_template<M: FusionMixRep>(duel: &Duel, c: CardId, selected: bool, search: Search, g: CardSet, rep: Rep) -> bool {
        let s = search.proc;
        let search = Search {
            mustg: search.mustg.difference(c.into()),
            ..search
        };
        let g = g.union(c.into());
        let cond = |rep| match selected {
            true => Fusion::check_mix_rep_selected_cond::<M>(duel, search, g, rep),
            false => Fusion::check_select_mix_rep::<M>(duel, search, g, rep),
        };
        let fixed = rest_iter::<M>(rep.rest).any(|(i, f)| rep.passes(duel, s, c, f, g) && cond(rep.after(duel, s, c, Some(i), f, g)));
        fixed || (rep.maxc > 0 && rep.passes(duel, s, c, M::FUN1, g) && cond(rep.after(duel, s, c, None, M::FUN1, g)))
    }

    fn check_mix_rep_selected_cond<M: FusionMixRep>(duel: &Duel, search: Search, g: CardSet, rep: Rep) -> bool {
        if g.len() < search.sg.len() {
            let selected = |c, duel: &Duel| Fusion::check_mix_rep_template::<M>(duel, c, true, search, g, rep);
            search.sg.is_exists(duel, selected, 1, Some(g))
        } else {
            Fusion::check_select_mix_rep::<M>(duel, search, g, rep)
        }
    }

    fn check_select_mix_rep<M: FusionMixRep>(duel: &Duel, search: Search, g: CardSet, rep: Rep) -> bool {
        let s = search.proc;
        if !check_additional(duel, s.tp, g, s.fc) {
            return false;
        }
        if s.location_count(duel, g, false) {
            if rep.minc <= 0 && rep.rest == 0 && g.includes(search.mustg) {
                return true;
            }
            let all = |c, duel: &Duel| Fusion::check_select_mix_rep_all::<M>(duel, c, search, g, rep);
            search.mg.is_exists(duel, all, 1, Some(g))
        } else {
            let in_mzone = |c: CardId, duel: &Duel| {
                c.is_controler(duel, s.tp)
                    && c.is_location(duel, LOCATION_MZONE)
                    && Fusion::check_mix_rep_template::<M>(duel, c, false, search, g, rep)
            };
            search.mg.is_exists(duel, in_mzone, 1, Some(g))
        }
    }

    fn check_select_mix_rep_all<M: FusionMixRep>(duel: &Duel, c: CardId, search: Search, g: CardSet, rep: Rep) -> bool {
        let s = search.proc;
        let g = g.union(c.into());
        if let Some((i, fun2)) = rest_iter::<M>(rep.rest).next() {
            return rep.passes(duel, s, c, fun2, g)
                && Fusion::check_select_mix_rep::<M>(duel, search, g, rep.after(duel, s, c, Some(i), fun2, g));
        }
        rep.maxc > 0
            && rep.passes(duel, s, c, M::FUN1, g)
            && Fusion::check_select_mix_rep::<M>(duel, search, g, rep.after(duel, s, c, None, M::FUN1, g))
    }

    fn select_mix_rep<M: FusionMixRep>(duel: &Duel, c: CardId, search: Search, rep: Rep) -> bool {
        let (s, mustgcount) = (search.proc, search.mustg.len() as i32);
        let (totalcount, mut rep) = (rep.fixed(), rep);
        if let Some(exact) = check(duel, CHECK_EXACT).map(|exact| exact as i32) {
            if exact < rep.minc + totalcount || mustgcount > exact {
                return false;
            }
            (rep.minc, rep.maxc) = (exact - totalcount, exact - totalcount);
        }
        if let Some(max) = check(duel, CHECK_MAX).map(|max| max as i32) {
            if max < rep.minc + totalcount || mustgcount > max {
                return false;
            }
            rep.maxc = rep.maxc.min(max - totalcount);
        }
        if let Some(min) = check(duel, CHECK_MIN).map(|min| min as i32) {
            if min > rep.maxc + totalcount {
                return false;
            }
            rep.minc = rep.minc.max(min - totalcount);
        }
        s.restrictions(duel, c, search.sg);
        let sg = search.sg.union(c.into());
        if !check_additional(duel, s.tp, sg, s.fc) {
            return false;
        }
        if Fusion::check_mix_rep_goal::<M>(duel, sg, s, rep) {
            return true;
        }
        let (search, g) = (Search { sg, ..search }, CardSet::default());
        sg.is_exists(
            duel,
            |tc, duel| Fusion::check_mix_rep_template::<M>(duel, tc, true, search, g, rep),
            1,
            None,
        )
    }
}

// Fusion.AddContactProc.
impl Fusion {
    pub fn add_contact_proc<P: ContactFusion>(duel: &mut Duel, c: CardId) {
        if c.is_status(duel, STATUS_COPYING_EFFECT) {
            return;
        }
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_FIELD);
        e1.set_description(duel, P::DESC);
        e1.set_code(duel, EFFECT_SPSUMMON_PROC);
        e1.set_range(duel, LOCATION_EXTRA);
        e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
        if let Some(sumtype) = P::SUMTYPE {
            e1.set_value(duel, sumtype as i32);
        }
        e1.set_condition(duel, Fusion::contact_con::<P>);
        e1.set_target(duel, Fusion::contact_tg::<P>);
        e1.set_operation(duel, Fusion::contact_op::<P>);
        let e2 = P::SUMCON.map(|sumcon| {
            let e2 = duel.create_effect(c);
            e2.set_type(duel, EFFECT_TYPE_SINGLE);
            e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
            e2.set_code(duel, EFFECT_SPSUMMON_CONDITION);
            if let Value::SpSummonCondition(_) = sumcon {
                e2.set_value(duel, sumcon);
            }
            e2
        });
        if P::CANNOT_BE_LIZARD {
            aux::add_lizard_check(duel, c);
        }
        c.register_effect(duel, e1);
        if let Some(e2) = e2 {
            c.register_effect(duel, e2);
        }
    }

    fn contact_con<P: ContactFusion>(duel: &Duel, a: &Args) -> bool {
        let Some(c) = a.c else { return true };
        let tp = a.e.get_handler_player(duel);
        let m = P::group(duel, tp);
        let chkf = u64::from(c.get_controler(duel)) | FUSPROC_CONTACTFUS;
        c.is_facedown(duel) && c.check_fusion_material(duel, Some(m), None, chkf) && P::CONDITION.is_none_or(|fcon| fcon(duel, tp))
    }

    fn contact_tg<P: ContactFusion>(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
        let (e, tp) = (a.e, a.tp);
        let m = P::group(duel, tp);
        let chkf = u64::from(tp) | FUSPROC_CONTACTFUS;
        let sg = duel.select_fusion_material(tp, e.get_handler(duel), m, None, chkf)?;
        if sg.is_empty() {
            return Ok(false);
        }
        e.set_label_object_group(duel, Some(sg));
        Ok(true)
    }

    fn contact_op<P: ContactFusion>(duel: &mut Duel, a: &Args) -> Script<()> {
        let Some(c) = a.c else { return Ok(()) };
        let g = a.e.get_label_object_group(duel).unwrap_or_default();
        c.set_material(duel, Some(g));
        P::op(duel, g, a.tp, c)
    }
}

// proc_fusion_spell.lua's replacement of Duel.GetFusionMaterial.
pub trait DuelFusion {
    fn get_fusion_material(&self, tp: u8) -> CardSet;
}

impl DuelFusion for Duel {
    fn get_fusion_material(&self, tp: u8) -> CardSet {
        let mut res = self.get_fusion_material_core(tp);
        let g = self.get_matching_group(
            |c, duel| c.is_has_effect(duel, EFFECT_EXTRA_FUSION_MATERIAL).is_some(),
            tp,
            LOCATION_EXTRA,
            0,
            None,
        );
        unported!(self, !g.is_empty());
        res.merge(g);
        res
    }
}

// GetExtraMatEff: whether an EFFECT_EXTRA_FUSION_MATERIAL effect other than proc_fusion_spell's own applies to c.
fn extra_mat_eff(c: CardId, duel: &Duel) -> bool {
    let effect = c.is_has_effect(duel, EFFECT_EXTRA_FUSION_MATERIAL);
    unported!(duel, effect.is_some());
    effect.is_some()
}

// Fusion.ExtraGroup is read only by proc_fusion_spell's global EFFECT_EXTRA_FUSION_MATERIAL, which is not ported: the
// effect lets these cards be materials, which monsters and cards on the field already may.
fn set_extra_group(duel: &Duel, g: CardSet) {
    unported!(
        duel,
        g.is_exists(
            duel,
            |c, duel| !c.is_on_field(duel) && !c.is_original_type(duel, TYPE_MONSTER),
            1,
            None
        )
    );
}

impl Fusion {
    pub fn create_summon_eff<P: FusionSummonParams>(duel: &mut Duel, c: CardId) -> EffectId {
        let e1 = duel.create_effect(c);
        e1.set_description(duel, P::DESC);
        e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_FUSION_SUMMON);
        e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
        e1.set_code(duel, EVENT_FREE_CHAIN);
        e1.set_target(duel, Fusion::summon_eff_tg::<P>);
        e1.set_operation(duel, Fusion::summon_eff_op::<P>);
        e1
    }

    pub fn register_summon_eff<P: FusionSummonParams>(duel: &mut Duel, c: CardId) -> EffectId {
        let e1 = Fusion::create_summon_eff::<P>(duel, c);
        c.register_effect(duel, e1);
        e1
    }

    // The closures SummonEffTG and SummonEffOP return assign their chkf upvalue on every call (chkf|tp), so a closure
    // keeps the bit of every player who used it; Effect.Clone shares the closure, so it is kept per card that created
    // it.
    fn upvalue_chkf<P: FusionSummonParams>(duel: &mut Duel, e: EffectId, tp: u8, op: bool, add: u64) -> u64 {
        let key = UPVALUE_CHKF | ((e.get_owner(duel).index() as u64) << 1) | u64::from(op);
        let stored = duel.get_script_state(key);
        let chkf = match stored & SET {
            0 => P::CHKF,
            _ => Some((stored & !SET) as u64),
        };
        let chkf = match chkf {
            Some(chkf) if chkf & u64::from(PLAYER_NONE) == u64::from(PLAYER_NONE) => chkf,
            Some(chkf) => chkf | u64::from(tp),
            None => u64::from(tp),
        } | add;
        duel.set_script_state(key, chkf as i64 | SET);
        chkf
    }

    // Fusion.SummonEffFilter; efmg holds no card while EFFECT_EXTRA_FUSION_MATERIAL is not ported.
    fn summon_eff_filter<P: FusionSummonParams>(
        c: CardId,
        duel: &Duel,
        e: EffectId,
        tp: u8,
        mg: CardSet,
        gc: Option<CardSet>,
        chkf: u64,
        value: u32,
        sumlimit: bool,
    ) -> bool {
        c.is_type(duel, TYPE_FUSION)
            && P::fusfilter(c, duel)
            && (P::NOSUMMONCHECK || c.is_can_be_special_summoned_full(duel, e, value, tp, sumlimit, false, P::SUMPOS))
            && c.check_fusion_material(duel, Some(mg), gc, chkf)
    }

    // The materials the target and the operation start from: mg1 after matfilter with the extra materials merged, the
    // extra materials, and the additional check extrafil returns. `extra_group` filters the extra materials into
    // Fusion.ExtraGroup.
    fn summon_materials<P: FusionSummonParams>(
        duel: &Duel,
        e: EffectId,
        tp: u8,
        extra_group: impl Fn(&Duel, CardSet) -> CardSet,
    ) -> (CardSet, Option<CardSet>, Option<FusionCheck>) {
        let fmg_all = duel.get_fusion_material(tp);
        let mut mg1 = fmg_all.filter(duel, |c, duel| P::matfilter(c, duel, e, tp), None);
        fmg_all.filter(duel, extra_mat_eff, None);
        let (extra, check_addition) = P::extrafil(duel, e, tp, mg1);
        if let Some(extra) = extra {
            set_extra_group(duel, extra_group(duel, extra));
            mg1.merge(extra);
        }
        (mg1, extra, check_addition)
    }

    pub fn summon_eff_tg<P: FusionSummonParams>(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
        let (e, tp) = (a.e, a.tp);
        let chkf = Fusion::upvalue_chkf::<P>(duel, e, tp, false, 0);
        let sumlimit = chkf & (FUSPROC_NOTFUSION | FUSPROC_NOLIMIT) != 0;
        let notfusion = chkf & FUSPROC_NOTFUSION != 0;
        let value = match notfusion {
            true => P::VALUE | MATERIAL_FUSION,
            false => P::VALUE | MATERIAL_FUSION | u64::from(SUMMON_TYPE_FUSION),
        };
        let gc = P::gc(duel, e, tp);
        if chk == 0 {
            let can_be_material = |c: CardId, duel: &Duel| c.is_can_be_fusion_material(duel, None, value);
            let not_immune = |c: CardId, duel: &Duel| !c.is_immune_to_effect(duel, e);
            let extra_group = |duel: &Duel, g: CardSet| g.filter(duel, can_be_material, None).filter(duel, not_immune, None);
            let (mut mg1, _, check_addition) = Fusion::summon_materials::<P>(duel, e, tp, extra_group);
            if gc.is_some_and(|gc| !mg1.includes(gc)) {
                return Ok(false);
            }
            duel.set_fusion_check_additional(check_addition);
            mg1.keep_matching(duel, can_be_material, None);
            mg1.keep_matching(duel, not_immune, None);
            set_check(duel, CHECK_EXACT, P::EXACTCOUNT);
            set_check(duel, CHECK_MIN, P::MINCOUNT);
            set_check(duel, CHECK_MAX, P::MAXCOUNT);
            mg1.filter(duel, extra_mat_eff, None);
            let filter = |c, duel: &Duel| Fusion::summon_eff_filter::<P>(c, duel, e, tp, mg1, gc, chkf, value as u32, sumlimit);
            let res = duel.is_existing_matching_card(filter, tp, P::LOCATION, 0, 1, None);
            duel.set_fusion_check_additional(None);
            if !res && !notfusion {
                unported!(duel, !duel.get_player_effect(tp, EFFECT_CHAIN_MATERIAL).is_empty());
            }
            set_check(duel, CHECK_EXACT, None);
            set_check(duel, CHECK_MIN, None);
            set_check(duel, CHECK_MAX, None);
            return Ok(res);
        }
        duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, P::LOCATION as i32);
        P::extratg(duel, a, chk)?;
        Ok(false)
    }

    pub fn summon_eff_op<P: FusionSummonParams>(duel: &mut Duel, a: &Args) -> Script<()> {
        let (e, tp) = (a.e, a.tp);
        let cancelable = if P::PRESELECT.is_none() { FUSPROC_CANCELABLE } else { 0 };
        let chkf = Fusion::upvalue_chkf::<P>(duel, e, tp, true, cancelable);
        let sumlimit = chkf & (FUSPROC_NOTFUSION | FUSPROC_NOLIMIT) != 0;
        let notfusion = chkf & FUSPROC_NOTFUSION != 0;
        let value = match notfusion {
            true => P::VALUE,
            false => P::VALUE | u64::from(SUMMON_TYPE_FUSION) | MATERIAL_FUSION,
        };
        let gc = P::gc(duel, e, tp);
        let material_filter = |c: CardId, duel: &Duel| {
            c.is_can_be_fusion_material(duel, None, value) && !c.is_immune_to_effect(duel, e) && aux::nvfilter(c, duel)
        };
        let extra_group = |duel: &Duel, g: CardSet| g.filter(duel, material_filter, None);
        let (mut mg1, extragroup, check_addition) = Fusion::summon_materials::<P>(duel, e, tp, extra_group);
        mg1.keep_matching(duel, material_filter, None);
        let forced_mat_validity = |c: CardId, duel: &Duel| match c == e.get_handler(duel) {
            true => !c.is_relate_to_effect(duel, e),
            false => c.is_immune_to_effect(duel, e),
        };
        if gc.is_some_and(|gc| !mg1.includes(gc) || gc.is_exists(duel, forced_mat_validity, 1, None)) {
            return Ok(());
        }
        set_check(duel, CHECK_EXACT, P::EXACTCOUNT);
        set_check(duel, CHECK_MIN, P::MINCOUNT);
        set_check(duel, CHECK_MAX, P::MAXCOUNT);
        duel.set_fusion_check_additional(check_addition);
        mg1.filter(duel, extra_mat_eff, None);
        let filter = |c, duel: &Duel| Fusion::summon_eff_filter::<P>(c, duel, e, tp, mg1, gc, chkf, value as u32, sumlimit);
        let sg1 = duel.get_matching_group(filter, tp, P::LOCATION, 0, None);
        duel.set_fusion_check_additional(None);
        if !notfusion {
            unported!(duel, !duel.get_player_effect(tp, EFFECT_CHAIN_MATERIAL).is_empty());
        }
        if !sg1.is_empty() {
            let (tc, mut mat1) = loop {
                duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
                let Some(tc) = sg1.select(duel, tp, 1, 1, None)?.get_first() else {
                    return Ok(());
                };
                if let Some(preselect) = P::PRESELECT
                    && !preselect(duel, e, tc)?
                {
                    return Ok(());
                }
                duel.set_fusion_check_additional(check_addition);
                if let Some(extragroup) = extragroup {
                    set_extra_group(duel, extragroup);
                }
                let mat1 = duel.select_fusion_material(tp, tc, mg1, gc, chkf)?;
                if !mat1.is_empty() {
                    break (tc, mat1);
                }
            };
            let backupmat = mat1;
            if !notfusion {
                tc.set_material(duel, Some(mat1));
            }
            mat1.filter(duel, extra_mat_eff, None);
            P::extraop(duel, e, tc, tp, &mut mat1)?;
            if !mat1.is_empty() {
                // Group.AddMaximumCheck adds only Maximum monsters (proc_maximum.lua, out of scope).
                let normal_mg = mat1.filter(duel, |c, duel| !extra_mat_eff(c, duel), None);
                if !normal_mg.is_empty() {
                    duel.send_to_grave(normal_mg, REASON_EFFECT + REASON_MATERIAL + REASON_FUSION)?;
                }
            }
            duel.break_effect()?;
            duel.special_summon_step(tc, value as u32, tp, tp, sumlimit, false, P::SUMPOS)?;
            P::stage2(duel, e, Some(tc), tp, Some(backupmat), 0)?;
            duel.special_summon_complete()?;
            P::stage2(duel, e, Some(tc), tp, Some(backupmat), 3)?;
            if chkf & FUSPROC_NOTFUSION == 0 {
                tc.complete_procedure(duel);
            }
            P::stage2(duel, e, Some(tc), tp, Some(backupmat), 1)?;
        }
        P::stage2(duel, e, None, tp, None, 2)?;
        set_check(duel, CHECK_MIN, None);
        set_check(duel, CHECK_MAX, None);
        set_check(duel, CHECK_EXACT, None);
        duel.set_fusion_check_additional(None);
        Ok(())
    }

    pub fn banish_material(duel: &mut Duel, _e: EffectId, _tc: CardId, _tp: u8, sg: &mut CardSet) -> Script<()> {
        duel.remove(*sg, POS_FACEUP, REASON_EFFECT + REASON_MATERIAL + REASON_FUSION)?;
        sg.clear();
        Ok(())
    }

    // Fusion.IsMonsterFilter(f): `filter` is f.
    pub fn is_monster_filter(c: CardId, duel: &Duel, filter: impl Fn(CardId, &Duel) -> bool) -> bool {
        c.is_monster(duel) && filter(c, duel)
    }

    // Fusion.CheckWithHandler(fun)(c, e, ...): `fun` is fun with its arguments.
    pub fn check_with_handler(c: CardId, duel: &Duel, e: EffectId, fun: impl Fn(CardId, &Duel) -> bool) -> bool {
        c == e.get_handler(duel) || fun(c, duel)
    }

    pub fn forced_handler(duel: &Duel, e: EffectId) -> CardSet {
        e.get_handler(duel).into()
    }
}
