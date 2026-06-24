#!/usr/bin/env node

import { mkdirSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";

const outPath = join("assets", "models", "player", "player.gltf");

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

const faceData = [
  [[0, 0, 1], [[-0.5, -0.5, 0.5], [0.5, -0.5, 0.5], [0.5, 0.5, 0.5], [-0.5, 0.5, 0.5]]],
  [[0, 0, -1], [[0.5, -0.5, -0.5], [-0.5, -0.5, -0.5], [-0.5, 0.5, -0.5], [0.5, 0.5, -0.5]]],
  [[1, 0, 0], [[0.5, -0.5, 0.5], [0.5, -0.5, -0.5], [0.5, 0.5, -0.5], [0.5, 0.5, 0.5]]],
  [[-1, 0, 0], [[-0.5, -0.5, -0.5], [-0.5, -0.5, 0.5], [-0.5, 0.5, 0.5], [-0.5, 0.5, -0.5]]],
  [[0, 1, 0], [[-0.5, 0.5, 0.5], [0.5, 0.5, 0.5], [0.5, 0.5, -0.5], [-0.5, 0.5, -0.5]]],
  [[0, -1, 0], [[-0.5, -0.5, -0.5], [0.5, -0.5, -0.5], [0.5, -0.5, 0.5], [-0.5, -0.5, 0.5]]],
];

const positions = [];
const normals = [];
const uvs = [];
const indices = [];
for (const [normal, verts] of faceData) {
  const start = positions.length / 3;
  const faceUvs = [[0, 0], [1, 0], [1, 1], [0, 1]];
  for (const vertex of verts) {
    positions.push(...vertex);
    normals.push(...normal);
  }
  for (const uv of faceUvs) {
    uvs.push(...uv);
  }
  indices.push(start, start + 1, start + 2, start, start + 2, start + 3);
}

const positionAccessor = gltfBuffer.addFloatAccessor(
  positions,
  "VEC3",
  [-0.5, -0.5, -0.5],
  [0.5, 0.5, 0.5],
);
const normalAccessor = gltfBuffer.addFloatAccessor(
  normals,
  "VEC3",
  [-1.0, -1.0, -1.0],
  [1.0, 1.0, 1.0],
);
const uvAccessor = gltfBuffer.addFloatAccessor(uvs, "VEC2", [0.0, 0.0], [1.0, 1.0]);
const indexAccessor = gltfBuffer.addIndexAccessor(indices);

const meshes = [
  cuboidMesh("Nau Suit Cuboid", 0),
  cuboidMesh("Nau Skin Cuboid", 1),
  cuboidMesh("Nau Accent Cuboid", 2),
];

function cuboidMesh(name, material) {
  return {
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
  };
}

const nodes = [
  { name: "Nau Self Authored Animated Player Root", children: [1, 2, 3, 4, 5, 6, 7, 8] },
  { name: "Nau Hips", mesh: 0, translation: [0, 0.52, 0], scale: [0.34, 0.42, 0.22] },
  { name: "Nau Torso", mesh: 0, translation: [0, 1.08, 0], scale: [0.45, 0.58, 0.24] },
  { name: "Nau Head", mesh: 1, translation: [0, 1.68, 0], scale: [0.28, 0.3, 0.26] },
  { name: "Nau Left Arm", mesh: 0, translation: [-0.56, 1.12, 0], scale: [0.13, 0.42, 0.13] },
  { name: "Nau Right Arm", mesh: 0, translation: [0.56, 1.12, 0], scale: [0.13, 0.42, 0.13] },
  { name: "Nau Left Leg", mesh: 0, translation: [-0.18, 0.25, 0], scale: [0.14, 0.5, 0.15] },
  { name: "Nau Right Leg", mesh: 0, translation: [0.18, 0.25, 0], scale: [0.14, 0.5, 0.15] },
  { name: "Nau Chest Focus", mesh: 2, translation: [0, 1.15, -0.26], scale: [0.11, 0.11, 0.05] },
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

function identityRotations(count) {
  return Array.from({ length: count }, () => [0, 0, 0, 1]);
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
