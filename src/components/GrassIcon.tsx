const DIRT = '#8a5a3b'
const DIRT_D = '#6f4528'
const DIRT_L = '#a5704a'
const GRASS = '#60af2d'
const GRASS_D = '#4d8f1e'
const GRASS_L = '#79c440'

export function GrassIcon({ className, size }: { className?: string; size?: number }) {
  return (
    <svg
      viewBox="0 0 16 16"
      className={className}
      style={size ? { width: size, height: size } : undefined}
      aria-hidden
      shapeRendering="crispEdges"
    >
      {}
      <polygon points="8,7.6 15,4.1 15,11.6 8,15.1" fill={DIRT_D} />
      <polygon points="8,7.6 15,4.1 15,5.7 8,9.2" fill={GRASS_D} />
      <polygon points="11.2,9.8 12.8,10.7 11.2,11.6 9.6,10.7" fill={DIRT} opacity="0.5" />
      <polygon points="11.2,12.6 12.8,13.5 11.2,14.4 9.6,13.5" fill={DIRT} opacity="0.5" />
      {}
      <polygon points="1,4.1 8,7.6 8,15.1 1,11.6" fill={DIRT} />
      <polygon points="1,4.1 8,7.6 8,9.2 1,5.7" fill={GRASS} />
      <polygon points="2.4,6.2 4,7.1 2.4,8 0.8,7.1" fill={DIRT_L} opacity="0.7" />
      <polygon points="5.6,8.9 7.2,9.8 5.6,10.7 4,9.8" fill={DIRT_L} opacity="0.7" />
      <polygon points="2.4,10.6 4,11.5 2.4,12.4 0.8,11.5" fill={DIRT_L} opacity="0.7" />
      <polygon points="5.6,13 7.2,13.9 5.6,14.8 4,13.9" fill={DIRT_L} opacity="0.7" />
      {}
      <polygon points="8,0.6 15,4.1 8,7.6 1,4.1" fill={GRASS} />
      <polygon points="4.8,2.3 6.3,3.1 4.8,3.9 3.3,3.1" fill={GRASS_L} />
      <polygon points="10.1,2 11.5,2.8 10.1,3.6 8.7,2.8" fill={GRASS_L} />
      <polygon points="3.4,4.4 4.7,5.1 3.4,5.8 2.1,5.1" fill={GRASS_D} />
      <polygon points="11,5.2 12.3,5.9 11,6.6 9.7,5.9" fill={GRASS_D} />
      {}
      <polygon points="1,4.1 8,7.6 1,5.7" fill={GRASS_L} opacity="0.3" />
      <polygon points="8,7.6 15,4.1 15,5.7 8,9.2" fill={GRASS_L} opacity="0.15" />
    </svg>
  )
}
