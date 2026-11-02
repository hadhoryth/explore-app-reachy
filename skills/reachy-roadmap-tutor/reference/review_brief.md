# Review brief for Claude Fable

Hand the draft to a subagent running Claude Fable (`model: fable`) once the page builds and the
reference crate passes. Fill in the bracketed parts. Give it the code and the page, not your own
conclusions, and ask for findings only: you apply the fixes.

---

You are reviewing a draft Rust robotics tutorial before it goes to a learner. Be a demanding,
independent reviewer: I want problems found, not reassurance. Do NOT edit any of the files listed
below; experiment only in copies under /tmp/review. Report findings back to me.

## Context
The learner, Ivan, is building step by step a Rust program that drives a Pollen Robotics Reachy
Mini Lite (9 Dynamixel motors over USB) in the MuJoCo simulator or on the real robot. He is new
to Rust and wants to write every line himself: each assignment gives prerequisites, theory with
algorithmic pseudocode, a UML class diagram, the Rust features needed, the task in words, and a
check. No Rust appears above the Files section; tests and data are at the bottom to copy;
solutions are folded at the very end. This guide: [number, title, goal].

## Files
- Draft page: [path to built .html]; built from [guide source folder] with
  <skill>/scripts/build_guide.py.
- Reference solution crate: [path]; intermediate stages: [path]; challenge solutions: [path].
- The learner's starting code (end of the previous guide): [path].
- The previous guide, for style and continuity: [path].
- Hardware facts and their sources: <skill>/reference/reachy_mini_facts.md;
  upstream repos cloned at [path].
- Build and test: [the cargo/MuJoCo environment commands].

## What to review, in priority order
1. Safety on the real robot: anything that could move a motor unexpectedly, drop the head, exceed
   limits or speeds, mishandle torque, misuse registers, or fail unsafely on errors.
2. Correctness: bugs in the solution; mismatches between diagrams, task text, tests, the stated
   test counts after each assignment, and the solution; any stage that wouldn't compile when the
   page says it will; wrong facts.
3. Pedagogy: can each assignment be done from its theory, pseudocode, diagram and Rust reference
   alone, without the solution? Is anything under-specified, or does anything above Files give
   the code away? Is a Rust feature the task needs missing from "Rust you'll need"?
4. Rust and embedded best practice: API design, errors versus panics in the control path,
   allocation and printing in the loop, timing, testability, naming. Say what a senior Rust
   robotics engineer would change, for a learner-level guide on a host computer.
5. Writing: unclear or wrong sentences, inconsistencies with the previous guides.

## Output
A prioritized list. For each finding: severity (blocker / should-fix / nice-to-have), where
(section id or file:line), what is wrong, why it matters, and a concrete fix. Verify every claim
(run the tests, read the source) and mark what you could not verify. Run any timing test many
times to catch flakiness. End with what you checked and found correct. Under ~1500 words.
