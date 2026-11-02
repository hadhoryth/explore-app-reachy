---
name: reachy-roadmap-tutor
description: Write, restructure or fix a guide in this repo's Reachy Mini Rust roadmap (.roadmap/NN_*.html) for Ivan, who is learning Rust and robotics by writing every line himself. Use when adding the next guide, changing a guide's format, or correcting its content. Covers the page and assignment anatomy, the build tooling, verification, robot safety rules, and the Claude Fable review.
---

# Reachy Mini roadmap tutor

The roadmap is a series of HTML guides in `.roadmap/` (from guide 03 on, one page per assignment), each building on the learner's
code from the previous one. The learner is Ivan: comfortable as an engineer, new to Rust, learning
robotics and embedded design on a Reachy Mini Lite. He wants theory and design from the guide and
**to write all the code himself**. Retyping is fine; copy-paste boilerplate is not the point.

## Where things live

```
.roadmap/
├── 01_Wiggle_Reachy_Mini_antennas_from_Rust.html     older format, hand-written
├── 02_Drive_Reachy_Mini_Lite_over_USB_from_Rust.html older format: starter code with todo!()
├── 03_Model_the_whole_Reachy_Mini_in_Rust/          current format, built: index.html, a0.html …
│                                                     a8.html, challenges.html, reference.html
├── 03_Model_the_whole_Reachy_Mini_in_Rust.html       redirect to the folder (keeps old #anchors)
└── src/NN_Title/                                     sources of a built guide
    ├── guide.json       title, short name, output folder, the pages and their parts
    ├── parts/*.html     the page body, one file per section or assignment
    ├── diagrams.py      DIAGRAMS = {"a1": fn → <figure>}, using scripts/uml.py
    ├── rust_refs.py     ENTRIES, PER_ASSIGNMENT, ASSIGNMENT_NAMES
    └── code/
        ├── solution/    the reference crate after the last assignment (Cargo.toml, src/)
        ├── stages/      main.rs and other files as they stand after particular assignments
        ├── challenges/  challenge snippets, make_challenges.py, the generated src/
        ├── files/       anything a copy block shows that isn't in the crates
        └── check_stages.py  builds every stage and prints its test count
skills/reachy-roadmap-tutor/         this skill (tracked); also works from .claude/skills/
├── SKILL.md                         this file
├── scripts/build_guide.py           builds a guide’s pages from src/NN_Title
├── scripts/uml.py                   UML class diagrams as themed inline SVG
├── scripts/head_ik.py               head pose → six motor angles (Pollen's IK, no dependencies)
├── scripts/mujoco_check.py          settle poses in MuJoCo headless, report the head pose
├── scripts/to_text.py               built pages → plain text for the student review
├── assets/page_head.html, page_tail.html   CSS and JS shell shared by every guide
├── assets/kinematics_data.json      Pollen's Stewart platform geometry
└── reference/
    ├── reachy_mini_facts.md         verified hardware, kinematics and daemon facts, with sources
    ├── assignment_template.html     one assignment page body
    ├── student_review.md            how to run the learner-agent review, with prompts
    └── review_brief.md              the prompt for the Claude Fable review
```

`.roadmap/` is git-ignored; the skill is tracked. Below, `<skill>` means the folder holding this
file: `skills/reachy-roadmap-tutor`, or `.claude/skills/reachy-roadmap-tutor` if it was moved
there so Claude Code loads it automatically. Remote Cowork sessions can't write inside `.claude/`
on the learner's computer; if the skill lives there, edits go through the learner. The project notes in the attached Claude project
(`claude/roadmap-guide-NN-notes.md`) summarize each guide for later sessions.

## Site anatomy (one page per assignment; Ivan, 2026-11-02)

Every page has the **step bar** on top (Start, 0 … 8, C, Ref; the current one filled) and
**previous / next** links at the bottom. Assignment pages have the **Rust panel** on the right:
"Rust you'll need" for that assignment, "New here" first, then "From earlier" (tagged "from N"),
every entry folded so the names read as a list. Under 1120 px the panel becomes a drawer behind a
**Rust** button. The builder adds all of this from `guide.json`; parts hold only page bodies.

1. **Start page** (`index.html`): hero with the **How this guide works** card (`id="how"`), What
   you're building, Before you start (prerequisites, builds-at-every-step note, safety), Design
   rules only (numbered, each tagged with its assignments), then `{{PAGES}}`: one card per
   assignment page with the first sentence of its goal.
2. **Assignment pages** `a0.html` … : one assignment each, anatomy below.
3. **Challenges** (`challenges.html`): no diagrams, no tests; each challenge ends with its folded
   solution.
4. **Reference** (`reference.html`): the complete program (every file, folded), the Rust index
   (`{{RUSTINDEX}}`), Troubleshooting, Next steps, footer with crate versions and sources.

The reading area is 19px (17px on phones); inside `article`, size things in `em`, not `rem`.
Links: write `href="#id"`; the builder points it at the page that has the id. Links to other
guides (`02_….html`) get `../` added.

## Assignment anatomy (every assignment page, same order)

Use `<skill>/reference/assignment_template.html`.

1. **Goal**: `div.goal`, right after the title and the learn chips. One or two sentences on what
   this assignment achieves and why it matters, then "When you're done:" and 3 or 4 concrete
   outcomes, the last one the test count (or what to watch). Keep outcomes literally true.
2. **Prerequisites**: `ul.prereq`: the previous count, files, hardware, and last a
   `li.have` "**You already have**: …" naming the earlier API **in real Rust**
   (`joints.clamped(spec)`, `Joints::from_array(..)`), since pseudocode writes them as plain
   functions. Students asked for this more than anything else.
3. **The idea** (multi-part) or **Theory** (single-part). The idea: one or two short paragraphs
   on why the assignment exists (a table helps: A1's "what the robot is / where it is"), then
   `ol.path`: one line per part, linked to `#aN-pK`, ending in
   `<span class="count">→ cargo test: K passing in all</span>`.
4. **Parts** (`div.part#aN-pK`, `h4.part-title` with "Part K of M" in `span.part-n`, then a title
   naming the file), each with `h5` **Theory** (only the theory this part needs, with
   pseudocode) → **Design** (one diagram per part) → **Task** → **Check**. Single-part
   assignments use plain `h4` Theory, Design, Task, Check.
   - Pseudocode: algorithmic (←, for, if, return), lines ≤ 78 characters, ranges spelled out
     ("for i from 0 to 8", "for n from 1 to N, N included"), never Rust-like `a .. b`.
   - Design: public names, fields and signatures the tests use, **and the private helpers,
     constants and fields the solution uses** (`-` lines; the tests don't call them, but the
     learner writes them). In `uml.py` `=` is drawn public and underlined (class-scope), `-=`
     private and underlined. Name what the diagram can't say; give a "three names, three things"
     note when names collide (`joints` module, `spec.joints()`, `Joints`).
   - Task: numbered steps in words. Data the learner can't derive goes in a table with a folded
     `details.files` copy block under it. Say which type a parameter should be when the obvious
     one fails (ticks as `i32`).
   - **The solution holds nothing the Design and Task didn't announce** (Ivan, 2026-11-02, after
     finding `Limits::symmetric`, `from_ticks` and `TAU` only in the A1 solution): every item,
     every std import (`std::f64::consts::TAU`), every attribute and derive. Say *why* where it
     first appears: `#[must_use]` ("`limits.clamp(angle);` alone leaves `angle` unchanged; the
     compiler warns"), `const fn` (a `static` is built at compile time), the derives (`assert_eq!`
     needs `PartialEq` and `Debug`). `code/audit_tasks.py` lists the leftovers; run it after
     every build.
   - Check: what to copy and where, the `details.files` blocks, then the exact
     `test result: ok. K passed` line; say how K adds up when old tests stay ("49 + 5 + 3").
     When a part's tests need many items before they compile, add a `p.tip`: write signatures
     with `todo!()` bodies first, then watch tests turn green one by one.
5. **Solution**: `<h4>Solution</h4>` and one folded `details.solution` (no copy buttons), summary
   "open it after your tests pass", with "why" notes on the choices the design didn't force.

Parts exist so Ivan can test each piece as soon as he has written it. Order parts so each
compiles on its own (a stub another part fills, a door that re-exports only what exists), make
every part's count a stage in `check_stages.py` (`A1.1`, `A4.2`, …) built from small files in
`code/stages/`, and inject tests pasted into an existing module with `RANGE` and an exact place
("inside mod tests, right below use super::*;").

**Student review** (Ivan, 2026-11-02): before delivering a new or restructured guide, convert the
pages to text (`reference/student_review.md` says how, with the prompts) and have
subagents read them as a basic-Rust learner, answer "in your own words" questions about every
type and why it exists, quote every confusing sentence, and rate each part. Fix what they
misunderstand, verify their technical claims first (one found a real torque bug in A7), then
re-check the weakest pages with a fresh student.

Hard rule: **every Rust code block is folded**: a `data-lang="rust"` block may only appear inside a
`<details>` (tests and data to copy, solutions, the complete program). The builder refuses to
write a page with an unfolded one. Inline `<code>` for names is fine.

## Writing style

- Second person, present tense, short sentences, curly quotes and apostrophes (’ “ ”).
- Bold lead-in phrase per theory paragraph: `<strong>Commanded state.</strong> …`.
- State facts with their source (Pollen file, ROBOTIS manual, MuJoCo check). Say what was verified
  and how; say what is a choice of the guide.
- No hype, no filler, no "simply". Safety warnings are concrete: what moves, what drops, what to do.
- Inline code in prose is escaped by hand (`&lt;`, `&gt;`, `&amp;`); code blocks with
  `data-lang` or `class="tree"` are written raw and escaped by the builder.

## Robot code principles (what solutions must follow)

- Plain `Copy` values for state; only backends do I/O. Generic over the `Backend` trait so tests
  use fakes.
- The robot is described once, as data: `REACHY_MINI_LITE` (a `ReachyMiniSpec` of `JointSpec`s:
  limits, motor ID, gain, simulator name and sign) holds every fixed fact; `Joints` holds only
  state. Backends are built from the description, never from their own tables. Bus order defined
  once; arrays sized by type; conversions only at the edges.
- Invariants protected by privacy (e.g. `Head`'s private field: only reachable poses).
- Files grouped by role, not by body part (since guide 03, at Ivan's request): `model/` (spec.rs,
  joints.rs) and `backend/` (sim.rs, device.rs) are folders with one door each, `model.rs` and
  `backend.rs`, which name their files with private `mod` and re-export with `pub use` only what
  other modules use (an unused re-export warns). Everyone imports through the door
  (`crate::model::Joints`), files inside the folder included. Single ideas stay single files
  (motion.rs, reachy.rs, routines.rs). Head, body and antennas are types, never folders. Folder
  style is `name.rs` + `name/`, not `mod.rs`. Until the last assignment the stage `main.rs`
  carries `#![allow(dead_code, unused_imports)]`, because doors re-export before anything uses
  the names. Recorded moves belong in data files (assets/moves/, serde), not Rust arrays.
- Validate at every boundary: clamp targets, limit speed (`MAX_SPEED`), refuse NaN, clamp again
  in the device, rely on EEPROM limits as the last net.
- Deadline-based fixed-rate loop (`Ticker`), no drift, report overruns, don't burst.
- No `unwrap`/`expect`/panicking indexing outside tests: errors reach code that can still land
  the robot; a panic only reaches `Drop`.
- Category 1 stops: land (sleep posture) before torque off, on every exit path; never cut torque
  found on with the head up; `Drop` cuts torque only as the last resort, and only torque this
  program turned on (`owns_torque`, set by `enable`), never torque `open` merely found on.
- Every given test must be deterministic and robust under load: loop timing tests 40+ times
  with parallel load before shipping them.

## Workflow for a new guide

1. **Read the learner's state** through the device bridge: the previous guide, his current
   `src/`, `git status`/`git log`, `Cargo.toml`, `target/.rustc_info.json` for his rustc. Look for
   bugs in his code; fix them in assignment 0 with a test that exposes each one.
2. **Research** facts in Pollen's sources (see `reference/reachy_mini_facts.md`); add new
   verified facts there. Compute head poses with `scripts/head_ik.py`; check them in MuJoCo with
   `scripts/mujoco_check.py` on the learner's computer (`pip install --user mujoco` in the device
   shell; the meshes are only there).
3. **Write the reference crate first**, in the cloud workspace: start from his code, build the
   final state, all tests, `cargo clippy --all-targets` and `cargo fmt --check` clean. Then the
   stage files, and `code/check_stages.py` to prove every stage compiles with its stated count.
   Write challenge solutions as snippets applied by `make_challenges.py`, and test them too.
4. **Write the parts**: `10_top`, one file per assignment (goal to solution, tests and data
   inside), challenges, `45_reference`, `50_end`. Diagrams in `diagrams.py`, Rust entries in `rust_refs.py`. Inject
   code with markers so the page can't drift from the verified code:
   `{{FILE:p}}`, `{{HEAD:p}}` (up to the tests), `{{ITEM:p::pattern||pattern}}` (`@last` for the
   last match), `{{TESTS:p::name,name}}`, `{{RANGE:p::start::end}}`, `{{UML:key}}`, `{{LEGEND}}`,
   `{{RUSTINDEX}}`, `{{PAGES}}`. The Rust panel comes from the page's `"rust"` key in guide.json.
   Copy `code/audit_tasks.py` from guide 03 and point its table at the new stage files.
5. **Build**: `python3 <skill>/scripts/build_guide.py .roadmap/src/NN_Title`
   (writes `.roadmap/NN_Title/*.html`, plus a redirect at `.roadmap/NN_Title.html`). Per page it
   checks markers, duplicate ids, dead links (across pages) and unfolded Rust. Then run
   `python3 code/audit_tasks.py`: every name it lists must be explained on the page, or be kept
   unchanged from an earlier assignment or guide.
6. **Render-check** every page with Playwright at 1280 px and 390 px, light and dark, plus the
   Rust drawer on a phone width: no page-level
   horizontal scroll, diagrams legible, every `#f…` link opens its block.
7. **Student review** (`reference/student_review.md`), then **review**: hand the pages, the crates and the learner's code to Claude Fable using
   `reference/review_brief.md`. Verify each finding before acting on it.
8. **Fix, then a best-practice pass** against the principles above; rerun steps 3, 5 and 6.
9. **Deliver**: the pages into `.roadmap/NN_Title/`, the sources into `.roadmap/src/NN_Title/`, notes into
   the Claude project. Tell the learner what changed in his own code (assignment 0) in one line each.

To change an existing built guide: edit its `src/NN_Title/` files, rebuild, re-check. If code
changes, rerun the crate tests and `check_stages.py`, and update the counts in the Check parts.

## Pitfalls already hit

- Integer division in progress (`tik / ticks` on `u32`): moves sat still, then jumped. Test whole
  trajectories, not just their ends.
- `a + (b − a) × 1.0` can miss `b`; `(1 − t)a + tb` makes still joints drift. Special-case `t ≥ 1`.
- `f64::clamp(NaN)` is NaN and `NaN as i32` is 0 → −π in rustypot. Refuse non-finite commands.
- Sim antennas: both negated. The learner's guide 02 code negated only one.
- A `const` atomic in tests is a fresh value at each use (clippy `declare_interior_mutable_const`);
  use `static`. Tests run in parallel: never set a shared static in a test; inject flags instead.
- Timing tests with per-tick assertions were flaky under load; assert only the overall window.
- Pseudocode lines over ~78 characters scroll; keep them short.
- Marker arguments are split on `::`, so a `RANGE`/`ITEM` pattern can't contain `::` (e.g. use
  `Error;` rather than `use std::error::Error;` as an end pattern).
- `check_stages.py` builds (`cargo build`) as well as tests each stage: unused re-exports only
  warn in the non-test build.
- `Device::open` cutting torque would drop a head another program left up; read the state instead.
