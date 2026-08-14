import type { ModLoader } from '../types'
import grassBlockLogo from '../assets/icons/grass_block.png'
import forgeLogo from '../assets/icons/forge.png'
import neoforgeLogo from '../assets/icons/neoforge.png'
import fabricLogo from '../assets/icons/fabric.png'
import quiltLogo from '../assets/icons/quilt.png'
import optifineLogo from '../assets/icons/optifine.png'

export function LoaderLogo({
  loader,
  versionId,
  className,
}: {
  loader: ModLoader | string | null
  versionId?: string | null
  className?: string
}) {
  if (loader === null || loader === undefined) return null
  const key = typeof loader === 'string' ? loader : Object.keys(loader)[0]
  const resolved = versionId?.includes('OptiFine') ? 'OptiFine' : key

  const map: Record<string, string> = {
    Forge: forgeLogo,
    NeoForge: neoforgeLogo,
    Fabric: fabricLogo,
    Quilt: quiltLogo,
    OptiFine: optifineLogo,
  }

  if (resolved === 'OptiFine') {
    return <img src={optifineLogo} className={className} alt="OptiFine" draggable={false} />
  }

  if (resolved === 'Vanilla' || resolved === 'vanilla') {
    return <img src={grassBlockLogo} className={`${className ?? ''} object-contain`} alt="Vanilla" draggable={false} />
  }

  const src = map[resolved]
  if (!src) return <img src={grassBlockLogo} className={`${className ?? ''} object-contain`} alt="Vanilla" draggable={false} />

  return (
    <img
      src={src}
      className={`${className ?? ''} object-contain`}
      alt={resolved}
      draggable={false}
    />
  )
}
