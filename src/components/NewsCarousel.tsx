import { useEffect, useState, useCallback, useRef } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { open as shellOpen } from '@tauri-apps/plugin-shell'
import { ChevronLeft, ChevronRight, Newspaper } from 'lucide-react'

interface McNewsItem {
  title: string
  image_url: string
  link: string
}

const INTERVAL = 6000

let newsCache: McNewsItem[] | null = null
let newsLoading = false
let newsWaiters: Array<(items: McNewsItem[] | null) => void> = []

export function NewsCarousel() {
  const [items, setItems] = useState<McNewsItem[]>(newsCache ?? [])
  const [current, setCurrent] = useState(0)
  const [loading, setLoading] = useState(newsCache === null)
  const [error, setError] = useState(false)
  const [direction, setDirection] = useState<'left' | 'right'>('right')
  const [animating, setAnimating] = useState(false)
  const timerRef = useRef<number | null>(null)

  useEffect(() => {
    if (newsCache !== null) return
    if (newsLoading) {
      const waiter = (cached: McNewsItem[] | null) => {
        if (cached) { setItems(cached); setError(false) }
        else setError(true)
        setLoading(false)
      }
      newsWaiters.push(waiter)
      return () => { newsWaiters = newsWaiters.filter(w => w !== waiter) }
    }
    newsLoading = true
    invoke<McNewsItem[]>('fetch_mc_news')
      .then((data) => {
        newsCache = data
        setItems(data); setError(false)
        newsWaiters.forEach(w => w(data))
      })
      .catch(() => {
        setError(true)
        newsWaiters.forEach(w => w(null))
      })
      .finally(() => { setLoading(false); newsWaiters = [] })
  }, [])

  const clearTimer = useCallback(() => {
    if (timerRef.current) {
      window.clearInterval(timerRef.current)
      timerRef.current = null
    }
  }, [])

  const startTimer = useCallback(() => {
    clearTimer()
    timerRef.current = window.setInterval(() => {
      setDirection('right')
      setAnimating(true)
      setTimeout(() => {
        setCurrent((i) => (i + 1) % (items.length || 1))
        setAnimating(false)
      }, 300)
    }, INTERVAL)
  }, [items.length, clearTimer])

  useEffect(() => {
    if (items.length > 1) startTimer()
    return clearTimer
  }, [items.length, startTimer, clearTimer])

  const go = useCallback(
    (dir: number) => {
      if (animating) return
      setDirection(dir > 0 ? 'right' : 'left')
      setAnimating(true)
      setTimeout(() => {
        setCurrent((i) => (i + dir + items.length) % items.length)
        setAnimating(false)
      }, 300)
      if (items.length > 1) startTimer()
    },
    [items.length, startTimer, animating],
  )

  const openLink = useCallback(async (url: string) => {
    try { await shellOpen(url) } catch {}
  }, [])

  if (loading) {
    return (
      <div className="w-full h-full rounded-2xl bg-white/60 dark:bg-surface-800/60 backdrop-blur-sm border border-surface-200/60 dark:border-surface-700/40 flex items-center justify-center">
        <div className="flex flex-col items-center gap-2 text-surface-400">
          <div className="w-5 h-5 rounded-full border-2 border-surface-300 dark:border-surface-600 border-t-[var(--accent-color)] animate-spin" />
          <span className="text-xs">加载资讯中...</span>
        </div>
      </div>
    )
  }

  if (error || items.length === 0) {
    return (
      <div className="w-full h-full rounded-2xl bg-white/60 dark:bg-surface-800/60 backdrop-blur-sm border border-surface-200/60 dark:border-surface-700/40 flex items-center justify-center">
        <div className="flex flex-col items-center gap-2 text-surface-400">
          <Newspaper className="w-8 h-8" />
          <span className="text-xs">暂无资讯</span>
        </div>
      </div>
    )
  }

  const item = items[current]

  return (
    <div
      className="group relative w-full h-full rounded-2xl overflow-hidden bg-surface-900 border border-surface-200/60 dark:border-surface-700/40 shadow-lg cursor-pointer"
      onClick={() => openLink(item.link)}
      title="在浏览器中打开"
    >
      <div className="absolute inset-0 overflow-hidden">
        <img
          key={current}
          src={item.image_url}
          alt=""
          className="absolute inset-0 w-full h-full object-contain bg-black/20 animate-news-in"
          style={{
            animationName: animating
              ? direction === 'right' ? 'newsSlideOutLeft' : 'newsSlideOutRight'
              : 'newsSlideIn',
            animationDuration: '0.35s',
            animationTimingFunction: 'cubic-bezier(0.4, 0, 0.2, 1)',
            animationFillMode: 'both',
          }}
          draggable={false}
        />
      </div>

      <div className="absolute inset-0 bg-gradient-to-t from-black/80 via-black/20 to-transparent z-10" />

      <div
        className="absolute bottom-7 left-0 right-0 z-20 px-4"
        style={{
          animation: animating ? 'none' : 'newsFadeIn 0.4s ease both',
          animationDelay: animating ? undefined : '0.15s',
        }}
      >
        <h3 className="text-white text-xs font-semibold leading-snug line-clamp-2 drop-shadow-md">
          {item.title}
        </h3>
      </div>

      {items.length > 1 && (
        <>
          <button
            onClick={(e) => { e.stopPropagation(); go(-1) }}
            className="absolute left-2 top-1/2 -translate-y-1/2 z-30 w-8 h-8 rounded-full bg-black/40 backdrop-blur-sm text-white/80 hover:bg-black/60 hover:text-white flex items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity cursor-pointer"
            title="上一条"
          >
            <ChevronLeft className="w-4 h-4" />
          </button>
          <button
            onClick={(e) => { e.stopPropagation(); go(1) }}
            className="absolute right-2 top-1/2 -translate-y-1/2 z-30 w-8 h-8 rounded-full bg-black/40 backdrop-blur-sm text-white/80 hover:bg-black/60 hover:text-white flex items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity cursor-pointer"
            title="下一条"
          >
            <ChevronRight className="w-4 h-4" />
          </button>
        </>
      )}

      {items.length > 1 && (
        <div className="absolute bottom-2 left-1/2 -translate-x-1/2 z-30 flex items-center gap-1">
          {items.map((_, idx) => (
            <button
              key={idx}
              onClick={(e) => { e.stopPropagation(); if (idx !== current && !animating) { setDirection(idx > current ? 'right' : 'left'); setAnimating(true); setTimeout(() => { setCurrent(idx); setAnimating(false) }, 300); if (items.length > 1) startTimer() } }}
              className={`rounded-full transition-all duration-300 cursor-pointer ${
                idx === current
                  ? 'w-5 h-1.5 bg-white'
                  : 'w-1.5 h-1.5 bg-white/40 hover:bg-white/60'
              }`}
            />
          ))}
        </div>
      )}
    </div>
  )
}
