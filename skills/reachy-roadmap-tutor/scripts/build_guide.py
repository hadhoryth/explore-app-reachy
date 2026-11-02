"""Build a roadmap guide from its source folder: one page per assignment.

    python3 build_guide.py <guide source folder> [output folder]

Without an output folder, the pages go to <source folder>/../../<output from guide.json>/, that is
.roadmap/NN_Title/ when the source folder is .roadmap/src/NN_Title/, and .roadmap/NN_Title.html
becomes a redirect to its index.html (the guide used to be a single page there).

The source folder holds:
    guide.json      {"title": ..., "short": "Guide 03", "output": "NN_Title",
                     "pages": [{"file": "index.html", "nav": "Start", "parts": [...]},
                               {"file": "a1.html", "nav": "1", "parts": [...], "rust": "a1"}, ...]}
    parts/*.html    page bodies; a page is the concatenation of its parts
    diagrams.py     DIAGRAMS = {"a1": function returning a <figure>}, built with uml.py
    rust_refs.py    ENTRIES = {id: (what, how)}, PER_ASSIGNMENT = {key: [ids]}, ASSIGNMENT_NAMES
    code/...        the verified Rust the markers below inject

A page with "rust": key gets the Rust it needs, PER_ASSIGNMENT[key], as a panel on the right
(a drawer behind a button on narrow screens). Every page gets the step bar on top and links to
the previous and next page at the bottom. href="#id" links to an id on another page are
rewritten to that page; an id that appears on several pages resolves to the first.

Markers in parts (paths are relative to the source folder):
    {{FILE:path}}                 whole file
    {{HEAD:path}}                 file up to its #[cfg(test)]
    {{ITEM:path::pat||pat}}       Rust items whose first line contains pat (suffix @last: last match)
    {{TESTS:path::name,name}}     #[test] functions by name
    {{RANGE:path::start::end}}    lines from the one containing start up to the one containing end,
                                  without the doc comments and attributes that belong to the end
    {{UML:key}}  {{LEGEND}}       a diagram, the UML legend
    {{RUSTINDEX}}  {{PAGES}}      the index of all Rust entries, the list of assignment pages

Code inside <code data-lang="..."> and <code class="tree"> is written raw and escaped here.
Checks, per page: no marker left, no duplicate ids, no link to a missing id or page, and no
Rust code block outside a folded <details> (tests to copy, solutions, the complete program);
pseudocode uses data-lang="text".
"""
import html
import importlib.util
import json
import pathlib
import re
import sys

sys.dont_write_bytecode = True  # keep __pycache__ out of the repo

TOOLS = pathlib.Path(__file__).resolve().parent
ASSETS = TOOLS.parent / 'assets'
sys.path.insert(0, str(TOOLS))
import uml  # noqa: E402

EXTRA_CSS = '''
/* Hero with the guide's anatomy beside the title */
@media (min-width: 961px) {
  .hero.two { grid-template-columns: minmax(0, 1.08fr) minmax(0, 0.92fr); align-items: start; }
}
.how-card { background: var(--surface); border: 1px solid var(--line); border-radius: 16px; padding: 20px 22px 8px; }
.how-card h2 { font-size: 1.2rem; margin: 0 0 6px; }
.how-card > p { color: var(--muted); font-size: 0.95rem; margin: 0 0 10px; }
.how-card ol { padding-left: 1.4em; margin: 0 0 12px; }
.how-card li { font-size: 0.95rem; margin-bottom: 5px; }
.how-card li b { font-weight: 600; }

/* Assignment anatomy */
.step h4 { border-top: 1px solid var(--line); padding-top: 14px; }
.step h3 + .learn + h4, .step .learn + h4 { border-top: 0; padding-top: 0; }
.prereq { list-style: none; padding: 0; }
.prereq li::before { content: "✓ "; color: var(--accent); font-weight: 700; }
.task-steps > li { margin-bottom: 10px; }
.also { color: var(--muted); font-size: 0.88em; }
.pseudo-note { color: var(--muted); font-size: 0.85em; margin: -10px 0 18px; }
.goal { border-left: 4px solid var(--link); background: var(--surface); border-radius: 0 10px 10px 0; padding: 14px 18px 2px; margin: 0 0 20px; }
.goal > p:first-child { font-size: 1.05em; }
.goal .goal-done { font-weight: 600; margin: 0 0 6px; }
.goal ul { margin: 0 0 12px; }
.step details.files, .step details.solution { margin: 0 0 14px; }
/* An assignment that builds several pieces repeats Design, Task and Check once per part */
.part { margin: 36px 0 8px; }
.step h4.part-title { font-size: 1.12em; text-transform: none; letter-spacing: normal; color: var(--ink); border-top: 2px solid var(--ink); padding-top: 14px; margin: 0 0 4px; }
.part-n { display: block; font-size: 0.66em; font-weight: 600; text-transform: uppercase; letter-spacing: 0.07em; color: var(--muted); margin-bottom: 2px; }
.step h5 { font-size: 0.74em; text-transform: uppercase; letter-spacing: 0.07em; color: var(--muted); margin: 26px 0 8px; border-top: 1px solid var(--line); padding-top: 12px; }
.step h4.part-title + h5 { border-top: 0; padding-top: 0; margin-top: 18px; }

/* Rust reference tables: long code may wrap */
.lookup td code { white-space: normal; overflow-wrap: anywhere; }
.lookup td:first-child { min-width: 9rem; }
.lookup tr:target td { background: color-mix(in srgb, var(--antenna) 12%, transparent); }
.step li > ul { margin: 6px 0 0; }
details.files { border-left: 4px solid var(--link); }
details.files summary::before { content: "Copy · "; color: var(--link); }

/* One page per assignment: step bar, article, Rust panel on the right */
.page { max-width: 1260px; }
.topbar { display: flex; flex-wrap: wrap; align-items: center; justify-content: space-between; gap: 10px 24px; padding: 18px 0 14px; border-bottom: 1px solid var(--line); margin-bottom: 36px; }
.topbar .guide-home { font-weight: 700; color: var(--ink); text-decoration: none; }
.topbar .guide-home span { color: var(--muted); font-weight: 500; }
.steps { display: flex; flex-wrap: wrap; gap: 6px; }
.steps a { min-width: 36px; height: 36px; padding: 0 10px; display: inline-grid; place-items: center; border-radius: 999px; border: 1.5px solid var(--line); color: var(--muted); text-decoration: none; font-weight: 600; font-size: 0.92rem; font-variant-numeric: tabular-nums; }
.steps a:hover { color: var(--ink); border-color: var(--ink); }
.steps a[aria-current="page"] { background: var(--ink); color: var(--bg); border-color: var(--ink); }
.layout2 { display: grid; grid-template-columns: minmax(0, 1fr) 330px; gap: 44px; align-items: start; }
.layout2.no-panel { grid-template-columns: minmax(0, 1fr); }
.layout2.no-panel article { margin: 0 auto; width: 100%; }
.page-assignment .step > h3:first-of-type { font-size: 1.9em; line-height: 1.15; margin: 0 0 12px; }
.page-assignment .step::before { top: 4px; }
.rust-panel { position: sticky; top: 16px; max-height: calc(100vh - 32px); overflow-y: auto; background: var(--surface); border: 1px solid var(--line); border-radius: 14px; padding: 16px 18px 10px; font-size: 16px; line-height: 1.55; }
.rust-panel h2 { font-size: 1.1rem; margin: 0 0 2px; }
.rust-panel .panel-note { color: var(--muted); font-size: 0.86rem; margin: 0 0 10px; }
.rust-panel .panel-group { font-size: 0.72rem; text-transform: uppercase; letter-spacing: 0.07em; color: var(--muted); margin: 14px 0 2px; font-weight: 600; }
details.rust-entry { border: 0; border-top: 1px solid var(--line); border-radius: 0; background: transparent; margin: 0; }
details.rust-entry > summary { padding: 8px 0; font-weight: 600; font-size: 0.95rem; }
details.rust-entry[open] > summary { border-bottom: 0; margin-bottom: 0; }
details.rust-entry > div { padding: 0 0 12px; font-size: 0.92rem; }
details.rust-entry code { overflow-wrap: anywhere; }
details.rust-entry:target > summary { color: var(--link); }
details.rust-entry .from { color: var(--muted); font-weight: 500; font-size: 0.82em; white-space: nowrap; }
.tip { border-left: 4px solid var(--accent); background: var(--surface); border-radius: 0 10px 10px 0; padding: 10px 14px; }
.prereq li.have::before { content: "→ "; color: var(--link); }
.rust-close, .rust-toggle { display: none; }
@media (max-width: 1120px) {
  .layout2 { grid-template-columns: minmax(0, 1fr); }
  .rust-panel { position: fixed; top: 0; right: 0; bottom: 0; width: min(390px, 92vw); max-height: none; border-radius: 0; border-width: 0 0 0 1px; z-index: 30; transform: translateX(105%); transition: transform 0.2s ease; box-shadow: -8px 0 24px rgba(0, 0, 0, 0.18); padding-top: 52px; }
  :root[data-rust="open"] .rust-panel { transform: none; }
  .rust-close { display: block; position: absolute; top: 12px; right: 12px; font: inherit; font-size: 0.9rem; font-weight: 600; padding: 6px 12px; border-radius: 8px; border: 1px solid var(--line); background: var(--bg); color: var(--ink); cursor: pointer; }
  .rust-toggle { display: block; position: fixed; right: 16px; bottom: 16px; z-index: 29; font: inherit; font-weight: 700; padding: 10px 16px; border-radius: 999px; border: 0; background: var(--ink); color: var(--bg); box-shadow: 0 4px 14px rgba(0, 0, 0, 0.25); cursor: pointer; }
}
.pager { display: flex; justify-content: space-between; gap: 12px; border-top: 1px solid var(--line); padding-top: 20px; margin: 48px 0 24px; }
.pager a { display: block; max-width: 48%; padding: 12px 16px; border: 1.5px solid var(--line); border-radius: 12px; text-decoration: none; color: var(--ink); font-weight: 600; }
.pager a:hover { border-color: var(--ink); }
.pager .next { margin-left: auto; text-align: right; }
.pager small { display: block; color: var(--muted); font-size: 0.78em; font-weight: 500; }
/* The path through a multi-part assignment */
ol.path { list-style: none; counter-reset: path; padding: 0; margin: 0 0 20px; }
ol.path > li { counter-increment: path; position: relative; padding: 10px 14px 10px 52px; margin: 0 0 8px; border: 1px solid var(--line); border-radius: 12px; background: var(--surface); }
ol.path > li::before { content: counter(path); position: absolute; left: 14px; top: 10px; width: 26px; height: 26px; border-radius: 50%; background: var(--ink); color: var(--bg); display: grid; place-items: center; font-weight: 700; font-size: 0.8em; }
ol.path a { font-weight: 600; }
ol.path .count { color: var(--muted); font-size: 0.88em; }
/* The list of assignment pages on the start page */
ol.cards { list-style: none; padding: 0; display: grid; gap: 12px; }
ol.cards a { display: grid; grid-template-columns: 44px minmax(0, 1fr); gap: 4px 14px; padding: 14px 16px; border: 1.5px solid var(--line); border-radius: 14px; text-decoration: none; color: var(--ink); background: var(--surface); }
ol.cards a:hover { border-color: var(--ink); }
ol.cards .n { grid-row: span 2; width: 40px; height: 40px; border-radius: 50%; border: 2px solid var(--ink); display: grid; place-items: center; font-weight: 700; }
ol.cards .t { font-weight: 700; }
ol.cards .g { color: var(--muted); font-size: 0.92em; }
'''


def load(path, name):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def item_end(lines, start):
    """Index of the last line of the Rust item starting at `start`, by bracket depth."""
    depth, opened = 0, False
    for i in range(start, len(lines)):
        line, j = lines[i], 0
        while j < len(line):
            c = line[j]
            if line.startswith('//', j):
                break
            if c == '"':
                j += 1
                while j < len(line) and line[j] != '"':
                    j += 2 if line[j] == '\\' else 1
            elif c == "'" and re.match(r"'(\\.|[^\\'])'", line[j:]):
                j += len(re.match(r"'(\\.|[^\\'])'", line[j:]).group(0)) - 1
            elif c in '([{':
                depth, opened = depth + 1, True
            elif c in ')]}':
                depth -= 1
                if depth == 0 and c == '}' and opened:
                    return i
            elif c == ';' and depth == 0:
                return i
            j += 1
    raise ValueError(f'unterminated item at line {start}: {lines[start]}')


def item_start(lines, i):
    while i > 0 and re.match(r'\s*(///|#\[)', lines[i - 1]):
        i -= 1
    return i


def find_line(lines, pattern):
    last = pattern.endswith('@last')
    pattern = pattern.removesuffix('@last')
    hits = [i for i, line in enumerate(lines) if pattern in line]
    if not hits:
        raise ValueError(f'pattern not found: {pattern!r}')
    return hits[-1] if last else hits[0]


def extract(text, pattern):
    lines = text.split('\n')
    i = find_line(lines, pattern)
    return '\n'.join(lines[item_start(lines, i):item_end(lines, i) + 1])


class Builder:
    def __init__(self, src):
        self.src = pathlib.Path(src).resolve()
        self.config = json.loads((self.src / 'guide.json').read_text())
        self.diagrams = load(self.src / 'diagrams.py', 'diagrams').DIAGRAMS
        refs = load(self.src / 'rust_refs.py', 'rust_refs')
        self.entries, self.per = refs.ENTRIES, refs.PER_ASSIGNMENT
        self.names = refs.ASSIGNMENT_NAMES
        self.pages = self.config['pages']
        self.page_of = {}  # assignment key -> page file
        for page in self.pages:
            if 'rust' in page:
                self.page_of[page['rust']] = page['file']

    def read(self, path):
        return (self.src / path).read_text()

    # ---------- Rust reference ----------

    def rust_panel(self, key):
        """The Rust an assignment needs, for the side panel: what's new here first, then what
        it uses from earlier assignments, every entry in full and folded."""
        home = {}
        for k, ids in self.per.items():
            for entry in ids:
                home.setdefault(entry, k)

        def item(entry, tag=''):
            what, how = self.entries[entry]
            return (f'<details class="rust-entry" id="{entry}"><summary>{what}{tag}</summary>'
                    f'<div>{how}</div></details>')
        new = [item(e) for e in self.per[key] if home[e] == key]
        old = [item(e, f' <span class="from">from {self.names[home[e]]}</span>')
               for e in self.per[key] if home[e] != key]
        body = ''
        if new:
            body += '<p class="panel-group">New here</p>' + ''.join(new)
        if old:
            body += '<p class="panel-group">From earlier</p>' + ''.join(old)
        return ('<aside class="rust-panel" id="rust" aria-label="Rust you’ll need">'
                '<button type="button" class="rust-close" aria-label="Close the Rust reference">'
                'Close</button>'
                '<h2>Rust you’ll need</h2><p class="panel-note">Open a name to read it. The '
                'examples are about other things, so they show the shape, not your solution.</p>'
                + body + '</aside>'
                '<button type="button" class="rust-toggle" aria-controls="rust">Rust</button>')

    def label(self, key):
        name = self.names[key]
        return name if not name[0].isdigit() else f'assignment {name}'

    def rust_index(self):
        home = {}
        for key, ids in self.per.items():
            for entry in ids:
                home.setdefault(entry, key)
        rows = []
        for entry, (what, _) in sorted(self.entries.items(),
                                       key=lambda kv: re.sub('<[^>]+>', '', kv[1][0]).lower()):
            key = home[entry]
            rows.append(f'<tr><td><a href="#{entry}">{what}</a></td>'
                        f'<td><a href="{self.page_of[key]}">{self.label(key)}</a></td></tr>')
        return ('<div class="table-wrap lookup"><table><thead><tr><th>Feature</th>'
                '<th>Explained in</th></tr></thead><tbody>' + ''.join(rows) + '</tbody></table></div>')

    # ---------- Pages ----------

    def page_title(self, page):
        """A page's own title: the first <h3> of its first part, or its "title"."""
        if 'title' in page:
            return page['title']
        m = re.search(r'<h3>(.*?)</h3>', self.read(page['parts'][0]), re.S)
        return re.sub('<[^>]+>', '', m.group(1)).strip()

    def pages_list(self):
        """Cards for the assignment pages, each with the first sentence of its goal."""
        cards = []
        for page in self.pages:
            if 'rust' not in page:
                continue
            first = self.read(page['parts'][0])
            m = re.search(r'<div class="goal">\s*<p>(.*?)</p>', first, re.S) or \
                re.search(r'<p>(.*?)</p>', first, re.S)
            goal = re.sub(r'\s+', ' ', m.group(1)).strip()
            cards.append(f'<li><a href="{page["file"]}"><span class="n">{page["nav"]}</span>'
                         f'<span class="t">{self.page_title(page)}</span>'
                         f'<span class="g">{goal}</span></a></li>')
        return '<ol class="cards">' + ''.join(cards) + '</ol>'

    def topbar(self, current):
        links = ''.join(
            f'<a href="{p["file"]}"' + (' aria-current="page"' if p is current else '')
            + f' title="{html.escape(self.page_title(p))}">{p["nav"]}</a>' for p in self.pages)
        return (f'<header class="topbar"><a class="guide-home" href="index.html">'
                f'{self.config["short"]} <span>· {self.config["title"]}</span></a>'
                f'<nav class="steps" aria-label="Pages of this guide">{links}</nav></header>')

    def pager(self, i):
        out = []
        if i > 0:
            p = self.pages[i - 1]
            out.append(f'<a class="prev" href="{p["file"]}"><small>← Previous</small>'
                       f'{self.nav_label(p)}</a>')
        if i + 1 < len(self.pages):
            p = self.pages[i + 1]
            out.append(f'<a class="next" href="{p["file"]}"><small>Next →</small>'
                       f'{self.nav_label(p)}</a>')
        return '<nav class="pager" aria-label="Previous and next page">' + ''.join(out) + '</nav>'

    def nav_label(self, page):
        title = html.escape(self.page_title(page), quote=False)
        return f'{page["nav"]} · {title}' if page['nav'][0].isdigit() else title

    # ---------- Markers ----------

    def expand(self, match):
        kind, arg = match.group(1), match.group(2)
        if kind == 'UML':
            return self.diagrams[arg]()
        if kind == 'FILE':
            return self.read(arg).rstrip('\n')
        path, _, rest = arg.partition('::')
        text = self.read(path)
        if kind == 'HEAD':
            return text[:text.index('#[cfg(test)]')].rstrip('\n')
        if kind == 'ITEM':
            return '\n\n'.join(extract(text, p) for p in rest.split('||'))
        if kind == 'TESTS':
            return '\n\n'.join(extract(text, f'fn {n.strip()}(') for n in rest.split(','))
        if kind == 'RANGE':
            start, end = rest.split('::')
            lines = text.split('\n')
            chunk = lines[find_line(lines, start):find_line(lines, end)]
            while chunk and (not chunk[-1].strip() or chunk[-1].lstrip().startswith(('///', '#['))):
                chunk.pop()
            return '\n'.join(chunk)
        raise ValueError(kind)

    def render(self, body):
        body = body.replace('{{LEGEND}}', uml.LEGEND).replace('{{RUSTINDEX}}', self.rust_index())
        body = body.replace('{{PAGES}}', self.pages_list())
        body = re.sub(r'\{\{(FILE|HEAD|ITEM|TESTS|RANGE|UML):(.*?)\}\}', self.expand, body)
        return re.sub(r'(<code data-lang="[a-z]+">|<code class="tree">)(.*?)</code>',
                      lambda m: m.group(1) + html.escape(m.group(2), quote=False) + '</code>',
                      body, flags=re.S)

    # ---------- Build ----------

    def build(self, out=None):
        root = self.src.parent.parent
        out = pathlib.Path(out or root / self.config['output'])
        out.mkdir(parents=True, exist_ok=True)
        head = (ASSETS / 'page_head.html').read_text()
        head = head.replace('{{EXTRA_CSS}}', EXTRA_CSS + uml.UML_CSS)
        tail = (ASSETS / 'page_tail.html').read_text()
        built = {}
        for i, page in enumerate(self.pages):
            body = self.render(''.join(self.read(part) for part in page['parts']))
            key = page.get('rust')
            if page['file'] == 'index.html':
                kind, title = 'page-index', self.config['title']
            else:
                kind = 'page-assignment' if key else 'page-other'
                title = f'{self.nav_label(page)} · {self.config["short"]}'
            hero = ''
            if page.get('hero'):  # the start page's hero sits above the columns
                hero_end = body.index('</header>') + len('</header>')
                hero, body = body[:hero_end], body[hero_end:]
            panel = self.rust_panel(key) if key else ''
            built[page['file']] = (
                head.replace('{{TITLE}}', html.escape(re.sub('<[^>]+>', '', title)))
                + f'<body class="{kind}">\n<div class="page">\n{self.topbar(page)}\n{hero}\n'
                + f'<div class="layout2{"" if panel else " no-panel"}">\n<article>\n{body}\n'
                + f'{self.pager(i)}\n</article>\n{panel}\n</div>\n</div>\n\n{tail}')
        ids = {f: set(re.findall(r'id="([^"]+)"', text)) for f, text in built.items()}
        home = {}
        for page in self.pages:
            for i in sorted(ids[page['file']]):
                home.setdefault(i, page['file'])

        def relink(f):
            def sub(m):
                target = m.group(1)
                if target in ids[f]:
                    return m.group(0)
                assert target in home, f'{f}: link to missing id #{target}'
                return f'href="{home[target]}#{target}"'
            return sub
        total = 0
        for f, text in built.items():
            text = re.sub(r'href="#([^"]+)"', relink(f), text)
            # Other guides sit one folder up now.
            text = re.sub(r'href="(\d\d_[^"/]+\.html)"', r'href="../\1"', text)
            self.check(f, text, ids)
            (out / f).write_text(text)
            total += len(text)
        redirect = root / f'{self.config["output"]}.html'
        if not pathlib.Path(out).resolve().parent == root.resolve():
            redirect = None
        if redirect:  # the old single page: send old links, anchors included, to the right page
            folder = self.config['output']
            where = json.dumps(home, separators=(',', ':'), sort_keys=True)
            redirect.write_text(
                f'<!doctype html>\n<html lang="en"><head><meta charset="utf-8">'
                f'<title>{self.config["title"]}</title>\n<script>\nvar where = {where};\n'
                f'var id = decodeURIComponent(location.hash.slice(1));\n'
                f'location.replace("{folder}/" + (where[id] || "index.html") + location.hash);\n'
                f'</script></head>\n<body><p>This guide is now one page per assignment: '
                f'<a href="{folder}/index.html">open it</a>.</p></body></html>\n')
        return out, len(built), total

    @staticmethod
    def check(name, page, ids):
        leftover = re.findall(r'\{\{[A-Z]+[:}]', page)
        assert not leftover, f'{name}: markers left: {leftover}'
        found = re.findall(r'id="([^"]+)"', page)
        dups = sorted({i for i in found if found.count(i) > 1})
        assert not dups, f'{name}: duplicate ids: {dups}'
        for target, anchor in re.findall(r'href="([a-z0-9_-]+\.html)?#([^"]+)"', page):
            where = target or name
            assert where in ids, f'{name}: link to a missing page {where}'
            assert anchor in ids[where], f'{name}: link to missing id {where}#{anchor}'
        for target in re.findall(r'href="([a-z0-9_-]+\.html)"', page):
            assert target in ids, f'{name}: link to a missing page {target}'
        depth, unfolded = 0, 0
        for tag in re.findall(r'<details[\s>]|</details>|<code data-lang="rust">', page):
            if tag.startswith('<details'):
                depth += 1
            elif tag == '</details>':
                depth -= 1
            elif depth == 0:
                unfolded += 1
        assert depth == 0, f'{name}: unbalanced <details>'
        assert unfolded == 0, f'{name}: {unfolded} Rust code blocks outside a folded <details>'


if __name__ == '__main__':
    folder, count, size = Builder(sys.argv[1]).build(sys.argv[2] if len(sys.argv) > 2 else None)
    print(folder, f'{count} pages', size)
