#!/usr/bin/env node

import { mkdirSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";

const TARGET_ARRAY_BUFFER = 34962;
const TARGET_ELEMENT_ARRAY_BUFFER = 34963;
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
    const bufferView = this.addBufferView([...buffer], TARGET_ARRAY_BUFFER);
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
    const bufferView = this.addBufferView([...buffer], TARGET_ELEMENT_ARRAY_BUFFER);
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

function add(a, b) {
  return [a[0] + b[0], a[1] + b[1], a[2] + b[2]];
}

function subtract(a, b) {
  return [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
}

function scale(a, value) {
  return [a[0] * value, a[1] * value, a[2] * value];
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

function makePrimitive(buffer, mesh) {
  const normals = mesh.normals ?? normalsFor(mesh.positions, mesh.indices);
  const uvs = mesh.uvs ?? mesh.positions.map((position) => [position[0], position[2]]);
  return {
    attributes: {
      POSITION: buffer.addFloatAccessor(
        flatten(mesh.positions),
        "VEC3",
        minByComponent(mesh.positions),
        maxByComponent(mesh.positions),
      ),
      NORMAL: buffer.addFloatAccessor(flatten(normals), "VEC3", [-1, -1, -1], [1, 1, 1]),
      TEXCOORD_0: buffer.addFloatAccessor(flatten(uvs), "VEC2", [0, 0], [1, 1]),
    },
    indices: buffer.addIndexAccessor(mesh.indices),
    material: mesh.material,
    mode: 4,
  };
}

function nauFixtureExtras(assetKind, assetLabel, residency) {
  return {
    nau: {
      schema: NAU_FIXTURE_SCHEMA,
      asset_kind: assetKind,
      asset_label: assetLabel,
      residency,
      license: NAU_FIXTURE_LICENSE,
    },
  };
}

function writeFixture({
  path,
  generator,
  copyright,
  assetKind,
  assetLabel,
  residency,
  materials,
  meshes,
  nodes,
}) {
  const buffer = new GltfBuffer();
  const gltfMeshes = meshes.map((mesh) => ({
    name: mesh.name,
    primitives: [makePrimitive(buffer, mesh)],
  }));
  const binary = buffer.toBuffer();
  const gltf = {
    asset: {
      version: "2.0",
      generator,
      copyright,
    },
    extras: nauFixtureExtras(assetKind, assetLabel, residency),
    scenes: [{ nodes: [0] }],
    scene: 0,
    nodes,
    meshes: gltfMeshes,
    materials,
    buffers: [
      {
        uri: `data:application/octet-stream;base64,${binary.toString("base64")}`,
        byteLength: binary.length,
      },
    ],
    bufferViews: buffer.bufferViews,
    accessors: buffer.accessors,
  };
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(path, `${JSON.stringify(gltf, null, 2)}\n`);
  console.log(`wrote ${path}`);
}

function pbrMaterial(name, color, roughness = 0.82, metallic = 0.0, extra = {}) {
  return {
    name,
    pbrMetallicRoughness: {
      baseColorFactor: color,
      metallicFactor: metallic,
      roughnessFactor: roughness,
    },
    ...extra,
  };
}

function terrainPatch(width, depth, resolution, heightFn, material) {
  const positions = [];
  const uvs = [];
  const indices = [];
  for (let z = 0; z <= resolution; z++) {
    for (let x = 0; x <= resolution; x++) {
      const u = x / resolution;
      const v = z / resolution;
      const px = (u - 0.5) * width;
      const pz = (v - 0.5) * depth;
      positions.push([px, heightFn(u, v), pz]);
      uvs.push([u, v]);
    }
  }
  const row = resolution + 1;
  for (let z = 0; z < resolution; z++) {
    for (let x = 0; x < resolution; x++) {
      const a = z * row + x;
      const b = a + 1;
      const c = a + row;
      const d = c + 1;
      indices.push(a, c, b, b, c, d);
    }
  }
  return {
    name: "Authored terrain relief surface",
    positions,
    indices,
    uvs,
    material,
  };
}

function irregularRingMesh(name, radiusX, radiusZ, topY, bottomY, segments, material) {
  const positions = [];
  const uvs = [];
  for (let i = 0; i < segments; i++) {
    const angle = (i / segments) * Math.PI * 2;
    const wobble = 1 + 0.11 * Math.sin(angle * 3.0) + 0.07 * Math.cos(angle * 5.0);
    const x = Math.cos(angle) * radiusX * wobble;
    const z = Math.sin(angle) * radiusZ * wobble;
    positions.push([x, topY + 0.08 * Math.sin(angle * 4.0), z]);
    positions.push([x * 0.62, bottomY + 0.18 * Math.cos(angle * 2.0), z * 0.62]);
    uvs.push([i / segments, 0], [i / segments, 1]);
  }
  const indices = [];
  for (let i = 0; i < segments; i++) {
    const next = (i + 1) % segments;
    const topA = i * 2;
    const bottomA = topA + 1;
    const topB = next * 2;
    const bottomB = topB + 1;
    indices.push(topA, bottomA, topB, topB, bottomA, bottomB);
  }
  return { name, positions, indices, uvs, material };
}

function discMesh(name, radiusX, radiusZ, y, segments, material) {
  const positions = [[0, y, 0]];
  const uvs = [[0.5, 0.5]];
  for (let i = 0; i < segments; i++) {
    const angle = (i / segments) * Math.PI * 2;
    const wobble = 1 + 0.08 * Math.sin(angle * 3.0);
    positions.push([Math.cos(angle) * radiusX * wobble, y, Math.sin(angle) * radiusZ * wobble]);
    uvs.push([0.5 + Math.cos(angle) * 0.5, 0.5 + Math.sin(angle) * 0.5]);
  }
  const indices = [];
  for (let i = 1; i <= segments; i++) {
    indices.push(0, i, i === segments ? 1 : i + 1);
  }
  return { name, positions, indices, uvs, material };
}

function prismBetween(name, start, end, radius, material) {
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
    name,
    positions,
    indices,
    uvs: positions.map((_, index) => [index < 4 ? 0 : 1, (index % 4) / 3]),
    material,
  };
}

function ellipsoidMesh(name, center, scaleVec, rings, segments, material) {
  const positions = [];
  const uvs = [];
  for (let ring = 0; ring <= rings; ring++) {
    const v = ring / rings;
    const phi = v * Math.PI;
    const y = Math.cos(phi);
    const radius = Math.sin(phi);
    for (let segment = 0; segment <= segments; segment++) {
      const u = segment / segments;
      const theta = u * Math.PI * 2;
      const wobble = 1 + 0.08 * Math.sin(theta * 3.0 + ring);
      positions.push([
        center[0] + Math.cos(theta) * radius * scaleVec[0] * wobble,
        center[1] + y * scaleVec[1],
        center[2] + Math.sin(theta) * radius * scaleVec[2] * wobble,
      ]);
      uvs.push([u, v]);
    }
  }
  const indices = [];
  const row = segments + 1;
  for (let ring = 0; ring < rings; ring++) {
    for (let segment = 0; segment < segments; segment++) {
      const a = ring * row + segment;
      const b = a + 1;
      const c = a + row;
      const d = c + 1;
      indices.push(a, c, b, b, c, d);
    }
  }
  return { name, positions, indices, uvs, material };
}

function leafPlane(name, center, width, height, rotation, material) {
  const cos = Math.cos(rotation);
  const sin = Math.sin(rotation);
  const local = [
    [-width, 0, -height],
    [width, 0, -height * 0.4],
    [width * 0.45, 0, height],
    [-width * 0.55, 0, height * 0.55],
  ];
  const positions = local.map(([x, y, z]) => [
    center[0] + x * cos - z * sin,
    center[1] + y,
    center[2] + x * sin + z * cos,
  ]);
  return {
    name,
    positions,
    indices: [0, 1, 2, 0, 2, 3],
    uvs: [[0, 0], [1, 0], [1, 1], [0, 1]],
    material,
  };
}

function terrainFixture() {
  const materials = [
    pbrMaterial("mossy terrain blend", [0.22, 0.48, 0.19, 1.0], 0.9),
    pbrMaterial("exposed stratified cliff", [0.38, 0.31, 0.25, 1.0], 0.86),
    pbrMaterial("mineral edge highlights", [0.62, 0.57, 0.47, 1.0], 0.78),
    pbrMaterial("landing path soil", [0.28, 0.22, 0.16, 1.0], 0.93),
    pbrMaterial("terrace scree contour", [0.46, 0.39, 0.29, 1.0], 0.91),
    pbrMaterial("erosion gully shadow soil", [0.13, 0.11, 0.08, 1.0], 0.96),
    pbrMaterial("embedded path stone caps", [0.52, 0.47, 0.37, 1.0], 0.84),
  ];
  const meshes = [
    terrainPatch(
      5.4,
      4.2,
      10,
      (u, v) =>
        0.14 * Math.sin(u * Math.PI * 3.0) +
        0.1 * Math.cos(v * Math.PI * 4.0) +
        0.05 * Math.sin((u + v) * Math.PI * 6.0),
      0,
    ),
    irregularRingMesh("authored cliff skirt", 2.95, 2.25, -0.1, -1.35, 32, 1),
    irregularRingMesh("authored underside rock mass", 2.1, 1.52, -1.16, -2.05, 28, 2),
    {
      ...terrainPatch(
        1.8,
        0.55,
        3,
        (u, v) => 0.18 + 0.02 * Math.sin(u * Math.PI * 2.0) + 0.01 * v,
        3,
      ),
      name: "readable landing soil strip",
    },
    irregularRingMesh("terrace contour ledge", 2.38, 1.78, 0.08, -0.06, 30, 4),
    prismBetween("forked erosion gully north", [-1.9, 0.16, -0.72], [-0.35, 0.13, -0.22], 0.055, 5),
    prismBetween("forked erosion gully south", [-1.74, 0.14, 0.34], [-0.18, 0.12, 0.04], 0.048, 5),
    prismBetween("curved landing path stone spine", [-0.74, 0.22, -0.43], [1.12, 0.2, -0.06], 0.075, 6),
    ellipsoidMesh("embedded path stone cap a", [-0.42, 0.25, -0.32], [0.2, 0.035, 0.12], 3, 8, 6),
    ellipsoidMesh("embedded path stone cap b", [0.4, 0.24, -0.18], [0.18, 0.035, 0.1], 3, 8, 6),
  ];
  const nodes = [
    { name: "Self Authored Island Terrain Kit", children: meshes.map((_, index) => index + 1) },
    ...meshes.map((mesh, index) => {
      const node = { name: mesh.name, mesh: index };
      if (mesh.name === "Authored terrain relief surface") {
        node.name = "terrain relief surface";
      }
      if (mesh.name === "readable landing soil strip") {
        node.translation = [0.55, 0.04, -0.25];
      }
      return node;
    }),
  ];
  writeFixture({
    path: join("assets", "models", "world", "island_terrain.gltf"),
    generator: "NAU Engine self-authored island terrain fixture generator",
    copyright: "Self-authored for NAU Engine; no third-party assets.",
    assetKind: "island_terrain",
    assetLabel: "island terrain kit",
    residency: "stream_window",
    materials,
    meshes,
    nodes,
  });
}

function foliageFixture() {
  const materials = [
    pbrMaterial("barked branch wood", [0.28, 0.17, 0.09, 1.0], 0.88),
    pbrMaterial("leaf canopy dark", [0.08, 0.32, 0.13, 1.0], 0.94),
    pbrMaterial("leaf canopy sunlit", [0.19, 0.48, 0.16, 1.0], 0.9),
    pbrMaterial("grass blade cluster", [0.22, 0.58, 0.18, 1.0], 0.96),
    pbrMaterial("leaf edge detail cards", [0.28, 0.62, 0.21, 1.0], 0.92),
    pbrMaterial("wildflower herb accent", [0.72, 0.67, 0.28, 1.0], 0.88),
    pbrMaterial("fern underside shade", [0.1, 0.24, 0.1, 1.0], 0.96),
  ];
  const meshes = [
    prismBetween("tapered main trunk", [0, 0, 0], [0.08, 1.45, 0.03], 0.07, 0),
    prismBetween("root flare north", [0.0, 0.1, 0.0], [0.42, 0.0, 0.18], 0.035, 0),
    prismBetween("root flare south", [0.0, 0.08, 0.0], [-0.34, 0.0, -0.28], 0.032, 0),
    prismBetween("left branch", [0.02, 0.9, 0.0], [-0.62, 1.25, -0.12], 0.04, 0),
    prismBetween("right branch", [0.05, 1.05, 0.02], [0.72, 1.38, 0.16], 0.035, 0),
    ellipsoidMesh("overlapping canopy core", [0.0, 1.62, 0.0], [0.58, 0.38, 0.46], 6, 10, 1),
    ellipsoidMesh("sunlit side canopy", [0.46, 1.5, 0.16], [0.4, 0.28, 0.32], 5, 10, 2),
    ellipsoidMesh("shadow side canopy", [-0.44, 1.46, -0.14], [0.42, 0.3, 0.34], 5, 10, 1),
    leafPlane("canopy serrated detail card front", [0.14, 1.72, -0.5], 0.28, 0.42, 0.28, 4),
    leafPlane("canopy serrated detail card left", [-0.53, 1.55, 0.04], 0.22, 0.38, -0.8, 4),
    leafPlane("canopy serrated detail card right", [0.58, 1.5, 0.1], 0.24, 0.36, 0.9, 4),
    leafPlane("grass fan north", [-0.55, 0.1, 0.45], 0.09, 0.42, 0.2, 3),
    leafPlane("grass fan east", [0.62, 0.08, -0.35], 0.08, 0.36, 1.2, 3),
    leafPlane("grass fan west", [-0.34, 0.07, -0.48], 0.07, 0.32, -0.9, 3),
    leafPlane("wildflower herb fan", [0.18, 0.12, 0.58], 0.08, 0.3, -0.2, 5),
    leafPlane("fern frond low shade a", [-0.72, 0.12, 0.16], 0.11, 0.42, -1.15, 6),
    leafPlane("fern frond low shade b", [0.7, 0.12, 0.28], 0.1, 0.38, 0.86, 6),
    leafPlane("hanging moss silhouette", [-0.18, 1.28, -0.46], 0.075, 0.58, 0.05, 6),
  ];
  const nodes = [
    { name: "Self Authored Foliage Kit", children: meshes.map((_, index) => index + 1) },
    ...meshes.map((mesh, index) => ({ name: mesh.name, mesh: index })),
  ];
  writeFixture({
    path: join("assets", "models", "world", "foliage.gltf"),
    generator: "NAU Engine self-authored foliage fixture generator",
    copyright: "Self-authored for NAU Engine; no third-party assets.",
    assetKind: "island_foliage",
    assetLabel: "island foliage kit",
    residency: "near_lod",
    materials,
    meshes,
    nodes,
  });
}

function waterFixture() {
  const materials = [
    pbrMaterial("clear pond surface", [0.22, 0.58, 0.66, 0.74], 0.28, 0.0, {
      alphaMode: "BLEND",
      doubleSided: true,
    }),
    pbrMaterial("foam ripple highlight", [0.78, 0.92, 0.95, 0.52], 0.34, 0.0, {
      alphaMode: "BLEND",
      doubleSided: true,
    }),
    pbrMaterial("wet stone rim", [0.19, 0.22, 0.2, 1.0], 0.82),
    pbrMaterial("reed cluster", [0.39, 0.48, 0.18, 1.0], 0.86),
    pbrMaterial("deep pond tint", [0.06, 0.23, 0.29, 0.46], 0.38, 0.0, {
      alphaMode: "BLEND",
      doubleSided: true,
    }),
    pbrMaterial("floating lily pad", [0.12, 0.34, 0.16, 1.0], 0.82),
    pbrMaterial("thin specular ripple streak", [0.9, 0.98, 1.0, 0.42], 0.24, 0.0, {
      alphaMode: "BLEND",
      doubleSided: true,
    }),
  ];
  const meshes = [
    discMesh("deep pond depth tint", 1.18, 0.58, 0.018, 32, 4),
    discMesh("oval pond plane", 1.55, 0.85, 0.02, 36, 0),
    irregularRingMesh("wet stone rim", 1.7, 0.98, 0.04, -0.05, 30, 2),
    discMesh("inner ripple ring", 0.92, 0.43, 0.055, 28, 1),
    discMesh("outer ripple ring", 1.28, 0.64, 0.06, 32, 1),
    discMesh("small wind ripple glint", 0.52, 0.18, 0.065, 18, 1),
    leafPlane("reed cluster left", [-1.25, 0.22, 0.38], 0.045, 0.58, -0.16, 3),
    leafPlane("reed cluster right", [1.13, 0.18, -0.24], 0.04, 0.46, 0.42, 3),
    discMesh("floating lily pad cluster", 0.24, 0.14, 0.074, 16, 5),
    leafPlane("thin specular ripple streak east", [0.64, 0.082, 0.18], 0.34, 0.035, 0.18, 6),
    leafPlane("thin specular ripple streak west", [-0.46, 0.083, -0.12], 0.28, 0.03, -0.36, 6),
  ];
  const nodes = [
    { name: "Self Authored Water Kit", children: meshes.map((_, index) => index + 1) },
    ...meshes.map((mesh, index) => ({ name: mesh.name, mesh: index })),
  ];
  writeFixture({
    path: join("assets", "models", "world", "water.gltf"),
    generator: "NAU Engine self-authored water fixture generator",
    copyright: "Self-authored for NAU Engine; no third-party assets.",
    assetKind: "island_water",
    assetLabel: "pond and water kit",
    residency: "near_lod",
    materials,
    meshes,
    nodes,
  });
}

function verticalRingMesh(name, radiusOuter, radiusInner, thickness, segments, material) {
  const positions = [];
  const uvs = [];
  for (let i = 0; i < segments; i++) {
    const angle = (i / segments) * Math.PI * 2;
    const c = Math.cos(angle);
    const s = Math.sin(angle);
    for (const z of [-thickness, thickness]) {
      positions.push([c * radiusOuter, s * radiusOuter, z]);
      positions.push([c * radiusInner, s * radiusInner, z]);
      uvs.push([i / segments, 0], [i / segments, 1]);
    }
  }
  const indices = [];
  for (let i = 0; i < segments; i++) {
    const next = (i + 1) % segments;
    const a = i * 4;
    const b = next * 4;
    indices.push(a, b, a + 1, a + 1, b, b + 1);
    indices.push(a + 2, a + 3, b + 2, b + 2, a + 3, b + 3);
    indices.push(a, a + 2, b, b, a + 2, b + 2);
    indices.push(a + 1, b + 1, a + 3, a + 3, b + 1, b + 3);
  }
  return { name, positions, indices, uvs, material };
}

function rocksFixture() {
  const materials = [
    pbrMaterial("weathered basalt", [0.22, 0.2, 0.18, 1.0], 0.9),
    pbrMaterial("warm exposed stone", [0.42, 0.34, 0.25, 1.0], 0.86),
    pbrMaterial("lichen highlight", [0.38, 0.48, 0.26, 1.0], 0.95),
    pbrMaterial("dark fracture seam", [0.08, 0.075, 0.07, 1.0], 0.96),
    pbrMaterial("quartz fleck highlight", [0.72, 0.68, 0.58, 1.0], 0.72),
    pbrMaterial("rust mineral seam", [0.55, 0.28, 0.16, 1.0], 0.9),
  ];
  const meshes = [
    ellipsoidMesh("large fractured boulder", [0, 0.42, 0], [0.76, 0.46, 0.58], 6, 12, 0),
    ellipsoidMesh("leaning side stone", [-0.66, 0.28, 0.14], [0.38, 0.3, 0.32], 5, 10, 1),
    ellipsoidMesh("low foreground stone", [0.62, 0.18, -0.36], [0.46, 0.2, 0.3], 5, 10, 0),
    irregularRingMesh("lichen strata band", 0.84, 0.62, 0.54, 0.46, 24, 2),
    prismBetween("dark fracture seam", [-0.18, 0.78, -0.58], [0.2, 0.24, -0.42], 0.018, 3),
    ellipsoidMesh("quartz fleck chip", [0.42, 0.48, -0.58], [0.08, 0.035, 0.06], 3, 6, 4),
    prismBetween("rust mineral seam", [-0.58, 0.5, 0.2], [-0.18, 0.18, 0.42], 0.02, 5),
    ellipsoidMesh("angular shale chip cluster", [-0.18, 0.12, 0.56], [0.18, 0.06, 0.12], 3, 7, 1),
    ellipsoidMesh("small fractured foreground stone", [0.1, 0.1, -0.72], [0.22, 0.11, 0.16], 3, 8, 0),
  ];
  const nodes = [
    { name: "Self Authored Rock Kit", children: meshes.map((_, index) => index + 1) },
    ...meshes.map((mesh, index) => ({ name: mesh.name, mesh: index })),
  ];
  writeFixture({
    path: join("assets", "models", "world", "rocks.gltf"),
    generator: "NAU Engine self-authored rock fixture generator",
    copyright: "Self-authored for NAU Engine; no third-party assets.",
    assetKind: "island_rock",
    assetLabel: "island rock kit",
    residency: "stream_window",
    materials,
    meshes,
    nodes,
  });
}

function routeMarkerFixture() {
  const materials = [
    pbrMaterial("glowing route ring", [1.0, 0.42, 0.52, 1.0], 0.36, 0.0, {
      emissiveFactor: [1.4, 0.2, 0.28],
    }),
    pbrMaterial("weathered route mast", [0.33, 0.24, 0.15, 1.0], 0.82),
    pbrMaterial("cool objective shard", [0.36, 0.76, 1.0, 1.0], 0.45, 0.0, {
      emissiveFactor: [0.1, 0.35, 0.7],
    }),
    pbrMaterial("stone base cairn", [0.27, 0.26, 0.24, 1.0], 0.88),
    pbrMaterial("weathered route pennant", [0.82, 0.28, 0.2, 0.82], 0.76, 0.0, {
      alphaMode: "BLEND",
      doubleSided: true,
    }),
    pbrMaterial("painted landing glyph", [0.95, 0.77, 0.32, 1.0], 0.7, 0.0, {
      emissiveFactor: [0.35, 0.16, 0.02],
    }),
  ];
  const meshes = [
    verticalRingMesh("route gate ring", 0.72, 0.58, 0.035, 32, 0),
    verticalRingMesh("inner readable route gate ring", 0.48, 0.42, 0.018, 28, 2),
    prismBetween("left support mast", [-0.62, -0.85, -0.02], [-0.62, 0.62, -0.02], 0.035, 1),
    prismBetween("right support mast", [0.62, -0.85, -0.02], [0.62, 0.62, -0.02], 0.035, 1),
    ellipsoidMesh("objective shard", [0.0, 0.03, -0.02], [0.12, 0.34, 0.08], 5, 8, 2),
    ellipsoidMesh("stacked cairn base", [0.0, -0.92, 0.0], [0.48, 0.18, 0.34], 4, 10, 3),
    leafPlane("weathered route pennant cloth", [0.78, 0.35, 0.02], 0.08, 0.34, -0.2, 4),
    verticalRingMesh("painted landing glyph halo", 0.32, 0.25, 0.012, 22, 5),
    leafPlane("small wind torn pennant tail", [0.9, 0.16, 0.02], 0.055, 0.24, -0.38, 4),
    ellipsoidMesh("offset cairn pebble marker", [-0.32, -0.72, 0.08], [0.18, 0.09, 0.14], 3, 8, 3),
  ];
  const nodes = [
    { name: "Self Authored Route Marker Kit", children: meshes.map((_, index) => index + 1) },
    ...meshes.map((mesh, index) => ({ name: mesh.name, mesh: index })),
  ];
  writeFixture({
    path: join("assets", "models", "world", "route_markers.gltf"),
    generator: "NAU Engine self-authored route marker fixture generator",
    copyright: "Self-authored for NAU Engine; no third-party assets.",
    assetKind: "route_marker",
    assetLabel: "route marker kit",
    residency: "always",
    materials,
    meshes,
    nodes,
  });
}

function weatherLayerFixture() {
  const materials = [
    pbrMaterial("soft cloud lobe", [0.78, 0.84, 0.88, 0.72], 0.62, 0.0, {
      alphaMode: "BLEND",
      doubleSided: true,
    }),
    pbrMaterial("shadowed cloud belly", [0.48, 0.56, 0.62, 0.68], 0.75, 0.0, {
      alphaMode: "BLEND",
      doubleSided: true,
    }),
    pbrMaterial("high cirrus veil", [0.9, 0.95, 1.0, 0.36], 0.48, 0.0, {
      alphaMode: "BLEND",
      doubleSided: true,
    }),
    pbrMaterial("feathered cloud edge wisp", [0.86, 0.91, 0.95, 0.44], 0.58, 0.0, {
      alphaMode: "BLEND",
      doubleSided: true,
    }),
    pbrMaterial("blue depth haze pocket", [0.54, 0.66, 0.78, 0.4], 0.68, 0.0, {
      alphaMode: "BLEND",
      doubleSided: true,
    }),
  ];
  const meshes = [
    ellipsoidMesh("cloud bank lobe center", [0, 0, 0], [1.2, 0.34, 0.62], 5, 12, 0),
    ellipsoidMesh("cloud bank lobe left", [-0.86, -0.04, 0.08], [0.68, 0.28, 0.42], 5, 10, 0),
    ellipsoidMesh("cloud bank lobe right", [0.92, -0.02, -0.1], [0.74, 0.3, 0.46], 5, 10, 0),
    ellipsoidMesh("cloud bank upper depth lobe", [-0.12, 0.24, 0.18], [0.64, 0.18, 0.38], 4, 10, 0),
    ellipsoidMesh("shadow belly lobe", [0.18, -0.24, 0.05], [0.95, 0.18, 0.5], 4, 10, 1),
    leafPlane("high cirrus slash a", [-0.18, 0.22, -0.02], 1.1, 0.18, 0.35, 2),
    leafPlane("high cirrus slash b", [0.34, 0.3, 0.2], 0.88, 0.12, -0.24, 2),
    leafPlane("feathered cloud edge wisp north", [-1.05, 0.04, 0.46], 0.62, 0.12, 0.72, 3),
    leafPlane("feathered cloud edge wisp south", [1.12, 0.02, -0.5], 0.58, 0.1, -0.64, 3),
    leafPlane("thin trailing cloud wisp", [0.18, -0.03, 0.72], 0.86, 0.09, 0.12, 3),
    ellipsoidMesh("blue depth haze pocket rear", [-0.32, -0.08, 0.54], [0.5, 0.12, 0.28], 4, 9, 4),
    leafPlane("feathered cloud edge wisp high", [-0.74, 0.3, -0.38], 0.66, 0.1, 0.42, 3),
    leafPlane("filament trailing curl", [0.84, 0.12, 0.62], 0.5, 0.08, -0.18, 3),
  ];
  const nodes = [
    { name: "Self Authored Weather Layer Kit", children: meshes.map((_, index) => index + 1) },
    ...meshes.map((mesh, index) => ({ name: mesh.name, mesh: index })),
  ];
  writeFixture({
    path: join("assets", "models", "world", "weather_layers.gltf"),
    generator: "NAU Engine self-authored weather layer fixture generator",
    copyright: "Self-authored for NAU Engine; no third-party assets.",
    assetKind: "weather_layer",
    assetLabel: "weather cloud layer kit",
    residency: "weather",
    materials,
    meshes,
    nodes,
  });
}

function impostorFixture() {
  const materials = [
    pbrMaterial("distant grassy cap", [0.16, 0.38, 0.18, 1.0], 0.9),
    pbrMaterial("distant cliff mass", [0.24, 0.22, 0.2, 1.0], 0.88),
    pbrMaterial("distant haze rim", [0.44, 0.56, 0.58, 0.65], 0.72, 0.0, {
      alphaMode: "BLEND",
      doubleSided: true,
    }),
    pbrMaterial("distant tree silhouettes", [0.05, 0.16, 0.08, 1.0], 0.92),
    pbrMaterial("far waterfall veil", [0.7, 0.86, 0.92, 0.48], 0.46, 0.0, {
      alphaMode: "BLEND",
      doubleSided: true,
    }),
  ];
  const meshes = [
    terrainPatch(
      3.2,
      1.5,
      5,
      (u, v) => 0.04 * Math.sin(u * Math.PI * 3.0) + 0.03 * Math.cos(v * Math.PI * 2.0),
      0,
    ),
    irregularRingMesh("distant underside silhouette", 1.72, 0.78, -0.04, -0.74, 24, 1),
    irregularRingMesh("far shadow shelf", 1.48, 0.62, -0.24, -0.42, 24, 1),
    irregularRingMesh("hazy far rim", 1.92, 0.9, 0.02, -0.08, 24, 2),
    leafPlane("distant tree silhouette cluster", [-0.74, 0.12, -0.18], 0.18, 0.46, 0.16, 3),
    leafPlane("distant ridge silhouette cluster", [0.66, 0.1, 0.12], 0.15, 0.38, -0.08, 3),
    leafPlane("far waterfall veil strip", [0.28, -0.26, -0.16], 0.08, 0.62, 0.03, 4),
    leafPlane("tiny distant tree silhouette line", [0.1, 0.18, 0.36], 0.42, 0.16, -0.08, 3),
    irregularRingMesh("broken far cliff shelf", 1.18, 0.5, -0.44, -0.58, 18, 1),
  ];
  const nodes = [
    { name: "Self Authored Island Impostor Kit", children: meshes.map((_, index) => index + 1) },
    ...meshes.map((mesh, index) => ({ name: mesh.name, mesh: index })),
  ];
  writeFixture({
    path: join("assets", "models", "world", "island_impostors.gltf"),
    generator: "NAU Engine self-authored distant impostor fixture generator",
    copyright: "Self-authored for NAU Engine; no third-party assets.",
    assetKind: "distant_impostor",
    assetLabel: "sky island distant impostor kit",
    residency: "far_lod",
    materials,
    meshes,
    nodes,
  });
}

terrainFixture();
foliageFixture();
rocksFixture();
waterFixture();
routeMarkerFixture();
weatherLayerFixture();
impostorFixture();
