import React, { useEffect, useRef } from 'react'
import WaveSurfer from 'wavesurfer.js'
import Regions from 'wavesurfer.js/dist/plugins/regions.js'
import Timeline from 'wavesurfer.js/dist/plugins/timeline.js'
import Hover from 'wavesurfer.js/dist/plugins/hover.js'
import Minimap from 'wavesurfer.js/dist/plugins/minimap.js'

type Trim = { start: number; end: number }
type Props = { audioUrl: string; value?: Trim | null; onChange?: (trim: Trim) => void }

const WavesurferTrimmer: React.FC<Props> = ({ audioUrl, value, onChange }) => {
  const waveRef = useRef<HTMLDivElement>(null)
  const timelineRef = useRef<HTMLDivElement>(null)
  const wsRef = useRef<WaveSurfer | null>(null)
  const regionsRef = useRef<ReturnType<typeof Regions.create> | null>(null)

  const upsertTrimRegion = (start: number, end: number) => {
    const regions = regionsRef.current!.getRegions()
    const prev = regions.find(r => String(r.id) === 'trim-region')
    const opts = { id: 'trim-region', start, end, color: 'rgba(0,0,0,0.2)', drag: true, resize: true, content: 'trim' }
    if (!prev) {
      const region = regionsRef.current!.addRegion(opts);
      if (region.element) {
        region.element.setAttribute('role', 'trim');
      }
    } else {
      prev.setOptions(opts);
    }
  }

  useEffect(() => {
    if (!waveRef.current || !timelineRef.current) return

    const ws = WaveSurfer.create({
      container: waveRef.current,
      url: audioUrl,
      waveColor: '#4F4A85',
      progressColor: '#383351',
      minPxPerSec: 60,
      autoScroll: true,
      autoCenter: true,
      dragToSeek: true,
      barWidth: 1, barGap: 1, barRadius: 1,
      plugins: [],
    })

    const regions = ws.registerPlugin(Regions.create())
    regionsRef.current = regions
    ws.registerPlugin(Timeline.create({ container: timelineRef.current }))
    ws.registerPlugin(Hover.create())
    ws.registerPlugin(Minimap.create({ height: 24 }))

    regions.enableDragSelection({ color: 'rgba(0,0,0,0.15)' })

    ws.on('ready', () => {
      const dur = ws.getDuration()
      const initial: Trim =
        value && value.end > value.start ? value : { start: Math.max(0, dur * 0.05), end: Math.max(0.3, dur * 0.9) }
      upsertTrimRegion(initial.start, initial.end)
      onChange?.(initial)
    })

    regions.on('region-created', r => {
      if (String(r.id) !== 'trim-region') {
        const existed = regions.getRegions().find(x => String(x.id) === 'trim-region')
        if (existed) r.remove()
        else r.setOptions({ id: 'trim-region', color: 'rgba(0,0,0,0.2)', content: 'trim' })
      }
    })

    regions.on('region-updated', r => {
      r.setOptions({ color: 'rgba(0,0,0,0.2)' });

      // Commit changes on update
      const start = Math.max(0, Math.min(r.start, r.end))
      const end = Math.max(start, Math.max(r.start, r.end))
      if (Math.abs(r.start - start) > 0.001 || Math.abs(r.end - end) > 0.001) {
        r.setOptions({ start, end })
      }
      onChange?.({ start, end })
    })

    wsRef.current = ws
    return () => ws.destroy()
  }, [audioUrl])

  useEffect(() => { if (value && regionsRef.current) upsertTrimRegion(value.start, value.end) }, [value?.start, value?.end])

  return (
    <div style={{ width: '100%' }}>
      <div id="trimmer" ref={waveRef} style={{ height: 160, border: '1px solid #ddd', borderRadius: 4, background: '#fafafa' }} />
      <div ref={timelineRef} />
      <div style={{ fontSize: 12, color: '#666', marginTop: 4 }}>
        Tip: drag to create/resize the dark region; only one trim region is allowed.
      </div>
    </div>
  )
}
export default WavesurferTrimmer
