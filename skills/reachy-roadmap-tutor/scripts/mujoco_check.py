"""Settle the MuJoCo model at given joint angles and report where the head ends up.

Runs headless with the `mujoco` Python package (pip install mujoco, version to match the
project's MuJoCo, 3.17 for guide 03). Run it where the model's meshes are: on the learner's
computer, in the explore-app folder.

    python3 mujoco_check.py assets/mjcf/scenes/empty.xml \
        NEUTRAL=0.6265,-0.6265,0.6265,-0.6265,0.6265,-0.6265 \
        LOOK_UP=0.7612,-0.6759,0.4584,-0.4584,0.6759,-0.7612,0.3     # optional 7th: body yaw

For each pose it prints the head site's roll, pitch and yaw in degrees (xyz Euler, like the SDK),
its position in mm (z includes the 177 mm offset), how far each motor is from its target, and the
number of contacts: a pose resting on the body, like SLEEP, shows contacts and a few degrees of
error. Antenna sign check: the model's antennas turn opposite to the SDK's.
"""
import math
import sys

import mujoco
import numpy as np

NAMES = ['yaw_body'] + [f'stewart_{i}' for i in range(1, 7)] + ['right_antenna', 'left_antenna']


def euler(r):
    return [math.degrees(a) for a in (math.atan2(r[2, 1], r[2, 2]),
                                      -math.asin(max(-1.0, min(1.0, r[2, 0]))),
                                      math.atan2(r[1, 0], r[0, 0]))]


def main(scene, poses):
    model = mujoco.MjModel.from_xml_path(scene)
    data = mujoco.MjData(model)
    act = [mujoco.mj_name2id(model, mujoco.mjtObj.mjOBJ_ACTUATOR, n) for n in NAMES]
    jnt = [mujoco.mj_name2id(model, mujoco.mjtObj.mjOBJ_JOINT, n) for n in NAMES]
    site = mujoco.mj_name2id(model, mujoco.mjtObj.mjOBJ_SITE, 'head')
    for arg in poses:
        name, _, values = arg.partition('=')
        angles = [float(v) for v in values.split(',')]
        head, body = angles[:6], (angles[6] if len(angles) > 6 else 0.0)
        for a, v in zip(act, [body] + head + [0.0, 0.0]):
            data.ctrl[a] = v
        for _ in range(1500):  # 3 s of physics
            mujoco.mj_step(model, data)
        rpy = euler(data.site_xmat[site].reshape(3, 3))
        pos = data.site_xpos[site] * 1000
        err = [abs(data.qpos[model.jnt_qposadr[j]] - v) for j, v in zip(jnt[1:7], head)]
        print(f'{name:12s} rpy {np.round(rpy, 1)} deg  pos {np.round(pos, 1)} mm  '
              f'worst motor error {max(err):.4f} rad  contacts {data.ncon}')


if __name__ == '__main__':
    main(sys.argv[1], sys.argv[2:])
