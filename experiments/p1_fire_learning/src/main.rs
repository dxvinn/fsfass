//! P1 runner. `full` runs the protocol and writes the report; `smoke <seed>`,
//! `hourly <seed>` and `trace <seed> <tick> <n>` are inspection tools.

use alife_core::Fx;
use p1_fire_learning::*;
use std::fmt::Write as _;

const SEEDS: u64 = 60;
const OUT: &str = "experiments/p1_fire_learning/results";

fn smoke(seed: u64) {
    let (mind, log, end) = run_learning(seed, true);
    println!("{log:#?}");
    let naive = new_mind(seed);
    for p in Probe::ALL {
        let e = run_probe(&mind, p, seed, end, false);
        let n = run_probe(&naive, p, seed, end, false);
        println!(
            "{:<14} experienced: contact {} lat {:?} min_d {} adj {} insp {} reaches {} aborted {} pav {:.2} touch {:.2} fear {:.2} | naive: contact {} lat {:?} min_d {} reaches {}",
            p.name(), e.contact, e.latency, e.min_dist, e.ticks_adjacent, e.inspects, e.reaches, e.aborted_reaches,
            e.pav_pain_first.to_f64(), e.touch_pain_first.to_f64(), e.max_fear.to_f64(),
            n.contact, n.latency, n.min_dist, n.reaches
        );
    }
}

#[derive(Default, Clone)]
struct Agg {
    n: u32,
    contact: u32,
    lat_sum: u64,
    lat_n: u32,
    min_dist_sum: i64,
    adj_sum: u64,
    reaches: u64,
    aborted: u64,
    inspects: u64,
    pav_sum: f64,
    touch_sum: f64,
    fear_sum: f64,
    ingest: u32,
    ingest_lat_sum: u64,
    ingestions: u64,
}

impl Agg {
    fn add(&mut self, r: &ProbeResult) {
        self.n += 1;
        if r.contact {
            self.contact += 1;
        }
        if let Some(l) = r.latency {
            self.lat_sum += l;
            self.lat_n += 1;
        }
        self.min_dist_sum += r.min_dist as i64;
        self.adj_sum += r.ticks_adjacent;
        self.reaches += r.reaches as u64;
        self.aborted += r.aborted_reaches as u64;
        self.inspects += r.inspects as u64;
        self.pav_sum += r.pav_pain_first.to_f64();
        self.touch_sum += r.touch_pain_first.to_f64();
        self.fear_sum += r.max_fear.to_f64();
        if r.ingested {
            self.ingest += 1;
            self.ingest_lat_sum += r.ingest_latency.unwrap_or(PROBE_TICKS);
        }
        self.ingestions += r.ingestions as u64;
    }
    fn rate(&self) -> f64 {
        self.contact as f64 / self.n.max(1) as f64
    }
    fn mean_lat(&self) -> f64 {
        if self.lat_n == 0 { f64::NAN } else { self.lat_sum as f64 / self.lat_n as f64 }
    }
    fn mean(&self, x: f64) -> f64 {
        x / self.n.max(1) as f64
    }
    /// Mean latency to first ingestion, counting "never" as the full probe.
    fn ingest_latency_all(&self) -> f64 {
        (self.ingest_lat_sum + (self.n - self.ingest) as u64 * PROBE_TICKS) as f64 / self.n.max(1) as f64
    }
}

fn full() {
    std::fs::create_dir_all(format!("{OUT}/traces")).unwrap();
    let mut exp_burned: Vec<Agg> = vec![Agg::default(); 6];
    let mut naive_burned: Vec<Agg> = vec![Agg::default(); 6];
    let mut exp_all: Vec<Agg> = vec![Agg::default(); 6];
    let mut naive_all: Vec<Agg> = vec![Agg::default(); 6];
    let mut csv = String::from("seed,burned,fire_contacts_hand,fire_contacts_mouth,food_ingestions,water_ingestions,probe,group,contact,latency,min_dist,ticks_adjacent,reaches,aborted,inspects,pav_pain_first,touch_pain_first,max_fear,ingest_latency,ingestions\n");
    let mut learn_rows = Vec::new();
    let mut rep_seed = None;
    for seed in 1..=SEEDS {
        let capture = rep_seed.is_none();
        let (mind, log, end) = run_learning(seed, capture);
        let naive = new_mind(seed);
        let burned = log.burned();
        if burned && rep_seed.is_none() {
            rep_seed = Some(seed);
            if let Some(t) = &log.burn_trace {
                std::fs::write(format!("{OUT}/traces/1_first_burn_seed{seed}.txt"), t).unwrap();
            }
        }
        for (pi, p) in Probe::ALL.iter().enumerate() {
            let cap = rep_seed == Some(seed) && burned;
            let e = run_probe(&mind, *p, seed, end, cap);
            let n = run_probe(&naive, *p, seed, end, false);
            if cap {
                let name = p.name().replace(' ', "_").to_lowercase();
                let mut txt = String::new();
                writeln!(txt, "=== EXPERIENCED CHILD (seed {seed}) — FIRST SIGHT OF {} IN A NEW ROOM ===", p.name()).unwrap();
                txt.push_str(&e.first_trace.clone().unwrap_or_default());
                if let Some(k) = &e.key_trace {
                    writeln!(txt, "\n=== MOMENT A REACH WAS UNDER WAY (abort or follow through?) ===").unwrap();
                    txt.push_str(k);
                }
                std::fs::write(format!("{OUT}/traces/2_probe_{}_{name}.txt", pi + 1), txt).unwrap();
            }
            exp_all[pi].add(&e);
            naive_all[pi].add(&n);
            if burned {
                exp_burned[pi].add(&e);
                naive_burned[pi].add(&n);
            }
            for (g, r) in [("experienced", &e), ("naive", &n)] {
                writeln!(
                    csv,
                    "{seed},{burned},{},{},{},{},{},{g},{},{},{},{},{},{},{},{:.4},{:.4},{:.4},{},{}",
                    log.fire_contacts_hand, log.fire_contacts_mouth, log.food_ingestions, log.water_ingestions,
                    p.name(), r.contact as u8, r.latency.map(|x| x.to_string()).unwrap_or_default(), r.min_dist,
                    r.ticks_adjacent, r.reaches, r.aborted_reaches, r.inspects, r.pav_pain_first.to_f64(),
                    r.touch_pain_first.to_f64(), r.max_fear.to_f64(),
                    r.ingest_latency.map(|x| x.to_string()).unwrap_or_default(), r.ingestions
                )
                .unwrap();
            }
        }
        eprintln!("seed {seed}: burned {burned} (hand {}, mouth {}), food {}, water {}", log.fire_contacts_hand, log.fire_contacts_mouth, log.food_ingestions, log.water_ingestions);
        learn_rows.push(log);
    }
    std::fs::write(format!("{OUT}/p1_raw.csv"), &csv).unwrap();

    // Criteria (fixed in docs/PHASE2_PLAN.md before results existed).
    let e = |pi: usize| naive_burned[pi].rate() - exp_burned[pi].rate();
    let (fa, fb, hm, fl, rk, fd) = (0, 1, 2, 3, 4, 5);
    let c1 = e(fa) >= 0.5;
    let c2 = e(fb) >= 0.5 * e(fa);
    let hm_rate_diff = e(hm);
    let hm_adj_exp = exp_burned[hm].mean(exp_burned[hm].adj_sum as f64);
    let hm_adj_naive = naive_burned[hm].mean(naive_burned[hm].adj_sum as f64);
    let c3 = hm_rate_diff >= 0.2 || (hm_adj_naive > 0.0 && hm_adj_exp <= 0.7 * hm_adj_naive);
    let c4 = e(fl) < e(fa) && exp_burned[rk].rate() >= 0.7 * naive_burned[rk].rate() && exp_burned[fd].rate() >= 0.7 * naive_burned[fd].rate();
    let c5 = exp_burned[fd].ingest_latency_all() < naive_burned[fd].ingest_latency_all();
    let burned_n = learn_rows.iter().filter(|l| l.burned()).count();

    let mut r = String::new();
    writeln!(r, "# P1 — Child and fire: results\n").unwrap();
    writeln!(r, "Generated by `cargo run --release -p p1-fire-learning -- full`. {SEEDS} seeds. Raw data: `p1_raw.csv`. Traces: `traces/`.\n").unwrap();
    writeln!(r, "## Learning phase (2 game days, room with FIRE A, ROCK, FOOD, WATER)\n").unwrap();
    let mean = |f: &dyn Fn(&LearnLog) -> f64| learn_rows.iter().map(f).sum::<f64>() / learn_rows.len() as f64;
    writeln!(r, "| Measure | Value |\n|---|---|").unwrap();
    writeln!(r, "| Children burned by fire at least once | {burned_n}/{SEEDS} |").unwrap();
    writeln!(r, "| …first burn by hand / by mouth (count of children) | {} / {} |", learn_rows.iter().filter(|l| l.fire_contacts_hand > 0).count(), learn_rows.iter().filter(|l| l.fire_contacts_mouth > 0 && l.fire_contacts_hand == 0).count()).unwrap();
    writeln!(r, "| Mean fire contacts per child (2 days) | {:.2} |", mean(&|l| l.fire_contacts as f64)).unwrap();
    writeln!(r, "| Mean painful fire events per child | {:.2} |", mean(&|l| l.fire_pain_events as f64)).unwrap();
    writeln!(r, "| Mean food portions eaten / water sips (2 days) | {:.1} / {:.1} |", mean(&|l| l.food_ingestions as f64), mean(&|l| l.water_ingestions as f64)).unwrap();
    writeln!(r, "| Mean hunger / thirst at end | {:.2} / {:.2} |", mean(&|l| l.final_hunger.to_f64()), mean(&|l| l.final_thirst.to_f64())).unwrap();
    writeln!(r, "| Mean time asleep | {:.1} h of 48 h |", mean(&|l| l.sleep_ticks as f64) / 3600.0).unwrap();
    writeln!(r, "| Mean concepts formed / episodes stored / beliefs crystallised | {:.1} / {:.1} / {:.1} |", mean(&|l| l.concepts as f64), mean(&|l| l.episodes as f64), mean(&|l| l.beliefs as f64)).unwrap();
    writeln!(r, "| Mind memory per child (approx.) | {:.0} KB |", mean(&|l| l.mind_bytes as f64) / 1024.0).unwrap();
    writeln!(r, "| Mind CPU per cognitive step (1 step = 1 game s) | {:.1} µs |", mean(&|l| l.step_ns as f64) / 1000.0).unwrap();
    writeln!(r, "\n## Probes (new room, new object instances, new tokens; brain preserved). Burned group n = {burned_n}\n").unwrap();
    writeln!(r, "Contact = touched or mouthed within 10 game minutes. E = contact rate(naive) − contact rate(experienced).\n").unwrap();
    writeln!(r, "| Probe | Naive contact | Experienced contact | E | Exp. mean latency (s) | Exp. reaches / aborted | Exp. inspections | Exp. mean s adjacent | Naive mean s adjacent | Gut pain prediction at first sight (exp.) | Contact pain prediction (exp.) | Max fear (exp.) |").unwrap();
    writeln!(r, "|---|---|---|---|---|---|---|---|---|---|---|---|").unwrap();
    for (pi, p) in Probe::ALL.iter().enumerate() {
        let x = &exp_burned[pi];
        let n = &naive_burned[pi];
        writeln!(
            r,
            "| {} | {:.2} | {:.2} | {:+.2} | {:.0} | {} / {} | {:.1} | {:.0} | {:.0} | {:.2} | {:.2} | {:.2} |",
            p.name(), n.rate(), x.rate(), e(pi), x.mean_lat(), x.reaches, x.aborted, x.mean(x.inspects as f64),
            x.mean(x.adj_sum as f64), n.mean(n.adj_sum as f64), x.mean(x.pav_sum), x.mean(x.touch_sum), x.mean(x.fear_sum)
        )
        .unwrap();
    }
    writeln!(r, "\nFOOD probe (hunger 0.5): mean seconds to first swallow — experienced {:.0}, naive {:.0}; portions eaten in 10 min — experienced {:.1}, naive {:.1}.", exp_burned[fd].ingest_latency_all(), naive_burned[fd].ingest_latency_all(), exp_burned[fd].mean(exp_burned[fd].ingestions as f64), naive_burned[fd].mean(naive_burned[fd].ingestions as f64)).unwrap();
    writeln!(r, "\n### All children (including never-burned), contact rates\n").unwrap();
    writeln!(r, "| Probe | Naive | Experienced |\n|---|---|---|").unwrap();
    for (pi, p) in Probe::ALL.iter().enumerate() {
        writeln!(r, "| {} | {:.2} | {:.2} |", p.name(), naive_all[pi].rate(), exp_all[pi].rate()).unwrap();
    }
    writeln!(r, "\n## Criteria (fixed before running)\n").unwrap();
    writeln!(r, "| Criterion | Measured | Result |\n|---|---|---|").unwrap();
    let pf = |b: bool| if b { "PASS" } else { "FAIL" };
    writeln!(r, "| C1 learning: E(FIRE A) ≥ 0.5 | E = {:+.2} | {} |", e(fa), pf(c1)).unwrap();
    writeln!(r, "| C2 not one object: E(FIRE B) ≥ 0.5·E(FIRE A) | E(B) = {:+.2} vs {:.2} | {} |", e(fb), 0.5 * e(fa), pf(c2)).unwrap();
    writeln!(r, "| C3 heat concept: HOT METAL rate diff ≥ 0.2 or ≥30% less time adjacent | rate diff {:+.2}; adjacent {:.0}s vs naive {:.0}s | {} |", hm_rate_diff, hm_adj_exp, hm_adj_naive, pf(c3)).unwrap();
    writeln!(r, "| C4 no blanket fear: E(FLOWER) < E(FIRE A); ROCK & FOOD contact ≥ 0.7×naive | E(flower) {:+.2}; rock {:.2} vs {:.2}; food {:.2} vs {:.2} | {} |", e(fl), exp_burned[rk].rate(), naive_burned[rk].rate(), exp_burned[fd].rate(), naive_burned[fd].rate(), pf(c4)).unwrap();
    writeln!(r, "| C5 learned eating: hungry experienced child swallows FOOD sooner than naive | {:.0}s vs {:.0}s | {} |", exp_burned[fd].ingest_latency_all(), naive_burned[fd].ingest_latency_all(), pf(c5)).unwrap();
    std::fs::write(format!("{OUT}/p1_report.md"), &r).unwrap();
    print!("{r}");
}

/// Hourly behaviour log for one child (debugging aid).
pub fn hourly(seed: u64) {
    use alife_world::sensors::sense;
    let mut w = learning_world(seed);
    let mut m = new_mind(seed);
    let mut hist = std::collections::BTreeMap::<String, u32>::new();
    while w.tick < LEARN_TICKS {
        let f = sense(&w, 0);
        let cmd = m.step(&f);
        let label = format!("{:?}", cmd).split_whitespace().next().unwrap_or("").to_string();
        let tgt = cmd.target().and_then(|t| w.objects.iter().find(|o| w.token_for(w.agents[0].id, o.id) == t)).map(|o| o.props.label).unwrap_or("-");
        *hist.entry(format!("{label} {tgt}")).or_default() += 1;
        w.step(&[cmd]);
        if w.tick % 3600 == 0 {
            let b = &w.agents[0].body;
            println!(
                "h{:>3} hunger {:.2} thirst {:.2} pain {:.2} S {:.2} asleep {} dmg {:.2}/{:.2} fear {:.2} | {:?}",
                w.tick / 3600, b.signals.hunger.to_f64(), b.signals.thirst.to_f64(), b.signals.pain.to_f64(),
                b.sleep_pressure.to_f64(), b.asleep, b.damage_hand.to_f64(), b.damage_mouth.to_f64(), m.affect.fear.to_f64(),
                hist.iter().filter(|(_, v)| **v > 30).collect::<Vec<_>>()
            );
            hist.clear();
        }
    }
}

/// Print brain traces for ticks [from, from+n).
pub fn trace_at(seed: u64, from: u64, n: u64) {
    use alife_world::sensors::sense;
    let mut w = learning_world(seed);
    let mut m = new_mind(seed);
    while w.tick < from + n {
        let f = sense(&w, 0);
        m.trace_enabled = w.tick >= from;
        let cmd = m.step(&f);
        if w.tick >= from {
            if let Some(t) = &m.last_trace {
                println!("{}", t.to_text());
            }
        }
        w.step(&[cmd]);
    }
}

fn main() {
    let _ = Fx::ZERO;
    let args: Vec<String> = std::env::args().collect();
    let num = |i: usize, d: u64| args.get(i).and_then(|s| s.parse().ok()).unwrap_or(d);
    match args.get(1).map(|s| s.as_str()) {
        Some("full") => full(),
        Some("smoke") => smoke(num(2, 1)),
        Some("hourly") => hourly(num(2, 1)),
        Some("trace") => trace_at(num(2, 1), num(3, 0), num(4, 3)),
        _ => eprintln!("usage: p1_fire_learning full | smoke <seed> | hourly <seed> | trace <seed> <tick> <n>"),
    }
}
