#!/usr/bin/env node

import { mkdirSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";

const outPath = join("assets", "models", "player", "glider.gltf");
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

function subtract(a, b) {
  return [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
}

function cross(a, b) {
  return [
    a[1] * b[2] - a[2] * b[1],
    a[2] * b[0] - a[0] * b[2],
    a[0] * b[1] - a[1] * b[0],
  ];
}

function dot(a, b) {
  return a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
}

function add(a, b) {
  return [a[0] + b[0], a[1] + b[1], a[2] + b[2]];
}

function scale(a, value) {
  return [a[0] * value, a[1] * value, a[2] * value];
}

function normalize(a) {
  const length = Math.hypot(a[0], a[1], a[2]);
  if (length <= 1e-6) {
    return [0, 1, 0];
  }
  return [a[0] / length, a[1] / length, a[2] / length];
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

function flatten(values) {
  return values.flatMap((value) => value);
}

function orientTrianglesUp(positions, triangles) {
  return triangles.flatMap((triangle) => {
    const [a, b, c] = triangle;
    const normal = cross(
      subtract(positions[b], positions[a]),
      subtract(positions[c], positions[a]),
    );
    return normal[1] < 0 ? [a, c, b] : [a, b, c];
  });
}

function normalsFor(positions, indices) {
  const normals = positions.map(() => [0, 0, 0]);
  for (let index = 0; index < indices.length; index += 3) {
    const a = indices[index];
    const b = indices[index + 1];
    const c = indices[index + 2];
    const normal = normalize(
      cross(subtract(positions[b], positions[a]), subtract(positions[c], positions[a])),
    );
    normals[a] = add(normals[a], normal);
    normals[b] = add(normals[b], normal);
    normals[c] = add(normals[c], normal);
  }
  return normals.map(normalize);
}

function prismBetween(start, end, radius) {
  const axis = normalize(subtract(end, start));
  const seed = Math.abs(dot(axis, [0, 1, 0])) > 0.92 ? [1, 0, 0] : [0, 1, 0];
  const side = normalize(cross(axis, seed));
  const up = normalize(cross(side, axis));
  const offsets = [
    add(scale(side, radius), scale(up, radius)),
    add(scale(side, -radius), scale(up, radius)),
    add(scale(side, -radius), scale(up, -radius)),
    add(scale(side, radius), scale(up, -radius)),
  ];
  const positions = [
    ...offsets.map((offset) => add(start, offset)),
    ...offsets.map((offset) => add(end, offset)),
  ];
  const indices = [
    0, 4, 1, 1, 4, 5,
    1, 5, 2, 2, 5, 6,
    2, 6, 3, 3, 6, 7,
    3, 7, 0, 0, 7, 4,
    0, 1, 2, 0, 2, 3,
    4, 7, 6, 4, 6, 5,
  ];
  return {
    positions,
    indices,
    normals: normalsFor(positions, indices),
    uvs: positions.map((_, index) => [index < 4 ? 0 : 1, (index % 4) / 3]),
  };
}

function clothPanel(side) {
  const s = side === "left" ? -1 : 1;
  const positions = [
    [0.0, 0.16, -0.76],
    [s * 0.62, 0.28, -0.88],
    [s * 1.35, 0.22, -0.66],
    [s * 2.34, 0.02, -0.25],
    [0.0, 0.03, 0.66],
    [s * 0.75, 0.08, 0.82],
    [s * 1.56, 0.03, 0.62],
    [s * 2.24, -0.06, 0.25],
  ];
  const indices = orientTrianglesUp(positions, [
    [0, 4, 1],
    [1, 4, 5],
    [1, 5, 2],
    [2, 5, 6],
    [2, 6, 3],
    [3, 6, 7],
  ]);
  return {
    positions,
    indices,
    normals: normalsFor(positions, indices),
    uvs: [
      [0.0, 0.0],
      [0.28, 0.0],
      [0.62, 0.0],
      [1.0, 0.0],
      [0.0, 1.0],
      [0.32, 1.0],
      [0.68, 1.0],
      [1.0, 1.0],
    ],
  };
}

function seamStrip(side) {
  const s = side === "left" ? -1 : 1;
  const positions = [
    [s * 0.03, 0.18, -0.73],
    [s * 0.16, 0.16, -0.72],
    [s * 0.12, 0.03, 0.66],
    [s * 0.02, 0.03, 0.65],
  ];
  const indices = orientTrianglesUp(positions, [
    [0, 3, 1],
    [1, 3, 2],
  ]);
  return {
    positions,
    indices,
    normals: normalsFor(positions, indices),
    uvs: [[0, 0], [1, 0], [1, 1], [0, 1]],
  };
}

const gltfBuffer = new GltfBuffer();
const meshes = [];

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
    [-1, -1, -1],
    [1, 1, 1],
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

addMesh("Nau Glider Left Cloth Panel", clothPanel("left"), 0);
addMesh("Nau Glider Right Cloth Panel", clothPanel("right"), 0);
addMesh("Nau Glider Left Center Seam", seamStrip("left"), 1);
addMesh("Nau Glider Right Center Seam", seamStrip("right"), 1);
addMesh("Nau Glider Leading Spar", prismBetween([-2.35, 0.06, -0.27], [2.35, 0.06, -0.27], 0.035), 2);
addMesh("Nau Glider Rear Spar", prismBetween([-2.22, -0.05, 0.34], [2.22, -0.05, 0.34], 0.026), 2);
addMesh("Nau Glider Center Keel", prismBetween([0.0, 0.05, -0.82], [0.0, -0.20, 1.04], 0.03), 2);
addMesh("Nau Glider Left Rib", prismBetween([-0.1, 0.09, -0.63], [-2.08, -0.03, 0.20], 0.022), 2);
addMesh("Nau Glider Right Rib", prismBetween([0.1, 0.09, -0.63], [2.08, -0.03, 0.20], 0.022), 2);
addMesh("Nau Glider Left Tether", prismBetween([-0.5, -0.55, 0.38], [-1.42, -0.02, 0.15], 0.014), 3);
addMesh("Nau Glider Right Tether", prismBetween([0.5, -0.55, 0.38], [1.42, -0.02, 0.15], 0.014), 3);
addMesh("Nau Glider Handle Bar", prismBetween([-0.44, -0.62, 0.42], [0.44, -0.62, 0.42], 0.034), 4);
addMesh("Nau Glider Left Grip", prismBetween([-0.52, -0.70, 0.43], [-0.52, -0.48, 0.43], 0.03), 4);
addMesh("Nau Glider Right Grip", prismBetween([0.52, -0.70, 0.43], [0.52, -0.48, 0.43], 0.03), 4);

const nodes = [
  {
    name: "NAU Authored Glider Root",
    children: meshes.map((_, index) => index + 1),
  },
  ...meshes.map((mesh, index) => ({
    name: mesh.name.replace(" Mesh", ""),
    mesh: index,
  })),
];

const gltf = {
  asset: {
    version: "2.0",
    generator: "NAU Engine self-authored glider fixture generator",
    copyright: "Self-authored for NAU Engine; no third-party assets.",
  },
  extras: {
    nau: {
      schema: NAU_FIXTURE_SCHEMA,
      asset_kind: "glider",
      asset_label: "player glider",
      residency: "always",
      license: NAU_FIXTURE_LICENSE,
    },
  },
  scene: 0,
  scenes: [
    {
      name: "NAU Authored Glider Fixture Scene",
      nodes: [0],
    },
  ],
  nodes,
  meshes,
  materials: [
    {
      name: "Nau Glider Layered Blue Cloth",
      doubleSided: true,
      alphaMode: "BLEND",
      pbrMetallicRoughness: {
        baseColorFactor: [0.18, 0.46, 0.78, 0.88],
        metallicFactor: 0.0,
        roughnessFactor: 0.72,
      },
    },
    {
      name: "Nau Glider Bright Cloth Seam",
      emissiveFactor: [0.08, 0.22, 0.35],
      pbrMetallicRoughness: {
        baseColorFactor: [0.74, 0.92, 1.0, 1.0],
        metallicFactor: 0.0,
        roughnessFactor: 0.58,
      },
    },
    {
      name: "Nau Glider Dark Wood Frame",
      pbrMetallicRoughness: {
        baseColorFactor: [0.23, 0.14, 0.08, 1.0],
        metallicFactor: 0.0,
        roughnessFactor: 0.64,
      },
    },
    {
      name: "Nau Glider Braided Tether",
      pbrMetallicRoughness: {
        baseColorFactor: [0.70, 0.56, 0.36, 1.0],
        metallicFactor: 0.0,
        roughnessFactor: 0.82,
      },
    },
    {
      name: "Nau Glider Leather Handle",
      pbrMetallicRoughness: {
        baseColorFactor: [0.16, 0.10, 0.07, 1.0],
        metallicFactor: 0.0,
        roughnessFactor: 0.76,
      },
    },
  ],
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
