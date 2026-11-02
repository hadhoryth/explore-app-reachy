# Student review

Ivan asked (2026-11-02) that every guide be read by a learner agent before it reaches him: can a
basic Rust user say, in their own words, what each type is and why it exists? Run it after the
page builds and the crates pass, alongside or before the Claude Fable review.

1. Convert the pages: `python3 <skill>/scripts/to_text.py .roadmap/NN_Title <scratch>/student/v1`.
   Diagrams become box lists, solutions are removed, copy blocks and the Rust panel are labelled.
2. Run students as subagents, in parallel: one reads the start page and the first two or three
   assignments carefully, another reads the rest. Give each only the text folder, never the
   crates or solutions.
3. Read their answers against what the page meant. A wrong answer is a page bug. Verify every
   technical claim before acting on it: the first review of guide 03 found a real bug (`Drop`
   could cut torque that `open` only found on).
4. Fix, rebuild, convert again into `v2`, and give the weakest pages to a fresh student.

## Prompt for the careful reader (one assignment, deep)

> You are playing a learner, not a code reviewer. Stay in character.
>
> **Who you are.** You know basic Rust: structs, enums, `impl`, `Vec`, `Option`/`Result`,
> `match`, `mod`/`use` at a beginner level, `cargo test`. [List what is new to the learner at
> this point.] You are new to robotics. [One line on the previous guide and the assignments
> already done.]
>
> **Read.** Plain-text pages are in [folder]. Skim index.txt, then read [aN.txt] carefully.
> Don't open anything else on disk. Format: "[UML DIAGRAM: …]" lists class-diagram boxes (BOX
> name, «stereotype», fields and methods; `+` public, `-` private). "[FOLDED COPY BLOCK: …]" is
> test code to copy. "=== SIDE PANEL ===" is the "Rust you'll need" reference beside the page.
> Solutions are removed on purpose.
>
> **Send back, in plain words, under ~900 words.**
> 1. In your own words, without copying sentences: [one question per type and file the
>    assignment builds: what is it, why does it exist, why separate from X].
> 2. Your plan per part: what you write, what you run, what count you expect.
> 3. Every sentence or term that confused you or came before it was explained (quote it, say
>    what you guessed).
> 4. Anything you'd get stuck writing, and whether the side panel covered it.
> 5. Clarity 1 to 5 per section, and any paragraph you'd cut.

## Prompt for the broad reader (several assignments)

Same persona, adjusted to what is done. For each assignment: two sentences of their own on what
they build and why the robot needs it; the path per part with the expected counts; quoted
confusions; what they couldn't write; a clarity score. Then the three biggest fixes and what
feels like too much text.
