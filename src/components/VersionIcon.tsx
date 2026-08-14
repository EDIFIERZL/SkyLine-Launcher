import grassBlock from '../assets/icons/grass_block.png'
import commandBlock from '../assets/icons/command_block.png'
import furnace from '../assets/icons/furnace.png'
import slimeBlock from '../assets/icons/slime_block.png'

export type VersionGroup = 'release' | 'snapshot' | 'old' | 'april'

const ICONS: Record<VersionGroup, string> = {
  release: grassBlock,
  snapshot: commandBlock,
  old: furnace,
  april: slimeBlock,
}

export function VersionIcon({
  group,
  size = 24,
  className,
}: {
  group: VersionGroup
  size?: number
  className?: string
}) {
  return (
    <img
      src={ICONS[group]}
      alt=""
      draggable={false}
      className={`${className ?? ''} object-contain shrink-0`}
      style={{ width: size, height: size }}
    />
  )
}
