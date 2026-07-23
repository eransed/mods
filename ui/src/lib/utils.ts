export function msPretty(ms: number, dec = 1, lvl = 0): string {
  if (isNaN(ms)) return '-'
  if (lvl > 6) {
    return ''
  }

  const MS_YEAR = 12 * (365 / 12) * 24 * 60 * 60 * 1e3
  const MS_MONTH = (365 / 12) * 24 * 60 * 60 * 1e3
  const MS_DAY = 24 * 60 * 60 * 1e3
  const MS_HOUR = 60 * 60 * 1e3
  const MS_MINUTE = 60 * 1e3
  const MS_SECOND = 1e3

  let v = 0
  let u = ''
  let m = ''
  let r = 0

  if (ms >= MS_YEAR) {
    v = ms / MS_YEAR
    r = ms % MS_YEAR
    u = 'y'
    dec = 0
  } else if (ms >= MS_MONTH) {
    v = ms / MS_MONTH
    r = ms % MS_MONTH
    u = 'mon'
    dec = 0
  } else if (ms >= MS_DAY) {
    v = ms / MS_DAY
    r = ms % MS_DAY
    u = 'd'
    dec = 0
  } else if (ms >= MS_HOUR) {
    v = ms / MS_HOUR
    r = ms % MS_HOUR
    u = 'h'
    dec = 0
  } else if (ms >= MS_MINUTE) {
    v = ms / MS_MINUTE
    r = ms % MS_MINUTE
    u = 'm'
    dec = 0
  } else if (ms >= MS_SECOND) {
    v = ms / MS_SECOND
    r = ms % MS_SECOND
    u = 's'
    dec = 0
  } else {
    v = ms
    u = 'ms'
    dec = 0
  }

  // if (v.toFixed(0) !== '1') {
  //   m = 's'
  // }

  const out = `${v.toFixed(dec)}${u}${m}`
  // console.log(`ms=${ms}, out=${out}, lvl=${lvl}`)

  if (r >= 1e3) {
    const next = msPretty(r, dec, ++lvl)
    if (next.length === 0) {
      return `${out}`
    } else {
      return `${out} ${next}`
    }
  } else {
    return out
  }
}
