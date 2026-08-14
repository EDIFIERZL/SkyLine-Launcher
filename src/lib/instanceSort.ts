import type { Instance, ModLoader } from '../types'

function versionToParts(v: string): number[] {
  const base = v.split('-')[0]
  return base.split('.').map((s) => {
    const m = s.match(/^\d+/)
    return m ? parseInt(m[0], 10) : 0
  })
}

function compareVersion(a: string, b: string): number {
  const pa = versionToParts(a)
  const pb = versionToParts(b)
  const len = Math.max(pa.length, pb.length)
  for (let i = 0; i < len; i++) {
    const x = pa[i] ?? 0
    const y = pb[i] ?? 0
    if (x !== y) return x - y
  }
  return 0
}

export function getLoaderKey(loader: ModLoader | null | undefined): string {
  if (!loader) return 'Vanilla'
  if (typeof loader === 'string') return loader
  return Object.keys(loader)[0] ?? 'Vanilla'
}

export function extractMcVersion(versionId: string): string {
  const m = versionId.match(/(\d+\.\d+(?:\.\d+)?)/)
  return m ? m[1] : versionId
}

export function instanceSortKey(inst: Instance): { group: number; version: string } {
  const loader = getLoaderKey(inst.modloader)
  const isOptifine = /optifine/i.test(inst.version_id)
  let group = 0
  if (isOptifine) {
    group = 2
  } else {
    switch (loader) {
      case 'Vanilla': group = 1; break
      case 'Fabric': group = 3; break
      case 'Forge': group = 4; break
      case 'NeoForge': group = 5; break
      case 'Quilt': group = 6; break
      default: group = 9
    }
  }
  return { group, version: extractMcVersion(inst.version_id) }
}

export function sortInstances(list: Instance[]): Instance[] {
  return [...list].sort((a, b) => {
    const ka = instanceSortKey(a)
    const kb = instanceSortKey(b)
    if (ka.group !== kb.group) return ka.group - kb.group
    return compareVersion(ka.version, kb.version)
  })
}
