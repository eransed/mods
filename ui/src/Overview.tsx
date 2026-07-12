
import { useEffect, useRef, useState } from 'react'
import './Overview.css'

function resizeCanvas(canvas: HTMLCanvasElement) {
  const { width, height } = canvas.getBoundingClientRect()

  if (canvas.width !== width || canvas.height !== height) {
    const { devicePixelRatio: ratio = 1 } = window
    const context = canvas.getContext('2d')
    if (!context) return false
    canvas.width = width * ratio
    canvas.height = height * ratio
    context.scale(ratio, ratio)
    return true
  }

  return false
}

function addMouseTrack(canvas: HTMLCanvasElement, onmove: (x: number, y: number) => void) {
  canvas.addEventListener('mousemove', (ev) => {
    const { left, top } = canvas.getBoundingClientRect()
    const x = ev.clientX - left
    const y = ev.clientY - top
    onmove(x, y)
  })
}

function draw(ctx: CanvasRenderingContext2D, x: number, y: number, frameCount: number) {
  ctx.clearRect(0, 0, ctx.canvas.width, ctx.canvas.height)
  ctx.fillStyle = 'rgb(0, 145, 255)'
  ctx.beginPath()
  ctx.arc(x, y, 15 + 10 * Math.sin(frameCount * 0.07) ** 2, 0, 2 * Math.PI)
  ctx.fill()
}

function crosshair(x: number, y: number, lw: number, c: string, ctx: CanvasRenderingContext2D) {
  ctx.save()
  ctx.beginPath()
  ctx.strokeStyle = c
  ctx.lineWidth = lw
  const s = 20
  ctx.moveTo(x, y - s)
  ctx.lineTo(x, y + s)
  ctx.moveTo(x - s, y)
  ctx.lineTo(x + s, y)
  ctx.closePath()
  ctx.stroke()
  ctx.restore()
}

function dist(x0: number, y0: number, x1: number, y1: number): number {
  return Math.sqrt(Math.pow(x1 - x0, 2) + Math.pow(y1 - y0, 2))
}

const useCanvas = (draw: (ctx: CanvasRenderingContext2D, x: number, y: number, frameCount: number) => void, setd: (d: number) => void, onmove: (x: number, y: number) => void) => {

  const canvasRef = useRef<HTMLCanvasElement>(null)

  let mx = 0
  let my = 0
  let d = 0

  useEffect(() => {
    const canvas = canvasRef.current
    if (!canvas) return
    const context = canvas.getContext('2d')

    if (!context) return

    const cx = context.canvas.getBoundingClientRect().width / 2
    const cy = context.canvas.getBoundingClientRect().height / 2

    resizeCanvas(canvas)

    addMouseTrack(canvas, (x, y) => {
      mx = x
      my = y
      onmove(x, y)
      d = dist(cx, cy, x, y)
      setd(d)
    })

    let frameCount = 0
    let animationFrameId: number

    const render = () => {
      frameCount++
      draw(context, cx, cy, frameCount)
      if (d < 20) {
        crosshair(mx, my, 3, '#00f', context)
      } else {
        crosshair(mx, my, 2, '#fa0', context)
      }
      animationFrameId = window.requestAnimationFrame(render)
    }
    render()

    return () => {
      window.cancelAnimationFrame(animationFrameId)
    }
  }, [draw])

  return canvasRef
}

function Overview() {

  const [x, setx] = useState(0)
  const [y, sety] = useState(0)
  const [d, setd] = useState(0)

  const canvasRef = useCanvas(draw, setd, (x: number, y: number) => {
    setx(x)
    sety(y)
  })

  return <>
    <p>{x}, {y}, Distance: {d.toFixed()}</p>
    <section id="center">
      <canvas ref={canvasRef}></canvas>
    </section>
  </>
}

export default Overview;
