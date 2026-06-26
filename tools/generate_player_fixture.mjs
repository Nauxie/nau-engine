#!/usr/bin/env node

import { mkdirSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";

const outPath = join("assets", "models", "player", "player.gltf");
const NAU_FIXTURE_SCHEMA = "nau_visual_asset_fixture.v1";
const NAU_FIXTURE_LICENSE = "self_authored_no_third_party";

class GltfBuffer {
  constructor() {
    this.bytes = [];
    this.bufferViews = [];
    this.accessors = [];
  }

  align() {
    while (this.bytes.length % 4 !== 0) {
      this.bytes.push(0);
    }
  }

  addBufferView(data, target) {
    this.align();
    const byteOffset = this.bytes.length;
    this.bytes.push(...data);
    const view = {
      buffer: 0,
      byteOffset,
      byteLength: data.length,
    };
    if (target !== undefined) {
      view.target = target;
    }
    this.bufferViews.push(view);
    return this.bufferViews.length - 1;
  }

  addFloatAccessor(values, type, min, max) {
    const buffer = Buffer.alloc(values.length * 4);
    values.forEach((value, index) => buffer.writeFloatLE(value, index * 4));
    const bufferView = this.addBufferView([...buffer], 34962);
    this.accessors.push({
      bufferView,
      byteOffset: 0,
      componentType: 5126,
      count: values.length / componentCount(type),
      type,
      min,
      max,
    });
    return this.accessors.length - 1;
  }

  addIndexAccessor(values) {
    const buffer = Buffer.alloc(values.length * 2);
    values.forEach((value, index) => buffer.writeUInt16LE(value, index * 2));
    const bufferView = this.addBufferView([...buffer], 34963);
    this.accessors.push({
      bufferView,
      byteOffset: 0,
      componentType: 5123,
      count: values.length,
      type: "SCALAR",
      min: [Math.min(...values)],
      max: [Math.max(...values)],
    });
    return this.accessors.length - 1;
  }

  toBuffer() {
    return Buffer.from(this.bytes);
  }
}

function componentCount(type) {
  return {
    SCALAR: 1,
    VEC2: 2,
    VEC3: 3,
    VEC4: 4,
  }[type];
}

const gltfBuffer = new GltfBuffer();

const meshes = [];

function normalize(value) {
  const length = Math.hypot(value[0], value[1], value[2]);
  if (length <= 1e-6) {
    return [0, 1, 0];
  }
  return [value[0] / length, value[1] / length, value[2] / length];
}

function flatten(values) {
  return values.flatMap((value) => value);
}

function addMesh(name, data, material) {
  const positionAccessor = gltfBuffer.addFloatAccessor(
    flatten(data.positions),
    "VEC3",
    minByComponent(data.positions),
    maxByComponent(data.positions),
  );
  const normalAccessor = gltfBuffer.addFloatAccessor(
    flatten(data.normals),
    "VEC3",
    [-1.0, -1.0, -1.0],
    [1.0, 1.0, 1.0],
  );
  const uvAccessor = gltfBuffer.addFloatAccessor(
    flatten(data.uvs),
    "VEC2",
    minByComponent(data.uvs),
    maxByComponent(data.uvs),
  );
  const indexAccessor = gltfBuffer.addIndexAccessor(data.indices);
  meshes.push({
    name,
    primitives: [
      {
        attributes: {
          POSITION: positionAccessor,
          NORMAL: normalAccessor,
          TEXCOORD_0: uvAccessor,
        },
        indices: indexAccessor,
        material,
        mode: 4,
      },
    ],
  });
}

function taperedCylinderMesh(bottomRadius, topRadius, segments = 14) {
  const positions = [];
  const normals = [];
  const uvs = [];
  for (let ring = 0; ring < 2; ring += 1) {
    const y = ring === 0 ? -0.5 : 0.5;
    const radius = ring === 0 ? bottomRadius : topRadius;
    for (let index = 0; index < segments; index += 1) {
      const theta = (Math.PI * 2 * index) / segments;
      const x = Math.cos(theta);
      const z = Math.sin(theta);
      positions.push([x * radius[0], y, z * radius[1]]);
      normals.push(normalize([x / radius[0], (bottomRadius[0] - topRadius[0]) * 0.45, z / radius[1]]));
      uvs.push([index / segments, ring]);
    }
  }

  const bottomCenter = positions.length;
  positions.push([0, -0.5, 0]);
  normals.push([0, -1, 0]);
  uvs.push([0.5, 0.5]);
  const topCenter = positions.length;
  positions.push([0, 0.5, 0]);
  normals.push([0, 1, 0]);
  uvs.push([0.5, 0.5]);

  const indices = [];
  for (let index = 0; index < segments; index += 1) {
    const next = (index + 1) % segments;
    const bottom = index;
    const bottomNext = next;
    const top = segments + index;
    const topNext = segments + next;
    indices.push(bottom, bottomNext, topNext, bottom, topNext, top);
    indices.push(bottomCenter, bottom, bottomNext);
    indices.push(topCenter, topNext, top);
  }

  return { positions, normals, uvs, indices };
}

function ellipsoidMesh(radius, segments = 14, rings = 8) {
  const positions = [];
  const normals = [];
  const uvs = [];
  for (let ring = 0; ring <= rings; ring += 1) {
    const v = ring / rings;
    const phi = -Math.PI / 2 + Math.PI * v;
    const y = Math.sin(phi);
    const flat = Math.cos(phi);
    for (let index = 0; index <= segments; index += 1) {
      const u = index / segments;
      const theta = Math.PI * 2 * u;
      const x = Math.cos(theta) * flat;
      const z = Math.sin(theta) * flat;
      positions.push([x * radius[0], y * radius[1], z * radius[2]]);
      normals.push(normalize([x / radius[0], y / radius[1], z / radius[2]]));
      uvs.push([u, v]);
    }
  }

  const indices = [];
  const stride = segments + 1;
  for (let ring = 0; ring < rings; ring += 1) {
    for (let index = 0; index < segments; index += 1) {
      const a = ring * stride + index;
      const b = a + 1;
      const c = a + stride;
      const d = c + 1;
      indices.push(a, c, b, b, c, d);
    }
  }

  return { positions, normals, uvs, indices };
}

function panelMesh(widthTop, widthBottom, height, depthOffset = 0.0) {
  const positions = [
    [-widthBottom * 0.5, -height * 0.5, depthOffset],
    [widthBottom * 0.5, -height * 0.5, depthOffset],
    [widthTop * 0.5, height * 0.5, depthOffset],
    [-widthTop * 0.5, height * 0.5, depthOffset],
  ];
  return {
    positions,
    normals: positions.map(() => [0, 0, -1]),
    uvs: [[0, 0], [1, 0], [1, 1], [0, 1]],
    indices: [0, 1, 2, 0, 2, 3],
  };
}

function crystalMesh() {
  const positions = [
    [0, 0.55, 0],
    [0.34, 0, 0],
    [0, 0, -0.18],
    [-0.34, 0, 0],
    [0, 0, 0.18],
    [0, -0.55, 0],
  ];
  return {
    positions,
    normals: positions.map(normalize),
    uvs: [[0.5, 1], [1, 0.5], [0.5, 0.5], [0, 0.5], [0.5, 0.5], [0.5, 0]],
    indices: [0, 1, 2, 0, 2, 3, 0, 3, 4, 0, 4, 1, 5, 2, 1, 5, 3, 2, 5, 4, 3, 5, 1, 4],
  };
}

addMesh("Nau Suit Tapered Hips", taperedCylinderMesh([0.34, 0.23], [0.28, 0.18]), 0);
addMesh("Nau Suit Armored Torso", taperedCylinderMesh([0.42, 0.24], [0.32, 0.19]), 0);
addMesh("Nau Accent Split Tunic Panel", panelMesh(0.36, 0.56, 0.62, -0.035), 4);
addMesh("Nau Skin Rounded Head", ellipsoidMesh([0.27, 0.31, 0.24]), 1);
addMesh("Nau Accent Helmet Crest", taperedCylinderMesh([0.20, 0.12], [0.10, 0.06], 10), 5);
addMesh("Nau Suit Upper Arm", taperedCylinderMesh([0.12, 0.11], [0.10, 0.09], 10), 0);
addMesh("Nau Leather Forearm Wrap", taperedCylinderMesh([0.10, 0.09], [0.085, 0.075], 10), 3);
addMesh("Nau Suit Thigh Guard", taperedCylinderMesh([0.14, 0.13], [0.11, 0.11], 10), 0);
addMesh("Nau Leather Boot", taperedCylinderMesh([0.14, 0.18], [0.11, 0.13], 10), 3);
addMesh("Nau Chest Focus Crystal", crystalMesh(), 2);
addMesh("Nau Accent Shoulder Guard", ellipsoidMesh([0.20, 0.08, 0.13], 10, 5), 4);
addMesh("Nau Accent Scarf Trail", panelMesh(0.16, 0.34, 0.84, 0.025), 4);
addMesh("Nau Face Mask Panel", panelMesh(0.20, 0.28, 0.15, -0.02), 5);
addMesh("Nau Amber Eye Lens", ellipsoidMesh([0.048, 0.026, 0.018], 8, 4), 6);
addMesh("Nau Belt Sash Band", taperedCylinderMesh([0.43, 0.25], [0.42, 0.24], 16), 7);
addMesh("Nau Belt Buckle Plate", crystalMesh(), 7);
addMesh("Nau Leather Gauntlet Cuff", taperedCylinderMesh([0.11, 0.095], [0.105, 0.09], 12), 3);
addMesh("Nau Accent Knee Guard", ellipsoidMesh([0.12, 0.055, 0.085], 10, 5), 4);

const nodes = [
  { name: "Nau Self Authored Animated Player Root", children: [1, 2, 3, 4, 5, 6, 7, 8, 13, 14, 18] },
  { name: "Nau Hips", mesh: 0, translation: [0, 0.54, 0], scale: [1.0, 0.76, 1.0], children: [23, 24] },
  { name: "Nau Torso", mesh: 1, translation: [0, 1.08, 0], scale: [1.0, 0.94, 1.0], children: [15, 16, 19] },
  { name: "Nau Head", mesh: 3, translation: [0, 1.68, 0], children: [17, 20, 21, 22] },
  { name: "Nau Left Arm", mesh: 5, translation: [-0.48, 1.18, 0.01], scale: [1.0, 0.62, 1.0], children: [9] },
  { name: "Nau Right Arm", mesh: 5, translation: [0.48, 1.18, 0.01], scale: [1.0, 0.62, 1.0], children: [10] },
  { name: "Nau Left Leg", mesh: 7, translation: [-0.17, 0.30, 0.01], scale: [1.0, 0.68, 1.0], children: [11, 27] },
  { name: "Nau Right Leg", mesh: 7, translation: [0.17, 0.30, 0.01], scale: [1.0, 0.68, 1.0], children: [12, 28] },
  { name: "Nau Chest Focus", mesh: 9, translation: [0, 1.15, -0.24], scale: [0.35, 0.24, 0.6] },
  { name: "Nau Left Forearm", mesh: 6, translation: [0, -0.44, 0.02], scale: [1.0, 0.52, 1.0], children: [25] },
  { name: "Nau Right Forearm", mesh: 6, translation: [0, -0.44, 0.02], scale: [1.0, 0.52, 1.0], children: [26] },
  { name: "Nau Left Boot", mesh: 8, translation: [0, -0.46, -0.01], scale: [1.0, 0.36, 1.0] },
  { name: "Nau Right Boot", mesh: 8, translation: [0, -0.46, -0.01], scale: [1.0, 0.36, 1.0] },
  { name: "Nau Front Accent Tunic", mesh: 2, translation: [0, 0.78, -0.24], scale: [1.0, 1.0, 1.0] },
  { name: "Nau Rear Accent Tunic", mesh: 2, translation: [0, 0.78, 0.21], rotation: [0, 1, 0, 0], scale: [0.82, 0.9, 1.0] },
  { name: "Nau Left Shoulder Accent", mesh: 10, translation: [-0.35, 0.22, -0.01], rotation: rotZ(-0.24) },
  { name: "Nau Right Shoulder Accent", mesh: 10, translation: [0.35, 0.22, -0.01], rotation: rotZ(0.24) },
  { name: "Nau Helmet Accent Crest", mesh: 4, translation: [0, 0.20, -0.02], rotation: rotX(0.16), scale: [1.0, 0.32, 1.0] },
  { name: "Nau Wind Scarf Accent", mesh: 11, translation: [0.24, 1.18, 0.26], rotation: rotX(-0.55), scale: [1.0, 1.0, 1.0] },
  { name: "Nau Back Scarf Anchor Accent", mesh: 11, translation: [0, 0.0, 0.25], rotation: rotX(-1.24), scale: [0.74, 0.42, 1.0] },
  { name: "Nau Face Mask Panel", mesh: 12, translation: [0, -0.02, -0.245], scale: [1.0, 0.72, 1.0] },
  { name: "Nau Left Amber Eye Lens", mesh: 13, translation: [-0.075, 0.035, -0.255], scale: [1.0, 1.0, 1.0] },
  { name: "Nau Right Amber Eye Lens", mesh: 13, translation: [0.075, 0.035, -0.255], scale: [1.0, 1.0, 1.0] },
  { name: "Nau Belt Sash Band", mesh: 14, translation: [0, 0.28, -0.005], scale: [1.0, 0.12, 1.0] },
  { name: "Nau Belt Buckle Plate", mesh: 15, translation: [0, 0.28, -0.245], rotation: rotX(1.5708), scale: [0.20, 0.12, 0.11] },
  { name: "Nau Left Leather Gauntlet Cuff", mesh: 16, translation: [0, -0.13, 0.01], scale: [1.0, 0.20, 1.0] },
  { name: "Nau Right Leather Gauntlet Cuff", mesh: 16, translation: [0, -0.13, 0.01], scale: [1.0, 0.20, 1.0] },
  { name: "Nau Left Accent Knee Guard", mesh: 17, translation: [0, -0.19, -0.10], rotation: rotX(0.08), scale: [1.0, 1.0, 1.0] },
  { name: "Nau Right Accent Knee Guard", mesh: 17, translation: [0, -0.19, -0.10], rotation: rotX(0.08), scale: [1.0, 1.0, 1.0] },
];

function quat(axis, radians) {
  const half = radians / 2;
  const s = Math.sin(half);
  return [axis[0] * s, axis[1] * s, axis[2] * s, Math.cos(half)];
}

function rotX(radians) {
  return quat([1, 0, 0], radians);
}

function rotZ(radians) {
  return quat([0, 0, 1], radians);
}

function animation(name, tracks) {
  const samplers = [];
  const channels = [];
  for (const track of tracks) {
    const input = gltfBuffer.addFloatAccessor(track.times, "SCALAR", [track.times[0]], [track.times.at(-1)]);
    const outputValues = track.values.flat();
    const outputType = track.path === "rotation" ? "VEC4" : "VEC3";
    const output = gltfBuffer.addFloatAccessor(
      outputValues,
      outputType,
      minByComponent(track.values),
      maxByComponent(track.values),
    );
    samplers.push({
      input,
      output,
      interpolation: "LINEAR",
    });
    channels.push({
      sampler: samplers.length - 1,
      target: {
        node: track.node,
        path: track.path,
      },
    });
  }
  return { name, samplers, channels };
}

function minByComponent(values) {
  const width = values[0].length;
  return Array.from({ length: width }, (_, component) =>
    Math.min(...values.map((value) => value[component])),
  );
}

function maxByComponent(values) {
  const width = values[0].length;
  return Array.from({ length: width }, (_, component) =>
    Math.max(...values.map((value) => value[component])),
  );
}

const loopTimes = [0, 0.5, 1.0];
const shortTimes = [0, 0.35, 0.8];

const animations = [
  animation("Idle_Loop", [
    { node: 3, path: "rotation", times: loopTimes, values: [rotZ(-0.04), rotZ(0.04), rotZ(-0.04)] },
    { node: 2, path: "translation", times: loopTimes, values: [[0, 1.08, 0], [0, 1.1, 0], [0, 1.08, 0]] },
  ]),
  animation("Jog_Fwd_Loop", [
    { node: 4, path: "rotation", times: loopTimes, values: [rotX(-0.45), rotX(0.45), rotX(-0.45)] },
    { node: 5, path: "rotation", times: loopTimes, values: [rotX(0.45), rotX(-0.45), rotX(0.45)] },
    { node: 6, path: "rotation", times: loopTimes, values: [rotX(0.5), rotX(-0.5), rotX(0.5)] },
    { node: 7, path: "rotation", times: loopTimes, values: [rotX(-0.5), rotX(0.5), rotX(-0.5)] },
  ]),
  animation("Launch_Start", [
    { node: 0, path: "translation", times: shortTimes, values: [[0, 0, 0], [0, 0.16, 0], [0, 0.04, 0]] },
    { node: 4, path: "rotation", times: shortTimes, values: [rotX(0.15), rotX(-0.85), rotX(-0.35)] },
    { node: 5, path: "rotation", times: shortTimes, values: [rotX(0.15), rotX(-0.85), rotX(-0.35)] },
  ]),
  animation("Glide_Loop", [
    { node: 4, path: "rotation", times: loopTimes, values: [rotZ(1.08), rotZ(1.18), rotZ(1.08)] },
    { node: 5, path: "rotation", times: loopTimes, values: [rotZ(-1.08), rotZ(-1.18), rotZ(-1.08)] },
    { node: 2, path: "rotation", times: loopTimes, values: [rotX(0.08), rotX(0.14), rotX(0.08)] },
  ]),
  animation("Air_Brake", [
    { node: 2, path: "rotation", times: shortTimes, values: [rotX(0.0), rotX(-0.24), rotX(-0.16)] },
    { node: 4, path: "rotation", times: shortTimes, values: [rotZ(0.55), rotZ(0.92), rotZ(0.72)] },
    { node: 5, path: "rotation", times: shortTimes, values: [rotZ(-0.55), rotZ(-0.92), rotZ(-0.72)] },
  ]),
  animation("Land", [
    { node: 0, path: "translation", times: shortTimes, values: [[0, 0.09, 0], [0, 0.02, 0], [0, 0, 0]] },
    { node: 6, path: "rotation", times: shortTimes, values: [rotX(-0.15), rotX(0.24), rotX(0)] },
    { node: 7, path: "rotation", times: shortTimes, values: [rotX(-0.15), rotX(0.24), rotX(0)] },
  ]),
];

const gltf = {
  asset: {
    version: "2.0",
    generator: "NAU Engine self-authored animated player fixture",
    copyright: "Self-authored for NAU Engine; no third-party assets.",
  },
  extras: {
    nau: {
      schema: NAU_FIXTURE_SCHEMA,
      asset_kind: "player_character",
      asset_label: "player character rig",
      residency: "always",
      license: NAU_FIXTURE_LICENSE,
    },
  },
  scene: 0,
  scenes: [{ name: "NAU Animated Player Fixture Scene", nodes: [0] }],
  nodes,
  meshes,
  materials: [
    {
      name: "Nau Dark Suit Material",
      pbrMetallicRoughness: {
        baseColorFactor: [0.12, 0.17, 0.25, 1.0],
        metallicFactor: 0.0,
        roughnessFactor: 0.62,
      },
    },
    {
      name: "Nau Warm Skin Material",
      pbrMetallicRoughness: {
        baseColorFactor: [0.82, 0.56, 0.39, 1.0],
        metallicFactor: 0.0,
        roughnessFactor: 0.74,
      },
    },
    {
      name: "Nau Chest Focus Material",
      emissiveFactor: [1.0, 0.48, 0.12],
      pbrMetallicRoughness: {
        baseColorFactor: [1.0, 0.62, 0.18, 1.0],
        metallicFactor: 0.0,
        roughnessFactor: 0.38,
      },
    },
    {
      name: "Nau Leather Boot And Wrap Material",
      pbrMetallicRoughness: {
        baseColorFactor: [0.18, 0.11, 0.08, 1.0],
        metallicFactor: 0.0,
        roughnessFactor: 0.82,
      },
    },
    {
      name: "Nau Teal Accent Cloth Material",
      doubleSided: true,
      pbrMetallicRoughness: {
        baseColorFactor: [0.10, 0.48, 0.54, 1.0],
        metallicFactor: 0.0,
        roughnessFactor: 0.70,
      },
    },
    {
      name: "Nau Dark Helmet Crest Material",
      pbrMetallicRoughness: {
        baseColorFactor: [0.05, 0.07, 0.09, 1.0],
        metallicFactor: 0.0,
        roughnessFactor: 0.48,
      },
    },
    {
      name: "Nau Amber Eye Lens Material",
      emissiveFactor: [0.80, 0.34, 0.08],
      pbrMetallicRoughness: {
        baseColorFactor: [0.95, 0.42, 0.12, 1.0],
        metallicFactor: 0.0,
        roughnessFactor: 0.30,
      },
    },
    {
      name: "Nau Weathered Brass Belt Material",
      pbrMetallicRoughness: {
        baseColorFactor: [0.72, 0.48, 0.18, 1.0],
        metallicFactor: 0.15,
        roughnessFactor: 0.52,
      },
    },
  ],
  animations,
};

const buffer = gltfBuffer.toBuffer();
gltf.buffers = [
  {
    uri: `data:application/octet-stream;base64,${buffer.toString("base64")}`,
    byteLength: buffer.length,
  },
];
gltf.bufferViews = gltfBuffer.bufferViews;
gltf.accessors = gltfBuffer.accessors;

mkdirSync(dirname(outPath), { recursive: true });
writeFileSync(outPath, `${JSON.stringify(gltf, null, 2)}\n`);
console.log(`wrote ${outPath}`);
