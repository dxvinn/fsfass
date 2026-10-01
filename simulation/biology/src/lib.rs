//! Body truth (report 04 §C.3, simplified for Phase 2).
//!
//! This crate models innate biology only: metabolism, water balance, skin
//! temperature, thermal tissue damage, healing, nociceptors, a protective
//! withdrawal reflex and sleep pressure (Borbély process S). It knows nothing
//! about what kinds of objects exist. It is told physical quantities
//! (contact temperature, ingested kcal and water, radiant heat) by the world.

use alife_core::hash::{StableHash, StateHasher};
use alife_core::{fx, Fx};

/// Skin sites that can contact objects.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Site {
    Hand,
    Mouth,
}

/// Physiological constants (per second of game time).
pub mod k {
    use alife_core::{fx, Fx};
    pub fn energy_setpoint() -> Fx { fx(1000.0) }       // kcal reserve target
    pub fn hunger_span() -> Fx { fx(400.0) }            // kcal deficit at which hunger = 1
    pub fn bmr_per_s() -> Fx { fx(0.015) }              // ~1300 kcal/day (child)
    pub fn digest_tau_s() -> Fx { fx(600.0) }           // stomach -> reserve
    pub fn water_setpoint() -> Fx { fx(1500.0) }        // ml
    pub fn thirst_span() -> Fx { fx(300.0) }            // ml deficit at which thirst = 1
    pub fn water_loss_per_s() -> Fx { fx(0.012) }       // ~1 L/day
    pub fn skin_rest_c() -> Fx { fx(33.0) }
    pub fn contact_gain() -> Fx { fx(0.12) }            // fraction of temperature gap closed per s of contact
    pub fn skin_relax() -> Fx { fx(0.30) }              // per s toward rest without contact
    pub fn damage_threshold_c() -> Fx { fx(44.0) }
    pub fn damage_rate() -> Fx { fx(0.003) }            // per degree above threshold per s
    pub fn heal_tau_s() -> Fx { fx(216_000.0) }         // 2.5 days
    pub fn noci_threshold_c() -> Fx { fx(43.0) }
    pub fn noci_span_c() -> Fx { fx(25.0) }
    pub fn reflex_threshold() -> Fx { fx(0.25) }
    pub fn sleep_rise_tau_s() -> Fx { fx(36_000.0) }    // 10 h
    pub fn sleep_fall_tau_s() -> Fx { fx(21_600.0) }    // 6 h
    pub fn sleep_onset() -> Fx { fx(0.78) }
    pub fn sleep_offset() -> Fx { fx(0.20) }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Body {
    pub age_s: i64,
    pub energy: Fx,
    pub stomach_kcal: Fx,
    pub stomach_water: Fx,
    pub water: Fx,
    pub skin_hand: Fx,
    pub skin_mouth: Fx,
    pub damage_hand: Fx,
    pub damage_mouth: Fx,
    /// Radiant heat accumulated during the current tick (0..1 scale).
    pub radiant: Fx,
    /// Extra radiant heat on the hand / face when it is held close to a surface.
    pub radiant_hand: Fx,
    pub radiant_mouth: Fx,
    pub sleep_pressure: Fx,
    pub asleep: bool,
    /// General wounds (bites, blows, lightning), 0..1; heals like burns.
    pub damage_body: Fx,
    /// Acute nociception from a fresh wound or cut (decays within seconds).
    pub noci_wound: Fx,
    pub noci_cut_hand: Fx,
    /// Air temperature around the body (set by the world each step).
    pub ambient_c: Fx,
    /// Core thermal state: -1 freezing .. 0 comfortable .. +1 overheated.
    pub thermal: Fx,
    /// Contact sites touched this tick (reset each step).
    pub contact_hand: bool,
    pub contact_mouth: bool,
    /// Signals computed at the end of the last step.
    pub signals: BodySignals,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BodySignals {
    pub hunger: Fx,
    pub thirst: Fx,
    pub pain: Fx,
    pub pain_hand: Fx,
    pub pain_mouth: Fx,
    pub fullness: Fx,
    /// Nutrient and water sensed in the gut (stomach contents), 0..1.
    pub gut_nutrient: Fx,
    pub gut_fluid: Fx,
    pub fatigue: Fx,
    pub body_heat: Fx,
    /// Acute nociceptor firing (thermal), separate from tonic inflammatory pain.
    pub acute_pain: Fx,
    pub reflex_hand: bool,
    pub reflex_mouth: bool,
}

impl Body {
    /// A child body. `hunger0`, `thirst0` in 0..1 set the starting deficits.
    pub fn child(age_years: i64, hunger0: Fx, thirst0: Fx) -> Body {
        let mut b = Body {
            age_s: age_years * 360 * 86_400,
            energy: k::energy_setpoint() - k::hunger_span() * hunger0,
            stomach_kcal: Fx::ZERO,
            stomach_water: Fx::ZERO,
            water: k::water_setpoint() - k::thirst_span() * thirst0,
            skin_hand: k::skin_rest_c(),
            skin_mouth: k::skin_rest_c(),
            damage_hand: Fx::ZERO,
            damage_mouth: Fx::ZERO,
            radiant: Fx::ZERO,
            radiant_hand: Fx::ZERO,
            radiant_mouth: Fx::ZERO,
            sleep_pressure: fx(0.25),
            asleep: false,
            damage_body: Fx::ZERO,
            noci_wound: Fx::ZERO,
            noci_cut_hand: Fx::ZERO,
            ambient_c: fx(20.0),
            thermal: Fx::ZERO,
            contact_hand: false,
            contact_mouth: false,
            signals: BodySignals::default(),
        };
        b.signals = b.compute_signals();
        b
    }

    /// Skin contact with something at `temp_c` for one second.
    /// `conduct` 0..1 scales how fast heat flows (metal ~1, flower ~0.3).
    pub fn contact(&mut self, site: Site, temp_c: Fx, conduct: Fx) {
        let g = k::contact_gain() * conduct;
        match site {
            Site::Hand => {
                self.skin_hand += (temp_c - self.skin_hand) * g;
                self.contact_hand = true;
            }
            Site::Mouth => {
                self.skin_mouth += (temp_c - self.skin_mouth) * g;
                self.contact_mouth = true;
            }
        }
    }

    /// Matter swallowed: it enters the stomach (fast satiation / thirst relief signals).
    pub fn ingest(&mut self, kcal: Fx, water_ml: Fx) {
        self.stomach_kcal += kcal;
        self.stomach_water += water_ml;
    }

    /// Radiant heat reaching the skin this tick (0..1 sensation scale).
    pub fn add_radiant(&mut self, flux: Fx) {
        self.radiant += flux;
    }

    /// A wound (bite, blow, lightning).
    pub fn wound(&mut self, amount: Fx) {
        self.damage_body = (self.damage_body + amount).clamp01();
        self.noci_wound = self.noci_wound.max((amount * fx(4.0)).clamp01());
    }

    /// A cut on the hand (sharp edge).
    pub fn cut_hand(&mut self, amount: Fx) {
        self.damage_hand = (self.damage_hand + amount).clamp01();
        self.noci_cut_hand = self.noci_cut_hand.max((amount * fx(6.0)).clamp01());
    }

    /// Radiant heat on one site only (hand or face held close to a surface).
    pub fn add_radiant_site(&mut self, site: Site, flux: Fx) {
        match site {
            Site::Hand => self.radiant_hand += flux,
            Site::Mouth => self.radiant_mouth += flux,
        }
    }

    /// Advance physiology by one second.
    pub fn step(&mut self) {
        self.age_s += 1;
        let rate = if self.asleep { fx(0.7) } else { Fx::ONE };
        // Metabolism and digestion.
        self.energy -= k::bmr_per_s() * rate;
        let moved = self.stomach_kcal / k::digest_tau_s();
        self.stomach_kcal -= moved;
        self.energy += moved;
        let wmoved = self.stomach_water / fx(120.0);
        self.stomach_water -= wmoved;
        self.water += wmoved;
        let heat_loss = Fx::ONE + self.radiant.clamp01() * fx(2.0);
        self.water -= k::water_loss_per_s() * heat_loss;
        if self.energy > k::energy_setpoint() + fx(200.0) {
            self.energy = k::energy_setpoint() + fx(200.0);
        }
        if self.water > k::water_setpoint() + fx(300.0) {
            self.water = k::water_setpoint() + fx(300.0);
        }

        // Skin relaxes toward rest (plus a little radiant warming) where there is no contact.
        let rest = k::skin_rest_c() + self.radiant.clamp01() * fx(12.0);
        let rest_hand = rest + self.radiant_hand.clamp01() * fx(14.0);
        let rest_mouth = rest + self.radiant_mouth.clamp01() * fx(14.0);
        if !self.contact_hand {
            self.skin_hand += (rest_hand - self.skin_hand) * k::skin_relax();
        }
        if !self.contact_mouth {
            self.skin_mouth += (rest_mouth - self.skin_mouth) * k::skin_relax();
        }

        // Thermal damage and healing.
        let heal = Fx::decay(Fx::ONE, k::heal_tau_s());
        self.damage_hand = (self.damage_hand * heal + burn(self.skin_hand)).clamp01();
        self.damage_mouth = (self.damage_mouth * heal + burn(self.skin_mouth)).clamp01();
        self.damage_body = (self.damage_body * heal).clamp01();
        // Core thermal state follows felt temperature (air + radiant heat) slowly.
        let felt = self.ambient_c + self.radiant.clamp01() * fx(30.0);
        let target = ((felt - fx(20.0)) / fx(14.0)).clamp(Fx::NEG_ONE, Fx::ONE);
        let rate = if self.asleep { fx(0.0008) } else { fx(0.0015) };
        self.thermal += (target - self.thermal) * rate;

        // Sleep pressure (Borbély process S) and sleep state.
        if self.asleep {
            self.sleep_pressure = self.sleep_pressure * Fx::decay(Fx::ONE, k::sleep_fall_tau_s());
            if self.sleep_pressure < k::sleep_offset() {
                self.asleep = false;
            }
        } else {
            let g = Fx::ONE - Fx::decay(Fx::ONE, k::sleep_rise_tau_s());
            self.sleep_pressure += (Fx::ONE - self.sleep_pressure) * g;
            if self.sleep_pressure > k::sleep_onset() && self.signals.pain < fx(0.5) {
                self.asleep = true;
            }
        }

        self.signals = self.compute_signals();
        self.noci_wound = self.noci_wound * fx(0.5);
        self.noci_cut_hand = self.noci_cut_hand * fx(0.5);
        self.radiant = Fx::ZERO;
        self.radiant_hand = Fx::ZERO;
        self.radiant_mouth = Fx::ZERO;
        self.contact_hand = false;
        self.contact_mouth = false;
    }

    fn compute_signals(&self) -> BodySignals {
        let hunger = ((k::energy_setpoint() - self.energy - self.stomach_kcal) / k::hunger_span()).clamp01();
        let thirst = ((k::water_setpoint() - self.water - self.stomach_water) / k::thirst_span()).clamp01();
        let noci_hand = noci(self.skin_hand).max(self.noci_cut_hand);
        let noci_mouth = noci(self.skin_mouth);
        let pain_hand = combine(noci_hand, self.damage_hand * fx(0.7));
        let pain_mouth = combine(noci_mouth, self.damage_mouth * fx(0.7));
        let pain = combine(combine(pain_hand, pain_mouth), (self.damage_body * fx(0.8)).max(self.noci_wound));
        let fullness = (self.stomach_kcal / fx(150.0) + self.stomach_water / fx(400.0)).clamp01();
        BodySignals {
            hunger,
            thirst,
            pain,
            pain_hand,
            pain_mouth,
            fullness,
            gut_nutrient: (self.stomach_kcal / fx(150.0)).clamp01(),
            gut_fluid: (self.stomach_water / fx(400.0)).clamp01(),
            fatigue: self.sleep_pressure,
            acute_pain: noci_hand.max(noci_mouth).max(self.noci_wound),
            body_heat: self.thermal,
            reflex_hand: noci_hand > k::reflex_threshold(),
            reflex_mouth: noci_mouth > k::reflex_threshold(),
        }
    }

    pub fn age_years(&self) -> Fx {
        Fx::ratio(self.age_s, 360 * 86_400)
    }
}

fn burn(skin_c: Fx) -> Fx {
    let over = skin_c - k::damage_threshold_c();
    if over.is_positive() {
        over * k::damage_rate()
    } else {
        Fx::ZERO
    }
}

fn noci(skin_c: Fx) -> Fx {
    ((skin_c - k::noci_threshold_c()) / k::noci_span_c()).clamp01()
}

/// Probabilistic OR of two 0..1 intensities.
fn combine(a: Fx, b: Fx) -> Fx {
    Fx::ONE - (Fx::ONE - a) * (Fx::ONE - b)
}

impl StableHash for Body {
    fn stable_hash(&self, h: &mut StateHasher) {
        self.age_s.stable_hash(h);
        self.energy.stable_hash(h);
        self.stomach_kcal.stable_hash(h);
        self.stomach_water.stable_hash(h);
        self.water.stable_hash(h);
        self.skin_hand.stable_hash(h);
        self.skin_mouth.stable_hash(h);
        self.damage_hand.stable_hash(h);
        self.damage_mouth.stable_hash(h);
        self.sleep_pressure.stable_hash(h);
        self.asleep.stable_hash(h);
        self.damage_body.stable_hash(h);
        self.thermal.stable_hash(h);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn touching_very_hot_thing_burns_and_triggers_reflex() {
        let mut b = Body::child(4, fx(0.3), fx(0.3));
        b.contact(Site::Hand, fx(700.0), Fx::ONE);
        b.step();
        assert!(b.signals.reflex_hand);
        assert!(b.damage_hand > fx(0.1), "damage {:?}", b.damage_hand);
        assert!(b.signals.pain > fx(0.8));
        // Pain lingers (inflammation) after the contact ends, then heals over days.
        for _ in 0..60 {
            b.step();
        }
        assert!(b.signals.pain > fx(0.05) && b.signals.pain < fx(0.6));
    }

    #[test]
    fn warm_but_safe_contact_does_not_hurt() {
        let mut b = Body::child(4, fx(0.3), fx(0.3));
        b.contact(Site::Hand, fx(38.0), fx(0.5));
        b.step();
        assert_eq!(b.damage_hand, Fx::ZERO);
        assert!(!b.signals.reflex_hand);
    }

    #[test]
    fn ingestion_relieves_hunger_quickly_and_thirst_with_water() {
        let mut b = Body::child(4, fx(0.6), fx(0.6));
        let h0 = b.signals.hunger;
        b.ingest(fx(30.0), fx(5.0));
        b.step();
        assert!(b.signals.hunger < h0 - fx(0.05));
        let t0 = b.signals.thirst;
        b.ingest(Fx::ZERO, fx(50.0));
        b.step();
        assert!(b.signals.thirst < t0 - fx(0.1));
    }

    #[test]
    fn hunger_builds_over_hours() {
        let mut b = Body::child(4, fx(0.1), fx(0.1));
        for _ in 0..(6 * 3600) {
            b.step();
        }
        assert!(b.signals.hunger > fx(0.6), "{:?}", b.signals.hunger);
        assert!(b.signals.thirst > fx(0.6), "{:?}", b.signals.thirst);
    }
}
