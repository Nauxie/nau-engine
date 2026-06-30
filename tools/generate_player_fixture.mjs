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

function roundedTaperedCylinderMesh(bottomRadius, topRadius, segments = 32, rings = 8) {
  const positions = [];
  const normals = [];
  const uvs = [];
  for (let ring = 0; ring <= rings; ring += 1) {
    const t = ring / rings;
    const y = -0.5 + t;
    const centerFullness = 0.86 + Math.sin(Math.PI * t) * 0.14;
    const radiusX = (bottomRadius[0] + (topRadius[0] - bottomRadius[0]) * t) * centerFullness;
    const radiusZ = (bottomRadius[1] + (topRadius[1] - bottomRadius[1]) * t) * centerFullness;
    const normalY = (bottomRadius[0] - topRadius[0]) * 0.25 + (0.5 - t) * 0.10;
    for (let index = 0; index < segments; index += 1) {
      const theta = (Math.PI * 2 * index) / segments;
      const x = Math.cos(theta);
      const z = Math.sin(theta);
      positions.push([x * radiusX, y, z * radiusZ]);
      normals.push(normalize([x / radiusX, normalY, z / radiusZ]));
      uvs.push([index / segments, t]);
    }
  }

  const bottomCenter = positions.length;
  positions.push([0, -0.5, 0]);
  normals.push([0, -1, 0]);
  uvs.push([0.5, 0.0]);
  const topCenter = positions.length;
  positions.push([0, 0.5, 0]);
  normals.push([0, 1, 0]);
  uvs.push([0.5, 1.0]);

  const indices = [];
  for (let ring = 0; ring < rings; ring += 1) {
    const current = ring * segments;
    const nextRing = (ring + 1) * segments;
    for (let index = 0; index < segments; index += 1) {
      const next = (index + 1) % segments;
      indices.push(current + index, current + next, nextRing + next);
      indices.push(current + index, nextRing + next, nextRing + index);
    }
  }
  for (let index = 0; index < segments; index += 1) {
    const next = (index + 1) % segments;
    const top = rings * segments;
    indices.push(bottomCenter, index, next);
    indices.push(topCenter, top + next, top + index);
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

function boxMesh(width, height, depth) {
  const x = width * 0.5;
  const y = height * 0.5;
  const z = depth * 0.5;
  const faces = [
    { normal: [0, 0, -1], corners: [[-x, -y, -z], [x, -y, -z], [x, y, -z], [-x, y, -z]] },
    { normal: [0, 0, 1], corners: [[x, -y, z], [-x, -y, z], [-x, y, z], [x, y, z]] },
    { normal: [-1, 0, 0], corners: [[-x, -y, z], [-x, -y, -z], [-x, y, -z], [-x, y, z]] },
    { normal: [1, 0, 0], corners: [[x, -y, -z], [x, -y, z], [x, y, z], [x, y, -z]] },
    { normal: [0, 1, 0], corners: [[-x, y, -z], [x, y, -z], [x, y, z], [-x, y, z]] },
    { normal: [0, -1, 0], corners: [[-x, -y, z], [x, -y, z], [x, -y, -z], [-x, -y, -z]] },
  ];
  const positions = [];
  const normals = [];
  const uvs = [];
  const indices = [];

  for (const face of faces) {
    const start = positions.length;
    positions.push(...face.corners);
    normals.push(face.normal, face.normal, face.normal, face.normal);
    uvs.push([0, 0], [1, 0], [1, 1], [0, 1]);
    indices.push(start, start + 1, start + 2, start, start + 2, start + 3);
  }

  return { positions, normals, uvs, indices };
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

addMesh("Nau Suit Tapered Hips", taperedCylinderMesh([0.29, 0.205], [0.245, 0.165], 44), 0);
addMesh("Nau Suit Armored Torso", taperedCylinderMesh([0.36, 0.205], [0.285, 0.165], 44), 0);
addMesh("Nau Suit Ribcage Soft Volume", ellipsoidMesh([0.370, 0.390, 0.220], 48, 18), 0);
addMesh("Nau Suit Lower Rib Flex Lip", ellipsoidMesh([0.310, 0.084, 0.178], 38, 11), 0);
addMesh("Nau Suit Waist Soft Volume", ellipsoidMesh([0.300, 0.175, 0.190], 44, 14), 0);
addMesh("Nau Suit Abdominal Flex Gasket", ellipsoidMesh([0.315, 0.120, 0.185], 40, 12), 0);
addMesh("Nau Suit Oblique Flex Connector", ellipsoidMesh([0.104, 0.092, 0.074], 30, 10), 0);
addMesh("Nau Suit Pectoral Soft Volume", ellipsoidMesh([0.170, 0.052, 0.070], 34, 10), 0);
addMesh("Nau Suit Scapula Soft Volume", ellipsoidMesh([0.122, 0.044, 0.066], 30, 9), 0);
addMesh("Nau Accent Split Tunic Panel", panelMesh(0.34, 0.52, 0.58, -0.04), 4);
addMesh("Nau Skin Rounded Head", ellipsoidMesh([0.205, 0.255, 0.190], 42, 18), 1);
addMesh("Nau Skin Neck Column", roundedTaperedCylinderMesh([0.082, 0.068], [0.074, 0.060], 30, 7), 1);
addMesh("Nau Suit Neck Collar Pad", ellipsoidMesh([0.178, 0.052, 0.132], 32, 10), 0);
addMesh("Nau Accent Helmet Crest", taperedCylinderMesh([0.135, 0.078], [0.070, 0.042], 28), 5);
addMesh("Nau Suit Upper Arm", roundedTaperedCylinderMesh([0.118, 0.100], [0.092, 0.080], 46, 13), 0);
addMesh("Nau Leather Forearm Wrap", roundedTaperedCylinderMesh([0.102, 0.086], [0.076, 0.064], 46, 13), 3);
addMesh("Nau Suit Thigh Guard", roundedTaperedCylinderMesh([0.136, 0.118], [0.102, 0.090], 46, 13), 0);
addMesh("Nau Leather Boot", roundedTaperedCylinderMesh([0.122, 0.128], [0.098, 0.098], 38, 9), 3);
addMesh("Nau Chest Focus Crystal", crystalMesh(), 2);
addMesh("Nau Accent Shoulder Guard", ellipsoidMesh([0.142, 0.048, 0.096], 30, 10), 4);
addMesh("Nau Accent Scarf Trail", panelMesh(0.14, 0.30, 0.80, 0.025), 4);
addMesh("Nau Face Mask Panel", panelMesh(0.19, 0.26, 0.14, -0.02), 5);
addMesh("Nau Amber Eye Lens", ellipsoidMesh([0.045, 0.024, 0.017], 20, 8), 6);
addMesh("Nau Belt Sash Band", taperedCylinderMesh([0.41, 0.235], [0.40, 0.225], 40), 7);
addMesh("Nau Belt Buckle Plate", crystalMesh(), 7);
addMesh("Nau Leather Gauntlet Cuff", taperedCylinderMesh([0.084, 0.072], [0.078, 0.068], 30), 3);
addMesh("Nau Accent Knee Guard", ellipsoidMesh([0.088, 0.038, 0.064], 30, 10), 4);
addMesh("Nau Leather Hand Palm", ellipsoidMesh([0.094, 0.064, 0.088], 32, 12), 3);
addMesh("Nau Leather Finger Grip", roundedTaperedCylinderMesh([0.029, 0.022], [0.022, 0.016], 14, 5), 3);
addMesh("Nau Leather Finger Tip Pad", ellipsoidMesh([0.025, 0.017, 0.020], 16, 7), 3);
addMesh("Nau Leather Boot Toe Cap", ellipsoidMesh([0.120, 0.040, 0.130], 30, 12), 3);
addMesh("Nau Leather Boot Toe Lug", ellipsoidMesh([0.044, 0.020, 0.060], 18, 7), 3);
addMesh("Nau Accent Side Tunic Flap", panelMesh(0.12, 0.24, 0.52, -0.02), 4);
addMesh("Nau Suit Neck Gasket", taperedCylinderMesh([0.15, 0.105], [0.13, 0.092], 28), 0);
addMesh("Nau Accent Elbow Guard", ellipsoidMesh([0.085, 0.036, 0.060], 26, 9), 4);
addMesh("Nau Leather Ankle Wrap", taperedCylinderMesh([0.112, 0.12], [0.102, 0.108], 28), 3);
addMesh("Nau Suit Lower Leg Greave", roundedTaperedCylinderMesh([0.094, 0.100], [0.074, 0.084], 38, 10), 0);
addMesh("Nau Joint Shoulder Socket", ellipsoidMesh([0.138, 0.088, 0.120], 34, 12), 4);
addMesh("Nau Joint Hip Socket", ellipsoidMesh([0.120, 0.082, 0.110], 34, 12), 4);
addMesh("Nau Joint Knee Sleeve", taperedCylinderMesh([0.108, 0.084], [0.102, 0.080], 28), 3);
addMesh("Nau Joint Wrist Sleeve", taperedCylinderMesh([0.080, 0.066], [0.074, 0.060], 28), 3);
addMesh("Nau Cloth Harness Strap", panelMesh(0.070, 0.090, 0.76, -0.01), 7);
addMesh("Nau Suit Shoulder Bridge Sleeve", roundedTaperedCylinderMesh([0.096, 0.078], [0.084, 0.066], 38, 9), 0);
addMesh("Nau Suit Elbow Bridge Sleeve", roundedTaperedCylinderMesh([0.070, 0.055], [0.060, 0.048], 36, 8), 0);
addMesh("Nau Leather Wrist Bridge Sleeve", roundedTaperedCylinderMesh([0.041, 0.034], [0.036, 0.030], 32, 7), 3);
addMesh("Nau Suit Hip Bridge Sleeve", roundedTaperedCylinderMesh([0.088, 0.070], [0.078, 0.060], 36, 8), 0);
addMesh("Nau Suit Knee Bridge Sleeve", roundedTaperedCylinderMesh([0.072, 0.056], [0.062, 0.050], 36, 8), 0);
addMesh("Nau Leather Ankle Bridge Sleeve", roundedTaperedCylinderMesh([0.042, 0.036], [0.036, 0.032], 28, 5), 3);
addMesh("Nau Suit Shoulder Yoke Plate", ellipsoidMesh([0.52, 0.064, 0.148], 38, 11), 0);
addMesh("Nau Suit Collarbone Plate", ellipsoidMesh([0.16, 0.030, 0.045], 24, 8), 7);
addMesh("Nau Suit Pelvis Hip Yoke", ellipsoidMesh([0.305, 0.044, 0.126], 34, 10), 0);
addMesh("Nau Leather Knuckle Pad", ellipsoidMesh([0.026, 0.014, 0.018], 14, 6), 3);
addMesh("Nau Leather Boot Sole", boxMesh(0.24, 0.036, 0.34), 3);
addMesh("Nau Leather Boot Heel", boxMesh(0.20, 0.078, 0.14), 3);
addMesh("Nau Suit Deltoid Filler", ellipsoidMesh([0.142, 0.074, 0.106], 30, 10), 0);
addMesh("Nau Suit Pelvis Side Plate", ellipsoidMesh([0.105, 0.040, 0.095], 24, 8), 7);
addMesh("Nau Leather Palm Heel Pad", ellipsoidMesh([0.052, 0.020, 0.040], 18, 7), 3);
addMesh("Nau Suit Bicep Volume", ellipsoidMesh([0.070, 0.150, 0.054], 28, 10), 0);
addMesh("Nau Suit Tricep Sweep", ellipsoidMesh([0.052, 0.160, 0.046], 24, 9), 0);
addMesh("Nau Leather Forearm Tendon Strap", boxMesh(0.050, 0.260, 0.024), 3);
addMesh("Nau Leather Finger Web Bridge", boxMesh(0.205, 0.030, 0.040), 3);
addMesh("Nau Suit Shoulder Root Blend", ellipsoidMesh([0.160, 0.058, 0.104], 38, 12), 0);
addMesh("Nau Suit Shoulder Chest Blend", ellipsoidMesh([0.150, 0.030, 0.060], 28, 8), 7);
addMesh("Nau Suit Thigh Sweep", ellipsoidMesh([0.066, 0.185, 0.058], 28, 10), 0);
addMesh("Nau Suit Hip Root Blend", ellipsoidMesh([0.126, 0.056, 0.098], 38, 12), 0);
addMesh("Nau Suit Calf Volume", ellipsoidMesh([0.068, 0.170, 0.060], 28, 10), 0);
addMesh("Nau Suit Shin Ridge", boxMesh(0.052, 0.275, 0.030), 0);
addMesh("Nau Suit Knee Tendon Strap", boxMesh(0.046, 0.220, 0.026), 0);
addMesh("Nau Leather Wrist Palm Gusset", ellipsoidMesh([0.082, 0.034, 0.055], 24, 8), 3);
addMesh("Nau Leather Boot Instep Plate", boxMesh(0.195, 0.034, 0.226), 3);
addMesh("Nau Leather Lace Cross Strap", boxMesh(0.048, 0.030, 0.255), 7);
addMesh("Nau Leather Heel Tendon Guard", boxMesh(0.105, 0.190, 0.034), 3);
addMesh("Nau Leather Ankle Boot Tongue", boxMesh(0.120, 0.170, 0.026), 3);
addMesh("Nau Seamless Shoulder Flex Cover", ellipsoidMesh([0.140, 0.078, 0.124], 40, 14), 0);
addMesh("Nau Seamless Elbow Flex Cover", ellipsoidMesh([0.086, 0.058, 0.072], 36, 12), 0);
addMesh("Nau Seamless Wrist Flex Cover", ellipsoidMesh([0.070, 0.046, 0.062], 34, 11), 3);
addMesh("Nau Seamless Hip Flex Cover", ellipsoidMesh([0.116, 0.056, 0.102], 40, 14), 0);
addMesh("Nau Seamless Knee Flex Cover", ellipsoidMesh([0.080, 0.034, 0.068], 36, 12), 0);
addMesh("Nau Seamless Ankle Flex Cover", ellipsoidMesh([0.086, 0.054, 0.078], 34, 11), 3);
addMesh("Nau Suit Axilla Blend", ellipsoidMesh([0.118, 0.046, 0.082], 30, 10), 0);
addMesh("Nau Suit Hip Inguinal Blend", ellipsoidMesh([0.104, 0.040, 0.086], 30, 10), 0);
addMesh("Nau Suit Shoulder Web Capsule", ellipsoidMesh([0.178, 0.070, 0.118], 38, 12), 0);
addMesh("Nau Suit Hip Web Capsule", ellipsoidMesh([0.142, 0.060, 0.112], 38, 12), 0);
addMesh("Nau Suit Lat Shoulder Connector", ellipsoidMesh([0.132, 0.052, 0.086], 30, 10), 0);
addMesh("Nau Suit Glute Hip Connector", ellipsoidMesh([0.120, 0.052, 0.090], 30, 10), 0);
addMesh("Nau Suit Hip Thigh Fairing", ellipsoidMesh([0.112, 0.046, 0.078], 34, 10), 0);
addMesh("Nau Leather Palm Edge Pad", ellipsoidMesh([0.060, 0.030, 0.048], 18, 7), 3);
addMesh("Nau Leather Thumb Web Pad", ellipsoidMesh([0.052, 0.028, 0.044], 18, 7), 3);
addMesh("Nau Leather Boot Side Guard", ellipsoidMesh([0.055, 0.030, 0.112], 24, 8), 3);
addMesh("Nau Leather Boot Arch Rib", boxMesh(0.050, 0.038, 0.320), 7);

function meshIndex(name) {
  const index = meshes.findIndex((mesh) => mesh.name === name);
  if (index < 0) {
    throw new Error(`unknown mesh: ${name}`);
  }
  return index;
}

const nodes = [];
const nodeIds = {};

function addNode(name, fields = {}) {
  const node = { name, ...fields };
  nodes.push(node);
  return nodes.length - 1;
}

function addChild(parent, name, fields = {}) {
  const id = addNode(name, fields);
  nodes[parent].children ??= [];
  nodes[parent].children.push(id);
  return id;
}

function addMeshChild(parent, name, mesh, fields = {}) {
  return addChild(parent, name, { mesh, ...fields });
}

nodeIds.root = addNode("Nau Self Authored Animated Player Root");
nodeIds.hips = addChild(nodeIds.root, "Nau Hips", { translation: [0.0, 0.82, 0.0] });
addMeshChild(nodeIds.hips, "Nau Suit Tapered Hips Shell", meshIndex("Nau Suit Tapered Hips"), {
  translation: [0.0, 0.02, 0.0],
  scale: [1.0, 0.34, 1.0],
});
addMeshChild(nodeIds.hips, "Nau Suit Pelvis Hip Yoke", meshIndex("Nau Suit Pelvis Hip Yoke"), {
  translation: [0.0, -0.130, -0.005],
  scale: [1.0, 0.78, 1.0],
});
addMeshChild(nodeIds.hips, "Nau Suit Waist Soft Volume", meshIndex("Nau Suit Waist Soft Volume"), {
  translation: [0.0, 0.185, -0.004],
  scale: [1.0, 0.72, 1.0],
});
addMeshChild(nodeIds.hips, "Nau Belt Sash Band", meshIndex("Nau Belt Sash Band"), {
  translation: [0.0, 0.10, -0.005],
  scale: [1.0, 0.12, 1.0],
});
addMeshChild(nodeIds.hips, "Nau Belt Buckle Plate", meshIndex("Nau Belt Buckle Plate"), {
  translation: [0.0, 0.10, -0.25],
  rotation: rotX(1.5708),
  scale: [0.20, 0.12, 0.11],
});

nodeIds.torso = addChild(nodeIds.hips, "Nau Torso", { translation: [0.0, 0.39, 0.0] });
addMeshChild(nodeIds.torso, "Nau Suit Armored Torso Shell", meshIndex("Nau Suit Armored Torso"), {
  translation: [0.0, 0.20, -0.005],
  scale: [1.0, 0.78, 1.0],
});
addMeshChild(nodeIds.torso, "Nau Suit Ribcage Soft Volume", meshIndex("Nau Suit Ribcage Soft Volume"), {
  translation: [0.0, 0.205, 0.000],
  scale: [1.0, 0.90, 1.0],
});
addMeshChild(nodeIds.torso, "Nau Suit Lower Rib Flex Lip", meshIndex("Nau Suit Lower Rib Flex Lip"), {
  translation: [0.0, -0.078, -0.004],
  scale: [0.96, 0.72, 0.96],
});
addMeshChild(nodeIds.hips, "Nau Suit Abdominal Flex Gasket", meshIndex("Nau Suit Abdominal Flex Gasket"), {
  translation: [0.0, 0.245, -0.002],
  scale: [0.94, 0.64, 0.94],
});
addMeshChild(nodeIds.torso, "Nau Suit Shoulder Yoke Plate", meshIndex("Nau Suit Shoulder Yoke Plate"), {
  translation: [0.0, 0.515, -0.018],
  scale: [1.0, 1.0, 1.0],
});
addMeshChild(nodeIds.torso, "Nau Left Suit Collarbone Plate", meshIndex("Nau Suit Collarbone Plate"), {
  translation: [-0.21, 0.45, -0.145],
  rotation: rotZ(-0.24),
});
addMeshChild(nodeIds.torso, "Nau Right Suit Collarbone Plate", meshIndex("Nau Suit Collarbone Plate"), {
  translation: [0.21, 0.45, -0.145],
  rotation: rotZ(0.24),
});
addMeshChild(nodeIds.torso, "Nau Chest Focus", meshIndex("Nau Chest Focus Crystal"), {
  translation: [0.0, 0.24, -0.245],
  scale: [0.35, 0.24, 0.6],
});
addMeshChild(nodeIds.torso, "Nau Front Accent Tunic", meshIndex("Nau Accent Split Tunic Panel"), {
  translation: [0.0, -0.18, -0.24],
});
addMeshChild(nodeIds.torso, "Nau Rear Accent Tunic", meshIndex("Nau Accent Split Tunic Panel"), {
  translation: [0.0, -0.16, 0.21],
  rotation: [0, 1, 0, 0],
  scale: [0.82, 0.9, 1.0],
});
addMeshChild(nodeIds.torso, "Nau Cloth Harness Front Strap", meshIndex("Nau Cloth Harness Strap"), {
  translation: [-0.13, 0.18, -0.255],
  rotation: rotZ(-0.18),
});
addMeshChild(nodeIds.torso, "Nau Cloth Harness Rear Strap", meshIndex("Nau Cloth Harness Strap"), {
  translation: [0.13, 0.18, 0.235],
  rotation: rotZ(0.18),
});

nodeIds.neckSocket = addChild(nodeIds.torso, "Nau Neck Socket", {
  translation: [0.0, 0.78, -0.02],
});
addMeshChild(nodeIds.neckSocket, "Nau Neck Joint Cover", meshIndex("Nau Suit Neck Gasket"), {
  translation: [0.0, -0.230, 0.01],
  scale: [0.48, 0.078, 0.48],
});
addMeshChild(nodeIds.neckSocket, "Nau Skin Neck Column", meshIndex("Nau Skin Neck Column"), {
  translation: [0.0, -0.190, -0.016],
  scale: [0.84, 0.17, 0.84],
});
addMeshChild(nodeIds.neckSocket, "Nau Suit Neck Collar Pad", meshIndex("Nau Suit Neck Collar Pad"), {
  translation: [0.0, -0.245, -0.002],
  scale: [1.0, 0.72, 1.0],
});
nodeIds.head = addChild(nodeIds.torso, "Nau Head", { translation: [0.0, 0.78, -0.02] });
addMeshChild(nodeIds.head, "Nau Skin Rounded Head", meshIndex("Nau Skin Rounded Head"), {
  translation: [0.0, 0.06, -0.02],
});
addMeshChild(nodeIds.head, "Nau Helmet Accent Crest", meshIndex("Nau Accent Helmet Crest"), {
  translation: [0.0, 0.250, -0.02],
  rotation: rotX(0.16),
  scale: [1.0, 0.32, 1.0],
});
addMeshChild(nodeIds.head, "Nau Face Mask Panel", meshIndex("Nau Face Mask Panel"), {
  translation: [0.0, 0.02, -0.265],
  scale: [1.0, 0.72, 1.0],
});
addMeshChild(nodeIds.head, "Nau Left Amber Eye Lens", meshIndex("Nau Amber Eye Lens"), {
  translation: [-0.075, 0.075, -0.275],
});
addMeshChild(nodeIds.head, "Nau Right Amber Eye Lens", meshIndex("Nau Amber Eye Lens"), {
  translation: [0.075, 0.075, -0.275],
});

for (const side of [
  ["Left", -1],
  ["Right", 1],
]) {
  const [label, sign] = side;
  const lower = label.toLowerCase();
  addMeshChild(nodeIds.torso, `Nau ${label} Suit Shoulder Chest Blend`, meshIndex("Nau Suit Shoulder Chest Blend"), {
    translation: [sign * 0.392, 0.492, -0.106],
    rotation: rotZ(sign * 0.20),
    scale: [1.0, 1.0, 1.0],
  });
  addMeshChild(nodeIds.torso, `Nau ${label} Suit Pectoral Soft Volume`, meshIndex("Nau Suit Pectoral Soft Volume"), {
    translation: [sign * 0.186, 0.356, -0.164],
    rotation: rotZ(sign * 0.08),
    scale: [1.0, 0.96, 1.0],
  });
  addMeshChild(nodeIds.torso, `Nau ${label} Suit Scapula Soft Volume`, meshIndex("Nau Suit Scapula Soft Volume"), {
    translation: [sign * 0.226, 0.338, 0.166],
    rotation: rotZ(sign * -0.08),
    scale: [0.86, 0.82, 0.88],
  });
  addMeshChild(nodeIds.torso, `Nau ${label} Suit Axilla Blend`, meshIndex("Nau Suit Axilla Blend"), {
    translation: [sign * 0.420, 0.438, -0.010],
    rotation: rotZ(sign * 0.30),
    scale: [1.0, 0.92, 1.0],
  });
  addMeshChild(nodeIds.torso, `Nau ${label} Suit Shoulder Web Capsule`, meshIndex("Nau Suit Shoulder Web Capsule"), {
    translation: [sign * 0.462, 0.486, -0.014],
    rotation: rotZ(sign * 0.24),
    scale: [1.0, 0.96, 1.0],
  });
  addMeshChild(nodeIds.hips, `Nau ${label} Suit Oblique Flex Connector`, meshIndex("Nau Suit Oblique Flex Connector"), {
    translation: [sign * 0.282, 0.232, -0.020],
    rotation: rotZ(sign * 0.18),
    scale: [0.76, 0.82, 0.82],
  });
  const socket = addChild(nodeIds.torso, `Nau ${label} Shoulder Socket`, {
    translation: [sign * 0.515, 0.558, -0.02],
  });
  addMeshChild(socket, `Nau ${label} Shoulder Joint Cover`, meshIndex("Nau Joint Shoulder Socket"), {
    translation: [sign * -0.016, -0.016, 0.0],
    rotation: rotZ(sign * 0.14),
    scale: [0.25, 0.130, 0.26],
  });
  nodeIds[`${lower}ShoulderSocket`] = socket;
  const arm = addChild(nodeIds.torso, `Nau ${label} Arm`, {
    translation: [sign * 0.515, 0.558, -0.02],
  });
  nodeIds[`${lower}Arm`] = arm;
  addMeshChild(arm, `Nau ${label} Suit Shoulder Root Blend`, meshIndex("Nau Suit Shoulder Root Blend"), {
    translation: [sign * -0.068, -0.044, -0.006],
    rotation: rotZ(sign * 0.18),
    scale: [1.0, 0.96, 1.0],
  });
  addMeshChild(arm, `Nau ${label} Suit Lat Shoulder Connector`, meshIndex("Nau Suit Lat Shoulder Connector"), {
    translation: [sign * -0.074, -0.076, 0.064],
    rotation: rotZ(sign * 0.16),
    scale: [0.84, 0.84, 0.86],
  });
  addMeshChild(arm, `Nau ${label} Shoulder Bridge Sleeve`, meshIndex("Nau Suit Shoulder Bridge Sleeve"), {
    translation: [sign * -0.02, -0.010, 0.0],
    rotation: rotZ(sign * 0.10),
    scale: [0.88, 0.070, 0.88],
  });
  addMeshChild(arm, `Nau ${label} Seamless Shoulder Flex Cover`, meshIndex("Nau Seamless Shoulder Flex Cover"), {
    translation: [sign * -0.012, -0.032, 0.0],
    rotation: rotZ(sign * 0.10),
    scale: [0.82, 0.70, 0.82],
  });
  addMeshChild(arm, `Nau ${label} Suit Upper Arm`, meshIndex("Nau Suit Upper Arm"), {
    translation: [0.0, -0.315, 0.0],
    scale: [1.0, 0.62, 1.0],
  });
  addMeshChild(arm, `Nau ${label} Suit Bicep Volume`, meshIndex("Nau Suit Bicep Volume"), {
    translation: [sign * -0.026, -0.255, -0.070],
    rotation: rotZ(sign * 0.08),
  });
  addMeshChild(arm, `Nau ${label} Suit Tricep Sweep`, meshIndex("Nau Suit Tricep Sweep"), {
    translation: [sign * 0.022, -0.318, 0.060],
    rotation: rotZ(sign * -0.05),
  });
  addMeshChild(arm, `Nau ${label} Shoulder Accent`, meshIndex("Nau Accent Shoulder Guard"), {
    translation: [sign * 0.032, -0.035, -0.014],
    rotation: rotZ(sign * 0.20),
  });
  addMeshChild(arm, `Nau ${label} Suit Deltoid Filler`, meshIndex("Nau Suit Deltoid Filler"), {
    translation: [sign * 0.010, -0.052, 0.0],
    rotation: rotZ(sign * 0.16),
  });
  const elbowSocket = addChild(arm, `Nau ${label} Elbow Socket`, {
    translation: [0.0, -0.54, 0.018],
  });
  addMeshChild(elbowSocket, `Nau ${label} Elbow Joint Cover`, meshIndex("Nau Joint Wrist Sleeve"), {
    translation: [0.0, 0.0, 0.0],
    scale: [0.70, 0.050, 0.70],
  });
  const forearm = addChild(arm, `Nau ${label} Forearm`, {
    translation: [0.0, -0.54, 0.018],
  });
  nodeIds[`${lower}Forearm`] = forearm;
  addMeshChild(forearm, `Nau ${label} Elbow Bridge Sleeve`, meshIndex("Nau Suit Elbow Bridge Sleeve"), {
    translation: [0.0, 0.026, 0.0],
    scale: [0.90, 0.120, 0.90],
  });
  addMeshChild(forearm, `Nau ${label} Seamless Elbow Flex Cover`, meshIndex("Nau Seamless Elbow Flex Cover"), {
    translation: [0.0, 0.020, 0.0],
    scale: [0.94, 0.94, 0.94],
  });
  addMeshChild(forearm, `Nau ${label} Leather Forearm Wrap`, meshIndex("Nau Leather Forearm Wrap"), {
    translation: [0.0, -0.240, 0.0],
    scale: [1.0, 0.50, 1.0],
  });
  addMeshChild(forearm, `Nau ${label} Leather Forearm Tendon Strap`, meshIndex("Nau Leather Forearm Tendon Strap"), {
    translation: [sign * -0.018, -0.240, -0.086],
    rotation: rotZ(sign * -0.05),
  });
  addMeshChild(forearm, `Nau ${label} Accent Elbow Guard`, meshIndex("Nau Accent Elbow Guard"), {
    translation: [0.0, 0.03, -0.055],
    rotation: rotX(0.10),
  });
  addMeshChild(forearm, `Nau ${label} Leather Gauntlet Cuff`, meshIndex("Nau Leather Gauntlet Cuff"), {
    translation: [0.0, -0.375, 0.01],
    scale: [1.0, 0.18, 1.0],
  });
  const wristSocket = addChild(forearm, `Nau ${label} Wrist Socket`, {
    translation: [0.0, -0.49, -0.005],
  });
  addMeshChild(wristSocket, `Nau ${label} Wrist Joint Cover`, meshIndex("Nau Joint Wrist Sleeve"), {
    translation: [0.0, 0.040, 0.0],
    scale: [0.38, 0.064, 0.38],
  });
  const hand = addChild(forearm, `Nau ${label} Leather Hand Palm`, {
    mesh: meshIndex("Nau Leather Hand Palm"),
    translation: [0.0, -0.49, -0.005],
    rotation: rotX(0.08),
    scale: [1.02, 0.94, 1.04],
  });
  nodeIds[`${lower}Hand`] = hand;
  addMeshChild(hand, `Nau ${label} Wrist Bridge Sleeve`, meshIndex("Nau Leather Wrist Bridge Sleeve"), {
    translation: [0.0, 0.052, 0.0],
    scale: [0.82, 0.052, 0.82],
  });
  addMeshChild(hand, `Nau ${label} Seamless Wrist Flex Cover`, meshIndex("Nau Seamless Wrist Flex Cover"), {
    translation: [0.0, 0.054, 0.0],
    scale: [0.92, 0.92, 0.92],
  });
  addMeshChild(hand, `Nau ${label} Leather Wrist Palm Gusset`, meshIndex("Nau Leather Wrist Palm Gusset"), {
    translation: [0.0, 0.006, -0.045],
    rotation: rotX(0.08),
  });
  addMeshChild(hand, `Nau ${label} Leather Outer Palm Edge Pad`, meshIndex("Nau Leather Palm Edge Pad"), {
    translation: [sign * 0.094, -0.066, 0.018],
    rotation: rotZ(sign * -0.18),
  });
  addMeshChild(hand, `Nau ${label} Leather Inner Palm Edge Pad`, meshIndex("Nau Leather Palm Edge Pad"), {
    translation: [sign * -0.082, -0.072, 0.016],
    rotation: rotZ(sign * 0.10),
    scale: [0.84, 0.92, 0.90],
  });
  addMeshChild(hand, `Nau ${label} Leather Thumb Web Pad`, meshIndex("Nau Leather Thumb Web Pad"), {
    translation: [sign * 0.120, -0.060, -0.034],
    rotation: rotZ(sign * -0.34),
  });
  addMeshChild(hand, `Nau ${label} Leather Index Finger Grip`, meshIndex("Nau Leather Finger Grip"), {
    translation: [sign * -0.068, -0.138, -0.052],
    rotation: rotX(0.24),
    scale: [0.88, 0.168, 0.92],
  });
  addMeshChild(hand, `Nau ${label} Leather Finger Grip`, meshIndex("Nau Leather Finger Grip"), {
    translation: [0.0, -0.148, -0.060],
    rotation: rotX(0.24),
    scale: [1.0, 0.178, 1.0],
  });
  addMeshChild(hand, `Nau ${label} Leather Ring Finger Grip`, meshIndex("Nau Leather Finger Grip"), {
    translation: [sign * 0.060, -0.136, -0.052],
    rotation: rotX(0.22),
    scale: [0.84, 0.160, 0.88],
  });
  addMeshChild(hand, `Nau ${label} Leather Pinky Finger Grip`, meshIndex("Nau Leather Finger Grip"), {
    translation: [sign * 0.106, -0.122, -0.044],
    rotation: rotX(0.20),
    scale: [0.68, 0.128, 0.76],
  });
  addMeshChild(hand, `Nau ${label} Leather Thumb Grip`, meshIndex("Nau Leather Finger Grip"), {
    translation: [sign * 0.140, -0.096, -0.018],
    rotation: rotZ(sign * -0.56),
    scale: [0.78, 0.130, 0.84],
  });
  addMeshChild(hand, `Nau ${label} Leather Index Finger Tip Pad`, meshIndex("Nau Leather Finger Tip Pad"), {
    translation: [sign * -0.068, -0.232, -0.034],
    rotation: rotX(0.24),
  });
  addMeshChild(hand, `Nau ${label} Leather Middle Finger Tip Pad`, meshIndex("Nau Leather Finger Tip Pad"), {
    translation: [0.0, -0.250, -0.040],
    rotation: rotX(0.24),
  });
  addMeshChild(hand, `Nau ${label} Leather Ring Finger Tip Pad`, meshIndex("Nau Leather Finger Tip Pad"), {
    translation: [sign * 0.060, -0.228, -0.035],
    rotation: rotX(0.22),
  });
  addMeshChild(hand, `Nau ${label} Leather Pinky Finger Tip Pad`, meshIndex("Nau Leather Finger Tip Pad"), {
    translation: [sign * 0.106, -0.198, -0.030],
    rotation: rotX(0.20),
    scale: [0.82, 0.82, 0.86],
  });
  addMeshChild(hand, `Nau ${label} Leather Thumb Tip Pad`, meshIndex("Nau Leather Finger Tip Pad"), {
    translation: [sign * 0.174, -0.124, -0.018],
    rotation: rotZ(sign * -0.56),
    scale: [0.96, 0.96, 0.94],
  });
  addMeshChild(hand, `Nau ${label} Leather Palm Heel Pad`, meshIndex("Nau Leather Palm Heel Pad"), {
    translation: [sign * 0.010, -0.030, 0.052],
    rotation: rotX(-0.10),
  });
  addMeshChild(hand, `Nau ${label} Leather Finger Web Bridge`, meshIndex("Nau Leather Finger Web Bridge"), {
    translation: [sign * 0.010, -0.104, -0.071],
    rotation: rotX(0.18),
  });
  for (const [fingerName, offsetX] of [
    ["Index", sign * -0.058],
    ["Middle", 0.0],
    ["Ring", sign * 0.052],
    ["Pinky", sign * 0.092],
  ]) {
    addMeshChild(hand, `Nau ${label} Leather ${fingerName} Knuckle Pad`, meshIndex("Nau Leather Knuckle Pad"), {
      translation: [offsetX, -0.038, -0.064],
      rotation: rotX(0.18),
    });
  }
}

for (const side of [
  ["Left", -1],
  ["Right", 1],
]) {
  const [label, sign] = side;
  const lower = label.toLowerCase();
  const hipSocket = addChild(nodeIds.hips, `Nau ${label} Hip Socket`, {
    translation: [sign * 0.250, -0.170, 0.020],
  });
  nodeIds[`${lower}HipSocket`] = hipSocket;
  addMeshChild(nodeIds.hips, `Nau ${label} Suit Hip Inguinal Blend`, meshIndex("Nau Suit Hip Inguinal Blend"), {
    translation: [sign * 0.208, -0.205, -0.052],
    rotation: rotZ(sign * 0.12),
    scale: [1.0, 0.92, 1.0],
  });
  addMeshChild(nodeIds.hips, `Nau ${label} Suit Hip Web Capsule`, meshIndex("Nau Suit Hip Web Capsule"), {
    translation: [sign * 0.248, -0.178, -0.006],
    rotation: rotZ(sign * 0.10),
    scale: [1.0, 0.92, 1.0],
  });
  const leg = addChild(nodeIds.hips, `Nau ${label} Leg`, {
    translation: [sign * 0.250, -0.170, 0.020],
  });
  nodeIds[`${lower}Leg`] = leg;
  addMeshChild(leg, `Nau ${label} Hip Joint Cover`, meshIndex("Nau Joint Hip Socket"), {
    translation: [sign * 0.005, 0.002, 0.0],
    rotation: rotZ(sign * 0.10),
    scale: [0.42, 0.24, 0.44],
  });
  addMeshChild(leg, `Nau ${label} Hip Bridge Sleeve`, meshIndex("Nau Suit Hip Bridge Sleeve"), {
    translation: [sign * 0.006, 0.006, 0.0],
    rotation: rotZ(sign * 0.08),
    scale: [0.96, 0.086, 0.96],
  });
  addMeshChild(leg, `Nau ${label} Seamless Hip Flex Cover`, meshIndex("Nau Seamless Hip Flex Cover"), {
    translation: [sign * 0.008, -0.032, 0.0],
    rotation: rotZ(sign * 0.08),
    scale: [0.86, 0.82, 0.86],
  });
  addMeshChild(leg, `Nau ${label} Suit Hip Root Blend`, meshIndex("Nau Suit Hip Root Blend"), {
    translation: [sign * -0.014, -0.046, -0.002],
    rotation: rotZ(sign * 0.08),
    scale: [1.0, 0.96, 1.0],
  });
  addMeshChild(leg, `Nau ${label} Suit Glute Hip Connector`, meshIndex("Nau Suit Glute Hip Connector"), {
    translation: [sign * 0.018, -0.072, 0.076],
    rotation: rotZ(sign * 0.08),
    scale: [0.88, 0.88, 0.90],
  });
  addMeshChild(leg, `Nau ${label} Suit Hip Thigh Fairing`, meshIndex("Nau Suit Hip Thigh Fairing"), {
    translation: [sign * -0.014, 0.028, -0.026],
    rotation: rotZ(sign * 0.08),
    scale: [0.84, 0.86, 0.86],
  });
  addMeshChild(leg, `Nau ${label} Suit Thigh Guard`, meshIndex("Nau Suit Thigh Guard"), {
    translation: [0.0, -0.240, 0.0],
    scale: [1.0, 0.50, 1.0],
  });
  addMeshChild(leg, `Nau ${label} Suit Outer Thigh Sweep`, meshIndex("Nau Suit Thigh Sweep"), {
    translation: [sign * 0.084, -0.245, -0.018],
    rotation: rotZ(sign * -0.08),
  });
  addMeshChild(leg, `Nau ${label} Suit Inner Thigh Sweep`, meshIndex("Nau Suit Thigh Sweep"), {
    translation: [sign * -0.066, -0.250, 0.020],
    rotation: rotZ(sign * 0.06),
    scale: [0.84, 0.92, 0.86],
  });
  const kneeSocket = addChild(leg, `Nau ${label} Knee Socket`, {
    translation: [0.0, -0.43, 0.01],
  });
  addMeshChild(kneeSocket, `Nau ${label} Knee Joint Cover`, meshIndex("Nau Joint Knee Sleeve"), {
    translation: [0.0, 0.0, 0.0],
    scale: [0.58, 0.060, 0.58],
  });
  const lowerLeg = addChild(leg, `Nau ${label} Lower Leg`, {
    translation: [0.0, -0.43, 0.01],
  });
  nodeIds[`${lower}LowerLeg`] = lowerLeg;
  addMeshChild(lowerLeg, `Nau ${label} Knee Bridge Sleeve`, meshIndex("Nau Suit Knee Bridge Sleeve"), {
    translation: [0.0, -0.02, 0.0],
    scale: [1.0, 0.11, 1.0],
  });
  addMeshChild(lowerLeg, `Nau ${label} Seamless Knee Flex Cover`, meshIndex("Nau Seamless Knee Flex Cover"), {
    translation: [0.0, -0.026, 0.0],
  });
  addMeshChild(lowerLeg, `Nau ${label} Suit Lower Leg Greave`, meshIndex("Nau Suit Lower Leg Greave"), {
    translation: [0.0, -0.220, 0.0],
    scale: [1.0, 0.39, 1.0],
  });
  addMeshChild(lowerLeg, `Nau ${label} Suit Calf Volume`, meshIndex("Nau Suit Calf Volume"), {
    translation: [sign * 0.018, -0.240, 0.070],
    rotation: rotZ(sign * -0.04),
  });
  addMeshChild(lowerLeg, `Nau ${label} Suit Shin Ridge`, meshIndex("Nau Suit Shin Ridge"), {
    translation: [sign * -0.008, -0.232, -0.090],
    rotation: rotZ(sign * 0.03),
  });
  addMeshChild(lowerLeg, `Nau ${label} Accent Knee Guard`, meshIndex("Nau Accent Knee Guard"), {
    translation: [0.0, 0.02, -0.10],
    rotation: rotX(0.08),
  });
  addMeshChild(lowerLeg, `Nau ${label} Joint Knee Sleeve`, meshIndex("Nau Joint Knee Sleeve"), {
    translation: [0.0, 0.01, 0.0],
    scale: [1.0, 0.16, 1.0],
  });
  addMeshChild(lowerLeg, `Nau ${label} Suit Knee Tendon Strap`, meshIndex("Nau Suit Knee Tendon Strap"), {
    translation: [sign * -0.006, -0.060, -0.094],
    rotation: rotX(0.05),
  });
  const ankleSocket = addChild(lowerLeg, `Nau ${label} Ankle Socket`, {
    translation: [0.0, -0.405, -0.012],
  });
  addMeshChild(ankleSocket, `Nau ${label} Ankle Joint Cover`, meshIndex("Nau Leather Ankle Wrap"), {
    translation: [0.0, 0.170, 0.0],
    scale: [0.40, 0.038, 0.40],
  });
  const boot = addChild(lowerLeg, `Nau ${label} Boot`, {
    translation: [0.0, -0.405, -0.012],
  });
  nodeIds[`${lower}Boot`] = boot;
  addMeshChild(boot, `Nau ${label} Leather Boot Shell`, meshIndex("Nau Leather Boot"), {
    scale: [1.04, 0.34, 1.06],
  });
  addMeshChild(boot, `Nau ${label} Ankle Bridge Sleeve`, meshIndex("Nau Leather Ankle Bridge Sleeve"), {
    translation: [0.0, 0.186, 0.0],
    scale: [0.94, 0.046, 0.94],
  });
  addMeshChild(boot, `Nau ${label} Seamless Ankle Flex Cover`, meshIndex("Nau Seamless Ankle Flex Cover"), {
    translation: [0.0, 0.144, -0.004],
    scale: [0.98, 0.98, 0.98],
  });
  addMeshChild(boot, `Nau ${label} Leather Ankle Wrap`, meshIndex("Nau Leather Ankle Wrap"), {
    translation: [0.0, 0.08, -0.005],
    scale: [1.0, 0.18, 1.0],
  });
  addMeshChild(boot, `Nau ${label} Leather Heel Tendon Guard`, meshIndex("Nau Leather Heel Tendon Guard"), {
    translation: [0.0, 0.036, 0.128],
  });
  addMeshChild(boot, `Nau ${label} Leather Boot Instep Plate`, meshIndex("Nau Leather Boot Instep Plate"), {
    translation: [0.0, -0.098, -0.062],
    rotation: rotX(0.08),
  });
  addMeshChild(boot, `Nau ${label} Leather Outer Boot Side Guard`, meshIndex("Nau Leather Boot Side Guard"), {
    translation: [sign * 0.126, -0.110, -0.034],
    rotation: rotX(0.04),
  });
  addMeshChild(boot, `Nau ${label} Leather Inner Boot Side Guard`, meshIndex("Nau Leather Boot Side Guard"), {
    translation: [sign * -0.102, -0.112, -0.030],
    rotation: rotX(0.04),
    scale: [0.82, 0.88, 0.94],
  });
  addMeshChild(boot, `Nau ${label} Leather Boot Arch Rib`, meshIndex("Nau Leather Boot Arch Rib"), {
    translation: [0.0, -0.224, -0.016],
    rotation: rotZ(sign * 0.04),
  });
  addMeshChild(boot, `Nau ${label} Leather Ankle Boot Tongue`, meshIndex("Nau Leather Ankle Boot Tongue"), {
    translation: [0.0, -0.020, -0.108],
    rotation: rotX(0.08),
  });
  addMeshChild(boot, `Nau ${label} Leather Lace Cross Strap A`, meshIndex("Nau Leather Lace Cross Strap"), {
    translation: [sign * -0.034, -0.098, -0.074],
    rotation: rotZ(sign * 0.42),
  });
  addMeshChild(boot, `Nau ${label} Leather Lace Cross Strap B`, meshIndex("Nau Leather Lace Cross Strap"), {
    translation: [sign * 0.034, -0.098, -0.074],
    rotation: rotZ(sign * -0.42),
  });
  addMeshChild(boot, `Nau ${label} Leather Boot Toe Cap`, meshIndex("Nau Leather Boot Toe Cap"), {
    translation: [0.0, -0.136, -0.136],
    rotation: rotX(0.08),
    scale: [1.08, 1.0, 1.22],
  });
  addMeshChild(boot, `Nau ${label} Leather Outer Toe Lug`, meshIndex("Nau Leather Boot Toe Lug"), {
    translation: [sign * 0.086, -0.144, -0.166],
    rotation: rotX(0.08),
  });
  addMeshChild(boot, `Nau ${label} Leather Inner Toe Lug`, meshIndex("Nau Leather Boot Toe Lug"), {
    translation: [sign * -0.056, -0.142, -0.162],
    rotation: rotX(0.08),
    scale: [0.88, 0.88, 1.0],
  });
  addMeshChild(boot, `Nau ${label} Leather Boot Sole`, meshIndex("Nau Leather Boot Sole"), {
    translation: [0.0, -0.242, 0.000],
    scale: [1.08, 1.06, 1.20],
  });
  addMeshChild(boot, `Nau ${label} Leather Boot Heel`, meshIndex("Nau Leather Boot Heel"), {
    translation: [0.0, -0.196, 0.116],
    scale: [1.06, 1.0, 1.10],
  });
  addMeshChild(nodeIds.hips, `Nau ${label} Suit Pelvis Side Plate`, meshIndex("Nau Suit Pelvis Side Plate"), {
    translation: [sign * 0.29, -0.06, -0.005],
    rotation: rotZ(sign * -0.18),
  });
  addMeshChild(nodeIds.hips, `Nau ${label} Accent Side Tunic Flap`, meshIndex("Nau Accent Side Tunic Flap"), {
    translation: [sign * 0.36, -0.08, -0.03],
    rotation: rotZ(sign * -0.22),
    scale: [0.92, 0.90, 1.0],
  });
}

nodeIds.scarfAnchor = addMeshChild(nodeIds.torso, "Nau Back Scarf Anchor Accent", meshIndex("Nau Accent Scarf Trail"), {
  translation: [0.0, 0.42, 0.25],
  rotation: rotX(-1.24),
  scale: [0.74, 0.42, 1.0],
});
nodeIds.scarfTrail = addMeshChild(nodeIds.torso, "Nau Wind Scarf Accent", meshIndex("Nau Accent Scarf Trail"), {
  translation: [0.20, 0.32, 0.36],
  rotation: rotX(-0.55),
  scale: [0.92, 1.0, 1.0],
});

nodeIds.signalRoot = addChild(nodeIds.root, "Nau Animation Signal Root");
for (const name of [
  "Hips",
  "Torso",
  "Head",
  "Left Arm",
  "Right Arm",
  "Left Forearm",
  "Right Forearm",
  "Left Leg",
  "Right Leg",
  "Left Lower Leg",
  "Right Lower Leg",
  "Left Boot",
  "Right Boot",
  "Left Hand",
  "Right Hand",
  "Left Tunic",
  "Right Tunic",
]) {
  const key = `signal${name.replaceAll(" ", "")}`;
  nodeIds[key] = addChild(nodeIds.signalRoot, `Nau Animation Signal ${name}`, {
    translation: [0.0, 0.0, 0.0],
  });
}

function quat(axis, radians) {
  const half = radians / 2;
  const s = Math.sin(half);
  return [axis[0] * s, axis[1] * s, axis[2] * s, Math.cos(half)];
}

function rotX(radians) {
  return quat([1, 0, 0], radians);
}

function rotY(radians) {
  return quat([0, 1, 0], radians);
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

const s = {
  hips: nodeIds.signalHips,
  torso: nodeIds.signalTorso,
  head: nodeIds.signalHead,
  leftArm: nodeIds.signalLeftArm,
  rightArm: nodeIds.signalRightArm,
  leftLeg: nodeIds.signalLeftLeg,
  rightLeg: nodeIds.signalRightLeg,
  leftBoot: nodeIds.signalLeftBoot,
  rightBoot: nodeIds.signalRightBoot,
  leftHand: nodeIds.signalLeftHand,
  rightHand: nodeIds.signalRightHand,
  leftTunic: nodeIds.signalLeftTunic,
  rightTunic: nodeIds.signalRightTunic,
};

const animations = [
  animation("Idle_Loop", [
    { node: s.hips, path: "rotation", times: loopTimes, values: [rotX(0.0), rotX(0.012), rotX(0.0)] },
    { node: s.head, path: "rotation", times: loopTimes, values: [rotZ(-0.04), rotZ(0.04), rotZ(-0.04)] },
    { node: s.torso, path: "translation", times: loopTimes, values: [[0, 0, 0], [0, 0.025, 0], [0, 0, 0]] },
    { node: s.leftArm, path: "rotation", times: loopTimes, values: [rotZ(0.05), rotZ(0.08), rotZ(0.05)] },
    { node: s.rightArm, path: "rotation", times: loopTimes, values: [rotZ(-0.05), rotZ(-0.08), rotZ(-0.05)] },
    { node: s.leftLeg, path: "rotation", times: loopTimes, values: [rotX(0.02), rotX(0.05), rotX(0.02)] },
    { node: s.rightLeg, path: "rotation", times: loopTimes, values: [rotX(0.02), rotX(0.05), rotX(0.02)] },
    { node: s.leftTunic, path: "rotation", times: loopTimes, values: [rotZ(0.24), rotZ(0.32), rotZ(0.24)] },
    { node: s.rightTunic, path: "rotation", times: loopTimes, values: [rotZ(-0.24), rotZ(-0.32), rotZ(-0.24)] },
  ]),
  animation("Walk_Fwd_Loop", [
    { node: s.hips, path: "rotation", times: loopTimes, values: [rotX(-0.02), rotX(0.02), rotX(-0.02)] },
    { node: s.torso, path: "translation", times: loopTimes, values: [[0, 0, 0], [0, 0.020, 0], [0, 0, 0]] },
    { node: s.leftArm, path: "rotation", times: loopTimes, values: [rotX(-0.26), rotX(0.26), rotX(-0.26)] },
    { node: s.rightArm, path: "rotation", times: loopTimes, values: [rotX(0.26), rotX(-0.26), rotX(0.26)] },
    { node: s.leftLeg, path: "rotation", times: loopTimes, values: [rotX(0.3), rotX(-0.3), rotX(0.3)] },
    { node: s.rightLeg, path: "rotation", times: loopTimes, values: [rotX(-0.3), rotX(0.3), rotX(-0.3)] },
    { node: s.leftBoot, path: "rotation", times: loopTimes, values: [rotX(-0.10), rotX(0.08), rotX(-0.10)] },
    { node: s.rightBoot, path: "rotation", times: loopTimes, values: [rotX(0.08), rotX(-0.10), rotX(0.08)] },
    { node: s.leftTunic, path: "rotation", times: loopTimes, values: [rotZ(0.18), rotZ(0.36), rotZ(0.18)] },
    { node: s.rightTunic, path: "rotation", times: loopTimes, values: [rotZ(-0.36), rotZ(-0.18), rotZ(-0.36)] },
  ]),
  animation("Run_Fwd_Loop", [
    { node: s.hips, path: "rotation", times: loopTimes, values: [rotX(-0.04), rotX(0.04), rotX(-0.04)] },
    { node: s.torso, path: "translation", times: loopTimes, values: [[0, -0.02, 0], [0, 0.055, 0], [0, -0.02, 0]] },
    { node: s.head, path: "rotation", times: loopTimes, values: [rotX(-0.08), rotX(0.12), rotX(-0.08)] },
    { node: s.leftArm, path: "rotation", times: loopTimes, values: [rotX(-0.62), rotX(0.62), rotX(-0.62)] },
    { node: s.rightArm, path: "rotation", times: loopTimes, values: [rotX(0.62), rotX(-0.62), rotX(0.62)] },
    { node: s.leftLeg, path: "rotation", times: loopTimes, values: [rotX(0.7), rotX(-0.7), rotX(0.7)] },
    { node: s.rightLeg, path: "rotation", times: loopTimes, values: [rotX(-0.7), rotX(0.7), rotX(-0.7)] },
    { node: s.leftBoot, path: "rotation", times: loopTimes, values: [rotX(-0.18), rotX(0.16), rotX(-0.18)] },
    { node: s.rightBoot, path: "rotation", times: loopTimes, values: [rotX(0.16), rotX(-0.18), rotX(0.16)] },
    { node: s.leftTunic, path: "rotation", times: loopTimes, values: [rotZ(0.12), rotZ(0.42), rotZ(0.12)] },
    { node: s.rightTunic, path: "rotation", times: loopTimes, values: [rotZ(-0.42), rotZ(-0.12), rotZ(-0.42)] },
  ]),
  animation("Launch_Start", [
    { node: s.hips, path: "rotation", times: shortTimes, values: [rotX(0.0), rotX(-0.24), rotX(-0.18)] },
    { node: s.torso, path: "translation", times: shortTimes, values: [[0, 0, 0], [0, 0.16, 0], [0, 0.04, 0]] },
    { node: s.leftArm, path: "rotation", times: shortTimes, values: [rotX(0.15), rotX(-0.85), rotX(-0.35)] },
    { node: s.rightArm, path: "rotation", times: shortTimes, values: [rotX(0.15), rotX(-0.85), rotX(-0.35)] },
    { node: s.leftLeg, path: "rotation", times: shortTimes, values: [rotX(-0.20), rotX(-0.58), rotX(-0.30)] },
    { node: s.rightLeg, path: "rotation", times: shortTimes, values: [rotX(-0.20), rotX(-0.58), rotX(-0.30)] },
  ]),
  animation("Fall_Loop", [
    { node: s.hips, path: "rotation", times: loopTimes, values: [rotX(-1.18), rotX(-1.30), rotX(-1.18)] },
    { node: s.torso, path: "rotation", times: loopTimes, values: [rotX(-0.10), rotX(-0.16), rotX(-0.10)] },
    { node: s.head, path: "rotation", times: loopTimes, values: [rotX(0.16), rotX(0.22), rotX(0.16)] },
    { node: s.leftArm, path: "rotation", times: loopTimes, values: [rotZ(1.36), rotZ(1.46), rotZ(1.36)] },
    { node: s.rightArm, path: "rotation", times: loopTimes, values: [rotZ(-1.36), rotZ(-1.46), rotZ(-1.36)] },
    { node: s.leftLeg, path: "rotation", times: loopTimes, values: [rotX(0.48), rotX(0.58), rotX(0.48)] },
    { node: s.rightLeg, path: "rotation", times: loopTimes, values: [rotX(0.48), rotX(0.58), rotX(0.48)] },
  ]),
  animation("Glide_Loop", [
    { node: s.hips, path: "rotation", times: loopTimes, values: [rotX(-0.04), rotX(-0.06), rotX(-0.04)] },
    { node: s.leftArm, path: "rotation", times: loopTimes, values: [rotZ(1.08), rotZ(1.18), rotZ(1.08)] },
    { node: s.rightArm, path: "rotation", times: loopTimes, values: [rotZ(-1.08), rotZ(-1.18), rotZ(-1.08)] },
    { node: s.torso, path: "rotation", times: loopTimes, values: [rotX(0.08), rotX(0.14), rotX(0.08)] },
    { node: s.leftHand, path: "rotation", times: loopTimes, values: [rotX(0.14), rotX(0.24), rotX(0.14)] },
    { node: s.rightHand, path: "rotation", times: loopTimes, values: [rotX(0.14), rotX(0.24), rotX(0.14)] },
    { node: s.leftLeg, path: "rotation", times: loopTimes, values: [rotX(0.32), rotX(0.40), rotX(0.32)] },
    { node: s.rightLeg, path: "rotation", times: loopTimes, values: [rotX(0.32), rotX(0.40), rotX(0.32)] },
    { node: s.leftTunic, path: "rotation", times: loopTimes, values: [rotZ(0.42), rotZ(0.54), rotZ(0.42)] },
    { node: s.rightTunic, path: "rotation", times: loopTimes, values: [rotZ(-0.42), rotZ(-0.54), rotZ(-0.42)] },
  ]),
  animation("Bank_Left", [
    { node: s.hips, path: "rotation", times: loopTimes, values: [rotZ(0.12), rotZ(0.18), rotZ(0.12)] },
    { node: s.torso, path: "rotation", times: loopTimes, values: [rotZ(0.22), rotZ(0.34), rotZ(0.22)] },
    { node: s.head, path: "rotation", times: loopTimes, values: [rotZ(-0.12), rotZ(-0.18), rotZ(-0.12)] },
    { node: s.leftArm, path: "rotation", times: loopTimes, values: [rotZ(0.86), rotZ(0.98), rotZ(0.86)] },
    { node: s.rightArm, path: "rotation", times: loopTimes, values: [rotZ(-1.28), rotZ(-1.40), rotZ(-1.28)] },
    { node: s.leftLeg, path: "rotation", times: loopTimes, values: [rotX(0.12), rotX(0.22), rotX(0.12)] },
    { node: s.rightLeg, path: "rotation", times: loopTimes, values: [rotX(-0.08), rotX(-0.14), rotX(-0.08)] },
  ]),
  animation("Bank_Right", [
    { node: s.hips, path: "rotation", times: loopTimes, values: [rotZ(-0.12), rotZ(-0.18), rotZ(-0.12)] },
    { node: s.torso, path: "rotation", times: loopTimes, values: [rotZ(-0.22), rotZ(-0.34), rotZ(-0.22)] },
    { node: s.head, path: "rotation", times: loopTimes, values: [rotZ(0.12), rotZ(0.18), rotZ(0.12)] },
    { node: s.leftArm, path: "rotation", times: loopTimes, values: [rotZ(1.28), rotZ(1.40), rotZ(1.28)] },
    { node: s.rightArm, path: "rotation", times: loopTimes, values: [rotZ(-0.86), rotZ(-0.98), rotZ(-0.86)] },
    { node: s.leftLeg, path: "rotation", times: loopTimes, values: [rotX(-0.08), rotX(-0.14), rotX(-0.08)] },
    { node: s.rightLeg, path: "rotation", times: loopTimes, values: [rotX(0.12), rotX(0.22), rotX(0.12)] },
  ]),
  animation("Dive_Loop", [
    { node: s.hips, path: "rotation", times: loopTimes, values: [rotX(-2.45), rotX(-2.63), rotX(-2.45)] },
    { node: s.torso, path: "rotation", times: loopTimes, values: [rotX(-0.36), rotX(-0.48), rotX(-0.36)] },
    { node: s.head, path: "rotation", times: loopTimes, values: [rotX(-0.06), rotX(-0.02), rotX(-0.06)] },
    { node: s.leftArm, path: "rotation", times: loopTimes, values: [rotZ(0.14), rotZ(0.18), rotZ(0.14)] },
    { node: s.rightArm, path: "rotation", times: loopTimes, values: [rotZ(-0.14), rotZ(-0.18), rotZ(-0.14)] },
    { node: s.leftLeg, path: "rotation", times: loopTimes, values: [rotX(0.05), rotX(0.10), rotX(0.05)] },
    { node: s.rightLeg, path: "rotation", times: loopTimes, values: [rotX(0.05), rotX(0.10), rotX(0.05)] },
    { node: s.leftHand, path: "rotation", times: loopTimes, values: [rotX(0.04), rotX(0.08), rotX(0.04)] },
    { node: s.rightHand, path: "rotation", times: loopTimes, values: [rotX(0.04), rotX(0.08), rotX(0.04)] },
    { node: s.leftTunic, path: "rotation", times: loopTimes, values: [rotZ(0.18), rotZ(0.24), rotZ(0.18)] },
    { node: s.rightTunic, path: "rotation", times: loopTimes, values: [rotZ(-0.18), rotZ(-0.24), rotZ(-0.18)] },
  ]),
  animation("Air_Brake", [
    { node: s.hips, path: "rotation", times: shortTimes, values: [rotX(0.0), rotX(0.08), rotX(0.04)] },
    { node: s.torso, path: "rotation", times: shortTimes, values: [rotX(0.0), rotX(-0.24), rotX(-0.16)] },
    { node: s.leftArm, path: "rotation", times: shortTimes, values: [rotZ(0.55), rotZ(0.92), rotZ(0.72)] },
    { node: s.rightArm, path: "rotation", times: shortTimes, values: [rotZ(-0.55), rotZ(-0.92), rotZ(-0.72)] },
    { node: s.leftHand, path: "rotation", times: shortTimes, values: [rotX(0.05), rotX(-0.18), rotX(-0.10)] },
    { node: s.rightHand, path: "rotation", times: shortTimes, values: [rotX(0.05), rotX(-0.18), rotX(-0.10)] },
    { node: s.leftLeg, path: "rotation", times: shortTimes, values: [rotY(-0.10), rotY(-0.20), rotY(-0.12)] },
    { node: s.rightLeg, path: "rotation", times: shortTimes, values: [rotY(0.10), rotY(0.20), rotY(0.12)] },
  ]),
  animation("Land", [
    { node: s.hips, path: "rotation", times: shortTimes, values: [rotX(0.16), rotX(0.30), rotX(0.12)] },
    { node: s.torso, path: "translation", times: shortTimes, values: [[0, 0.09, 0], [0, 0.02, 0], [0, 0, 0]] },
    { node: s.leftArm, path: "rotation", times: shortTimes, values: [rotZ(0.60), rotZ(0.92), rotZ(0.26)] },
    { node: s.rightArm, path: "rotation", times: shortTimes, values: [rotZ(-0.60), rotZ(-0.92), rotZ(-0.26)] },
    { node: s.leftLeg, path: "rotation", times: shortTimes, values: [rotX(-0.15), rotX(0.24), rotX(0)] },
    { node: s.rightLeg, path: "rotation", times: shortTimes, values: [rotX(-0.15), rotX(0.24), rotX(0)] },
    { node: s.leftBoot, path: "rotation", times: shortTimes, values: [rotX(-0.12), rotX(0.22), rotX(0)] },
    { node: s.rightBoot, path: "rotation", times: shortTimes, values: [rotX(-0.12), rotX(0.22), rotX(0)] },
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
