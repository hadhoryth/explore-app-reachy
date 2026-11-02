"""Small UML class diagrams as inline SVG, themed by the page's CSS (see UML_CSS).

A diagram is a list of boxes placed by hand on a grid, plus edges between them:

    boxes = [
        Box('Joints', 'struct', ['+ body_yaw: f64', '+ head: Head'], ['+ to_array(self): [f64; 9]'],
            x=0, y=0, new=True),
        Box('Head', 'struct', ['- 0: [f64; 6]'], ['= NEUTRAL: Head'], x=360, y=0),
    ]
    edges = [Edge('Joints', 'Head', 'owns', '1')]
    svg = diagram(boxes, edges, title='The joint model')

Member lines starting with '=' are class-scope (consts, associated functions without self) and
are drawn underlined, as UML does. Edge kinds: owns (filled diamond at the owner), has (hollow
diamond), impl (dashed, hollow triangle at the trait), uses (dashed, open arrow), assoc (solid,
open arrow).
"""
import html
import math

CHAR_W = 7.3
LINE_H = 17
PAD_X = 10
PAD_Y = 6
MARGIN = 14

UML_CSS = '''
/* UML diagrams */
.uml-fig { margin: 0 0 20px; }
.uml-wrap { overflow-x: auto; background: var(--surface); border: 1px solid var(--line); border-radius: 12px; padding: 10px; }
.uml-wrap svg { display: block; width: 100%; height: auto; min-width: 34rem; }
.uml-fig figcaption { color: var(--muted); font-size: 0.9rem; padding: 6px 4px 0; }
.uml text { font-family: var(--mono); font-size: 12px; fill: var(--ink); white-space: pre; }
.uml .stereo { fill: var(--muted); font-size: 11px; }
.uml .name { font-weight: 700; font-size: 13px; }
.uml .static { text-decoration: underline; }
.uml .box { fill: var(--bg); stroke: var(--robot-edge); stroke-width: 1.2; }
.uml .box.new { stroke: var(--antenna); stroke-width: 2; }
.uml .box.fake { stroke-dasharray: 4 3; }
.uml .sep { stroke: var(--line); stroke-width: 1; }
.uml .edge { stroke: var(--muted); stroke-width: 1.3; fill: none; }
.uml .edge.dashed { stroke-dasharray: 6 4; }
.uml .tip-fill { fill: var(--muted); stroke: var(--muted); stroke-width: 1.3; }
.uml .tip-hollow { fill: var(--bg); stroke: var(--muted); stroke-width: 1.3; }
.uml .tip-open { fill: none; stroke: var(--muted); stroke-width: 1.3; }
.uml .lbl { fill: var(--muted); font-size: 11px; }
.uml .lbl-bg { fill: var(--surface); }
.uml-legend { display: flex; flex-wrap: wrap; gap: 6px 18px; font-size: 0.86rem; color: var(--muted); margin: 0 0 16px; }
.uml-legend b { color: var(--ink); font-family: var(--mono); font-weight: 600; }
'''


class Box:
    def __init__(self, name, stereo=None, fields=(), methods=(), x=0, y=0, new=False,
                 fake=False, min_w=0):
        self.name, self.stereo = name, stereo
        self.fields, self.methods = list(fields), list(methods)
        self.x, self.y, self.new, self.fake = x, y, new, fake
        lines = [name] + self.fields + self.methods + ([f'«{stereo}»'] if stereo else [])
        self.w = max(min_w, max(len(line.lstrip('=')) for line in lines) * CHAR_W + 2 * PAD_X)
        head = (LINE_H if stereo else 0) + LINE_H + 2 * PAD_Y
        self.head_h = head
        body = 0
        for part in (self.fields, self.methods):
            if part:
                body += len(part) * LINE_H + 2 * PAD_Y
        self.h = head + body

    def center(self):
        return self.x + self.w / 2, self.y + self.h / 2

    def svg(self):
        cls = 'box' + (' new' if self.new else '') + (' fake' if self.fake else '')
        out = [f'<rect class="{cls}" x="{self.x:.1f}" y="{self.y:.1f}" width="{self.w:.1f}" '
               f'height="{self.h:.1f}" rx="4"/>']
        cx = self.x + self.w / 2
        ty = self.y + PAD_Y + 12
        if self.stereo:
            out.append(f'<text class="stereo" x="{cx:.1f}" y="{ty:.1f}" text-anchor="middle">'
                       f'«{esc(self.stereo)}»</text>')
            ty += LINE_H
        out.append(f'<text class="name" x="{cx:.1f}" y="{ty:.1f}" text-anchor="middle">'
                   f'{esc(self.name)}</text>')
        y = self.y + self.head_h
        for part in (self.fields, self.methods):
            if not part:
                continue
            out.append(f'<line class="sep" x1="{self.x:.1f}" y1="{y:.1f}" '
                       f'x2="{self.x + self.w:.1f}" y2="{y:.1f}"/>')
            y += PAD_Y
            for line in part:
                static = line.startswith('=')
                text = line[1:].lstrip() if static else line
                klass = ' class="static"' if static else ''
                out.append(f'<text{klass} x="{self.x + PAD_X:.1f}" y="{y + 12:.1f}">'
                           f'{esc(text)}</text>')
                y += LINE_H
            y += PAD_Y
        return '\n'.join(out)


class Edge:
    def __init__(self, src, dst, kind='assoc', label=None, src_side=None, dst_side=None):
        self.src, self.dst, self.kind, self.label = src, dst, kind, label
        self.src_side, self.dst_side = src_side, dst_side


def esc(text):
    return html.escape(text, quote=False)


def anchor(box, toward, side=None):
    """Where a line from the box's center toward `toward` leaves the box, or the middle of a side."""
    if side:
        return {
            'top': (box.x + box.w / 2, box.y),
            'bottom': (box.x + box.w / 2, box.y + box.h),
            'left': (box.x, box.y + box.h / 2),
            'right': (box.x + box.w, box.y + box.h / 2),
        }[side]
    cx, cy = box.center()
    dx, dy = toward[0] - cx, toward[1] - cy
    if dx == 0 and dy == 0:
        return cx, cy
    sx = (box.w / 2) / abs(dx) if dx else math.inf
    sy = (box.h / 2) / abs(dy) if dy else math.inf
    s = min(sx, sy)
    return cx + dx * s, cy + dy * s


def tip(at, frm, kind):
    """The marker drawn at point `at`, for a line arriving from `frm`."""
    ang = math.atan2(at[1] - frm[1], at[0] - frm[0])
    ux, uy = math.cos(ang), math.sin(ang)
    px, py = -uy, ux

    def pt(back, side):
        return f'{at[0] - ux * back + px * side:.1f},{at[1] - uy * back + py * side:.1f}'

    if kind in ('owns', 'has'):
        cls = 'tip-fill' if kind == 'owns' else 'tip-hollow'
        pts = f'{at[0]:.1f},{at[1]:.1f} {pt(9, 5)} {pt(18, 0)} {pt(9, -5)}'
        return f'<polygon class="{cls}" points="{pts}"/>', 18
    if kind == 'impl':
        return f'<polygon class="tip-hollow" points="{at[0]:.1f},{at[1]:.1f} {pt(13, 7)} {pt(13, -7)}"/>', 13
    return f'<polyline class="tip-open" points="{pt(10, 6)} {at[0]:.1f},{at[1]:.1f} {pt(10, -6)}"/>', 0


def edge_svg(edge, boxes):
    a, b = boxes[edge.src], boxes[edge.dst]
    pa = anchor(a, b.center(), edge.src_side)
    pb = anchor(b, a.center(), edge.dst_side)
    out = []
    dashed = edge.kind in ('impl', 'uses')
    # Diamonds sit at the owner (source); arrows and triangles at the target.
    line_a, line_b = pa, pb
    if edge.kind in ('owns', 'has'):
        marker, back = tip(pa, pb, edge.kind)
        ang = math.atan2(pb[1] - pa[1], pb[0] - pa[0])
        line_a = (pa[0] + math.cos(ang) * back, pa[1] + math.sin(ang) * back)
    else:
        marker, back = tip(pb, pa, edge.kind)
        ang = math.atan2(pa[1] - pb[1], pa[0] - pb[0])
        line_b = (pb[0] + math.cos(ang) * back, pb[1] + math.sin(ang) * back)
    cls = 'edge dashed' if dashed else 'edge'
    out.append(f'<line class="{cls}" x1="{line_a[0]:.1f}" y1="{line_a[1]:.1f}" '
               f'x2="{line_b[0]:.1f}" y2="{line_b[1]:.1f}"/>')
    out.append(marker)
    if edge.label:
        mx, my = (pa[0] + pb[0]) / 2, (pa[1] + pb[1]) / 2
        w = len(edge.label) * 6.7 + 8
        out.append(f'<rect class="lbl-bg" x="{mx - w / 2:.1f}" y="{my - 9:.1f}" width="{w:.1f}" '
                   f'height="16" rx="3"/>')
        out.append(f'<text class="lbl" x="{mx:.1f}" y="{my + 3:.1f}" text-anchor="middle">'
                   f'{esc(edge.label)}</text>')
    return '\n'.join(out)


def diagram(boxes, edges=(), title='', caption=''):
    """A <figure> holding the SVG diagram and its caption."""
    by_name = {box.name: box for box in boxes}
    width = max(box.x + box.w for box in boxes) + 2 * MARGIN
    height = max(box.y + box.h for box in boxes) + 2 * MARGIN
    body = [edge_svg(edge, by_name) for edge in edges] + [box.svg() for box in boxes]
    # Never scale a diagram above its natural size: small diagrams would get huge text.
    svg = (f'<svg class="uml" viewBox="{-MARGIN} {-MARGIN} {width:.0f} {height:.0f}" '
           f'style="max-width: {width:.0f}px" '
           f'role="img" aria-label="{esc(title)}" xmlns="http://www.w3.org/2000/svg">'
           f'<title>{esc(title)}</title>\n' + '\n'.join(body) + '\n</svg>')
    cap = f'<figcaption>{caption}</figcaption>' if caption else ''
    return f'<figure class="uml-fig"><div class="uml-wrap">{svg}</div>{cap}</figure>'


LEGEND = ('<div class="uml-legend"><span><b>+</b> public</span><span><b>-</b> private</span>'
          '<span><b><u>underlined</u></b> belongs to the type, not to a value: a const or a '
          'function without <code>self</code></span><span><b>◆──</b> owns</span>'
          '<span><b>- - ▷</b> implements</span><span><b>- - &gt;</b> uses</span>'
          '<span>orange border: you build it in this assignment</span></div>')
