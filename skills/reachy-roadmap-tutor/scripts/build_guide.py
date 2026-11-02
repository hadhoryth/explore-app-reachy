"""Build one roadmap guide page from its source folder.

    python3 build_guide.py <guide source folder> [output.html]

Without an output path, the page is written to <source folder>/../../<output from guide.json>,
that is .roadmap/NN_Title.html when the source folder is .roadmap/src/NN_Title/.

The source folder holds:
    guide.json      {"title": ..., "output": "NN_Title.html", "parts": ["parts/10_top.html", ...]}
    parts/*.html    the page body, in order
    diagrams.py     DIAGRAMS = {"a1": function returning a <figure>}, built with uml.py
    rust_refs.py    ENTRIES = {id: (what, how)}, PER_ASSIGNMENT = {key: [ids]}, ASSIGNMENT_NAMES
    code/...        the verified Rust the markers below inject

Markers in parts (paths are relative to the source folder):
    {{FILE:path}}                 whole file
    {{HEAD:path}}                 file up to its #[cfg(test)]
    {{ITEM:path::pat||pat}}       Rust items whose first line contains pat (suffix @last: last match)
    {{TESTS:path::name,name}}     #[test] functions by name
    {{RANGE:path::start::end}}    lines from the one containing start up to the one containing end
    {{UML:key}}  {{LEGEND}}       a diagram, the UML legend
    {{RUST:key}}  {{RUSTINDEX}}   an assignment's Rust reference, the index of all entries

Code inside <code data-lang="..."> and <code class="tree"> is written raw and escaped here.
Checks: no marker left, no duplicate ids, no link to a missing id, and no Rust code block
above <section id="files"> (pseudocode uses data-lang="text").
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
.also { color: var(--muted); font-size: 0.93rem; }
.pseudo-note { color: var(--muted); font-size: 0.9rem; margin: -10px 0 18px; }

/* Rust reference tables: long code may wrap */
.lookup td code { white-space: normal; overflow-wrap: anywhere; }
.lookup td:first-child { min-width: 9rem; }
.lookup tr:target td { background: color-mix(in srgb, var(--antenna) 12%, transparent); }
.step li > ul { margin: 6px 0 0; }
details.files { border-left: 4px solid var(--link); }
details.files summary::before { content: "Copy · "; color: var(--link); }
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
        self.home = {}  # entry id -> assignment key where it is shown in full
        for key, ids in self.per.items():
            for entry in ids:
                self.home.setdefault(entry, key)

    def read(self, path):
        return (self.src / path).read_text()

    def rust_block(self, key):
        rows, also = [], []
        for entry in self.per[key]:
            what, how = self.entries[entry]
            if self.home[entry] == key:
                rows.append(f'<tr id="{entry}"><td>{what}</td><td>{how}</td></tr>')
            else:
                also.append(f'<a href="#{entry}">{what}</a> ({self.label(self.home[entry])})')
        out = ''
        if rows:
            out += ('<div class="table-wrap lookup"><table><thead><tr><th>What</th><th>How</th>'
                    '</tr></thead><tbody>' + ''.join(rows) + '</tbody></table></div>')
        if also:
            out += '<p class="also">Also from earlier: ' + ', '.join(also) + '.</p>'
        return out

    def label(self, key):
        name = self.names[key]
        return name if not name[0].isdigit() else f'assignment {name}'

    def rust_index(self):
        rows = []
        for entry, (what, _) in sorted(self.entries.items(),
                                       key=lambda kv: re.sub('<[^>]+>', '', kv[1][0]).lower()):
            key = self.home[entry]
            rows.append(f'<tr><td><a href="#{entry}">{what}</a></td>'
                        f'<td><a href="#{key}">{self.label(key)}</a></td></tr>')
        return ('<div class="table-wrap lookup"><table><thead><tr><th>Feature</th>'
                '<th>Explained in</th></tr></thead><tbody>' + ''.join(rows) + '</tbody></table></div>')

    def expand(self, match):
        kind, arg = match.group(1), match.group(2)
        if kind == 'UML':
            return self.diagrams[arg]()
        if kind == 'RUST':
            return self.rust_block(arg)
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
            while chunk and (not chunk[-1].strip() or chunk[-1].lstrip().startswith('///')):
                chunk.pop()
            return '\n'.join(chunk)
        raise ValueError(kind)

    def build(self, out=None):
        body = ''.join(self.read(part) for part in self.config['parts'])
        body = body.replace('{{LEGEND}}', uml.LEGEND).replace('{{RUSTINDEX}}', self.rust_index())
        body = re.sub(r'\{\{(FILE|HEAD|ITEM|TESTS|RANGE|UML|RUST):(.*?)\}\}', self.expand, body)
        body = re.sub(r'(<code data-lang="[a-z]+">|<code class="tree">)(.*?)</code>',
                      lambda m: m.group(1) + html.escape(m.group(2), quote=False) + '</code>',
                      body, flags=re.S)
        head = (ASSETS / 'page_head.html').read_text()
        head = head.replace('{{TITLE}}', self.config['title'])
        head = head.replace('{{EXTRA_CSS}}', EXTRA_CSS + uml.UML_CSS)
        page = head + body + '\n\n' + (ASSETS / 'page_tail.html').read_text()
        self.check(page)
        out = pathlib.Path(out or self.src.parent.parent / self.config['output'])
        out.write_text(page)
        return out, len(page)

    @staticmethod
    def check(page):
        leftover = re.findall(r'\{\{[A-Z]+[:}]', page)
        assert not leftover, f'markers left: {leftover}'
        ids = re.findall(r'id="([^"]+)"', page)
        dups = sorted({i for i in ids if ids.count(i) > 1})
        assert not dups, f'duplicate ids: {dups}'
        missing = sorted(set(re.findall(r'href="#([^"]+)"', page)) - set(ids))
        assert not missing, f'links to missing ids: {missing}'
        files_at = page.find('<section id="files"')
        assert files_at > 0, 'no <section id="files">'
        above = page[:files_at].count('<code data-lang="rust">')
        assert above == 0, f'{above} Rust code blocks above the Files section'


if __name__ == '__main__':
    path, size = Builder(sys.argv[1]).build(sys.argv[2] if len(sys.argv) > 2 else None)
    print(path, size)
