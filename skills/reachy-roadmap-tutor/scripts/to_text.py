"""Turn built guide pages into plain text a student agent can read: diagrams as box lists,
solutions removed, copy blocks and the Rust panel kept and labelled.

    python3 to_text.py .roadmap/NN_Title /tmp/student   # one .txt per page

Needs beautifulsoup4 (pip install beautifulsoup4).
"""
import sys, re, html, pathlib
from bs4 import BeautifulSoup, NavigableString

def text_of(page, keep_solutions=False):
    soup = BeautifulSoup(page, 'html.parser')
    for t in soup(['script', 'style', 'head']):
        t.decompose()
    for b in soup.select('button'):
        b.decompose()
    if not keep_solutions:
        for d in soup.select('details.solution'):
            d.replace_with(NavigableString('\n[SOLUTION: folded on the page; not shown to you]\n'))
    for fig in soup.select('figure.uml-fig'):
        svg = fig.find('svg')
        title = svg.get('aria-label', '')
        boxes = []
        for t in svg.find_all('text'):
            cls = ' '.join(t.get('class', []))
            txt = t.get_text()
            if 'name' in cls:
                boxes.append(f'\n  BOX {txt}')
            elif 'stereo' in cls:
                boxes.append(f' {txt}')
            elif 'lbl' in cls:
                boxes.append(f'\n  (edge label: {txt})')
            else:
                boxes.append(f'\n    {txt}')
        cap = fig.find('figcaption')
        fig.replace_with(NavigableString(f'\n[UML DIAGRAM: {title}]' + ''.join(boxes) + f'\n  CAPTION: {cap.get_text(" ", strip=True) if cap else ""}\n'))
    for d in soup.select('details.files'):
        s = d.find('summary').get_text(' ', strip=True)
        d.find('summary').replace_with(NavigableString(f'\n[FOLDED COPY BLOCK: {s}]\n'))
    for pre in soup.find_all('pre'):
        pre.replace_with(NavigableString('\n```\n' + pre.get_text() + '\n```\n'))
    for h in soup.find_all(['h1', 'h2', 'h3', 'h4', 'h5']):
        h.insert_before(NavigableString('\n\n' + '#' * int(h.name[1]) + ' '))
        h.insert_after(NavigableString('\n'))
    for li in soup.find_all('li'):
        li.insert_before(NavigableString('\n - '))
    for p in soup.find_all(['p', 'tr', 'div']):
        p.insert_after(NavigableString('\n'))
    for td in soup.find_all(['td', 'th']):
        td.insert_after(NavigableString(' | '))
    panel = soup.select_one('aside.rust-panel')
    if panel:
        panel.insert_before(NavigableString('\n\n=== SIDE PANEL (always visible on the right) ===\n'))
        for d in panel.select('details.rust-entry'):
            d.find('summary').insert_before(NavigableString('\n* '))
            d.find('summary').insert_after(NavigableString(': '))
    txt = soup.get_text()
    txt = re.sub(r'[ \t]+', ' ', txt)
    txt = re.sub(r'\n\s*\n\s*\n+', '\n\n', txt)
    return txt.strip() + '\n'

if __name__ == '__main__':
    src, dst = pathlib.Path(sys.argv[1]), pathlib.Path(sys.argv[2])
    dst.mkdir(parents=True, exist_ok=True)
    for f in sorted(src.glob('*.html')):
        (dst / (f.stem + '.txt')).write_text(text_of(f.read_text()))
        print(f.stem, len((dst / (f.stem + '.txt')).read_text()))
