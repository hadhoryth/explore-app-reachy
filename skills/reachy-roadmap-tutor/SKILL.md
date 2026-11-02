---
name: reachy-roadmap-tutor
description: Write, restructure or fix a guide in this repo's Reachy Mini Rust roadmap (.roadmap/NN_*.html) for Ivan, who is learning Rust and robotics by writing every line himself. Use when adding the next guide, changing a guide's format, or correcting its content. Covers the page and assignment anatomy, the build tooling, verification, robot safety rules, and the Claude Fable review.
---

# Reachy Mini roadmap tutor

The roadmap is a series of single-page HTML guides in `.roadmap/`, each building on the learner's
code from the previous one. The learner is Ivan: comfortable as an engineer, new to Rust, learning
robotics and embedded design on a Reachy Mini Lite. He wants theory and design from the guide and
**to write all the code himself**. Retyping is fine; copy-paste boilerplate is not the point.

## Where things live

```
.roadmap/
├── 01_Wiggle_Reachy_Mini_antennas_from_Rust.html     older format, hand-written
├── 02_Drive_Reachy_Mini_Lite_over_USB_from_Rust.html older format: starter code with todo!()
├── 03_Model_the_whole_Reachy_Mini_in_Rust.html       current format, built
└── src/NN_Title/                                     sources of a built guide
    ├── guide.json       title, output file name, the parts in order
    ├── parts/*.html     the page body, one file per section or assignment
    ├── diagrams.py      DIAGRAMS = {"a1": fn → <figure>}, using scripts/uml.py
    ├── rust_refs.py     ENTRIES, PER_ASSIGNMENT, ASSIGNMENT_NAMES
    └── code/
        ├── solution/    the reference crate after the last assignment (Cargo.toml, src/)
        ├── stages/      main.rs and other files as they stand after particular assignments
        ├── challenges/  challenge snippets, make_challenges.py, the generated src/
        ├── files/       anything the Files section shows that isn't in the crates
        └── check_stages.py  builds every stage and prints its test count
skills/reachy-roadmap-tutor/         this skill (tracked); also works from .claude/skills/
├── SKILL.md                         this file
├── scripts/build_guide.py           builds a guide page from src/NN_Title
├── scripts/uml.py                   UML class diagrams as themed inline SVG
├── scripts/head_ik.py               head pose → six motor angles (Pollen's IK, no dependencies)
├── scripts/mujoco_check.py          settle poses in MuJoCo headless, report the head pose
├── assets/page_head.html, page_tail.html   CSS and JS shell shared by every guide
├── assets/kinematics_data.json      Pollen's Stewart platform geometry
└── reference/
    ├── reachy_mini_facts.md         verified hardware, kinematics and daemon facts, with sources
    ├── assignment_template.html     one assignment with all six parts
    └── review_brief.md              the prompt for the Claude Fable review
```

`.roadmap/` is git-ignored; the skill is tracked. Below, `<skill>` means the folder holding this
file: `skills/reachy-roadmap-tutor`, or `.claude/skills/reachy-roadmap-tutor` if it was moved
there so Claude Code loads it automatically. Remote Cowork sessions can't write inside `.claude/`
on the learner's computer; if the skill lives there, edits go through the learner. The project notes in the attached Claude project
(`claude/roadmap-guide-NN-notes.md`) summarize each guide for later sessions.

## Page anatomy (in this order)

1. **Hero, two columns.** Left: title, lede, the three `cargo run` commands, "Start with
   assignment 0". Right: the **How this guide works** card (`id="how"`) listing the six parts of
   an assignment and saying that Rust only appears from Files down.
2. **What you're building**: the architecture as `.paths` box diagrams, and the end result.
3. **Before you start**: prerequisites for the whole guide, the builds-at-every-step note, safety notes.
4. **The ideas**: **Design rules only**: numbered principles, each tagged with the assignments
   that build it. Theory belongs inside assignments, not here.
5. **Assignments** 0 to N, each with the six parts below, then **Challenges** (no diagrams, no
   tests, solutions at the bottom).
6. **Files: tests and data** (`id="files"`): `details.files` blocks with copy buttons, one per
   test module or data block, each saying exactly where it goes. All tests are given; never ask
   the learner to write tests.
7. **Solutions** (`details.solution`, no copy buttons), one per assignment and challenge, each
   with "why" notes on the choices the design didn't force.
8. **The complete program**: every file, folded.
9. **Rust index** (`{{RUSTINDEX}}`), **Troubleshooting**, **Next steps**, footer with crate
   versions and sources.

## Assignment anatomy (every assignment, same order)

Use `<skill>/reference/assignment_template.html`.

1. **Prerequisites**: `ul.prereq`: the previous assignment's test count, files, hardware.
2. **Theory**: the robotics and embedded ideas from first principles, with the real numbers and
   their sources. Algorithms as **pseudocode** (`div.code.pseudo`, `data-lang="text"`), written
   algorithmically (←, for, if, return), lines ≤ 78 characters. This is where the learning is:
   explain why, not only what.
3. **Design**: a UML class diagram (`{{UML:aN}}`) with the public names, fields and signatures the
   tests rely on; orange border for what this assignment builds, dashed for test fakes. Show
   `{{LEGEND}}` the first time. Below it, what the diagram can't say, and what is left to the
   learner. Don't show private helpers unless tests use them.
4. **Rust you'll need** (`{{RUST:aN}}`): entries from `rust_refs.py`. Examples must be about
   *other* things (temperatures, colors, configs) so they teach the shape without giving the
   solution line. An entry is shown in full in the first assignment that lists it; later ones link back.
5. **Task**: numbered steps in words, pointing at the pseudocode and the diagram. Data the learner
   can't derive (head poses) goes in a table here and as a copy block in Files.
6. **Check**: which Files blocks to copy, where, and the exact `test result: ok. K passed` line.
   Add run instructions when there is something to watch.

Hard rule: **no Rust above `<section id="files">`**. The builder refuses to write a page that has a
`data-lang="rust"` block there. Inline `<code>` for names is fine.

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
  found on with the head up; `Drop` cuts torque only as the last resort.
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
4. **Write the parts**: `10_top`, one file per assignment, challenges, `30_files`,
   `40_solutions`, `50_end`. Diagrams in `diagrams.py`, Rust entries in `rust_refs.py`. Inject
   code with markers so the page can't drift from the verified code:
   `{{FILE:p}}`, `{{HEAD:p}}` (up to the tests), `{{ITEM:p::pattern||pattern}}` (`@last` for the
   last match), `{{TESTS:p::name,name}}`, `{{RANGE:p::start::end}}`, `{{UML:key}}`, `{{LEGEND}}`,
   `{{RUST:key}}`, `{{RUSTINDEX}}`.
5. **Build**: `python3 <skill>/scripts/build_guide.py .roadmap/src/NN_Title`
   (writes `.roadmap/NN_Title.html`). It checks markers, duplicate ids, dead links and Rust above Files.
6. **Render-check** with Playwright at 1280 px and 390 px, light and dark: no page-level
   horizontal scroll, diagrams legible, every `#f…` link opens its block.
7. **Review**: hand the page, the crates and the learner's code to Claude Fable using
   `reference/review_brief.md`. Verify each finding before acting on it.
8. **Fix, then a best-practice pass** against the principles above; rerun steps 3, 5 and 6.
9. **Deliver**: the page into `.roadmap/`, the sources into `.roadmap/src/NN_Title/`, notes into
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
