"""Head poses → the six Stewart motor angles, with Pollen's analytical inverse kinematics.

A dependency-free port of `inverse_kinematics` in pollen-robotics/reachy_mini_rust_kinematics
(src/lib.rs), using the daemon's kinematics_data.json (copied to ../assets). It reproduces the
crate's own unit test to 1e-9, and every pose in guide 03 to 4 decimals.

    python3 head_ik.py NAME=x,y,z,roll,pitch,yaw [...]       mm and degrees, relative to neutral
    python3 head_ik.py LOOK_UP=0,0,0,0,-15,0 TURN_LEFT=0,0,0,0,0,25

Prints one Rust const per pose and whether each angle is inside its motor's limits.
Conventions, as in the SDK: x forward, y left, z up; rotation R = Rz(yaw)·Ry(pitch)·Rx(roll),
so pitch +15 looks down, roll +15 tilts the right ear down, yaw +25 turns left.
Body yaw is independent: the platform turns with the body.

To check that blending two poses joint by joint stays reachable, solve forward kinematics at
points along the blend (least squares on rod-length errors; scipy) and look at the residual and
the path deviation. Guide 03 did that; see reference/reachy_mini_facts.md.
"""
import json
import math
import pathlib
import sys

DATA = json.loads((pathlib.Path(__file__).resolve().parent.parent / 'assets/kinematics_data.json').read_text())
ARM, ROD, Z_OFFSET = DATA['motor_arm_length'], DATA['rod_length'], DATA['head_z_offset']

# Motor limits in raw ticks (hardware_config.yaml), stewart_1 to stewart_6.
TICK_LIMITS = [(1502, 2958), (1138, 2844), (1502, 2958), (1138, 2594), (1252, 2958), (1138, 2594)]
LIMITS = [tuple((t - 2048) * 2 * math.pi / 4096 for t in pair) for pair in TICK_LIMITS]


def matmul(a, b):
    return [[sum(a[i][k] * b[k][j] for k in range(4)) for j in range(4)] for i in range(4)]


def rot(axis, angle):
    c, s = math.cos(angle), math.sin(angle)
    m = {'x': [[1, 0, 0], [0, c, -s], [0, s, c]],
         'y': [[c, 0, s], [0, 1, 0], [-s, 0, c]],
         'z': [[c, -s, 0], [s, c, 0], [0, 0, 1]]}[axis]
    return [m[0] + [0], m[1] + [0], m[2] + [0], [0, 0, 0, 1]]


def pose(x=0.0, y=0.0, z=0.0, roll=0.0, pitch=0.0, yaw=0.0):
    """A head pose: position in metres, angles in radians."""
    t = matmul(matmul(rot('z', yaw), rot('y', pitch)), rot('x', roll))
    t[0][3], t[1][3], t[2][3] = x, y, z
    return t


def wrap(angle):
    return angle - 2 * math.pi * math.floor((angle + math.pi) / (2 * math.pi))


def ik(t_head, body_yaw=0.0):
    """The six motor angles, stewart_1 first, for a head pose relative to neutral."""
    t = [row[:] for row in t_head]
    t[2][3] += Z_OFFSET
    t = matmul(rot('z', -body_yaw), t)
    angles = []
    for motor in DATA['motors']:
        branch = [[1, 0, 0, motor['branch_position'][0]], [0, 1, 0, motor['branch_position'][1]],
                  [0, 0, 1, motor['branch_position'][2]], [0, 0, 0, 1]]
        p = matmul(matmul(motor['T_motor_world'], t), branch)
        px, py, pz = p[0][3], p[1][3], p[2][3]
        sol = 1.0 if motor['solution'] else -1.0
        x = px**2 + 2 * px * ARM + py**2 + pz**2 - ROD**2 + ARM**2
        disc = (-(px**4) - 2 * px**2 * py**2 - 2 * px**2 * pz**2 + 2 * px**2 * ROD**2
                + 2 * px**2 * ARM**2 - py**4 - 2 * py**2 * pz**2 + 2 * py**2 * ROD**2
                + 2 * py**2 * ARM**2 - pz**4 + 2 * pz**2 * ROD**2 - 2 * pz**2 * ARM**2
                - ROD**4 + 2 * ROD**2 * ARM**2 - ARM**4)
        if disc < 0:
            raise ValueError(f"{motor['name']}: pose out of reach")
        y = 2 * py * ARM + sol * math.sqrt(disc)
        angles.append(wrap(2 * math.atan2(y, x)))
    return angles


def main(args):
    for arg in args:
        name, _, values = arg.partition('=')
        x, y, z, roll, pitch, yaw = (float(v) for v in values.split(','))
        angles = ik(pose(x / 1000, y / 1000, z / 1000, *(math.radians(a) for a in (roll, pitch, yaw))))
        inside = all(lo <= a <= hi for a, (lo, hi) in zip(angles, LIMITS))
        listed = ', '.join(f'{a:.4f}' for a in angles)
        print(f'pub const {name}: Head = Head([{listed}]);  // within limits: {inside}')


if __name__ == '__main__':
    main(sys.argv[1:] or ['NEUTRAL=0,0,0,0,0,0'])
