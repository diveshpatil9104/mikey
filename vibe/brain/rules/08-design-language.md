# 8. Design Language

## 8.1 Principles

- **Flat, not glossy.** No glassmorphism, gradients or drop shadows.
- **Honest, binary states.** On or off. The only exceptions are the amber "waiting" state and the mic level ring.
- **The screen is the button.** Each half of the phone screen is a tap target.
- **Rotation aware, not rotation reactive.** Glyphs rotate; the layout doesn't.
- **System font only.** No custom typefaces.
- **No decorative animation.** State changes snap. The only motion: drawer slide (standard bottom-sheet), level ring, live preview.

## 8.2 Color palette

| Token | Hex | Usage |
|-------|-----|-------|
| `bg` | `#000000` | Background — pure black, OLED-friendly |
| `surface` | `#111111` | Drawer, dialogs |
| `divider` | `#2C2C2E` | Separators, split line |
| `icon-off` | `#3A3A3C` | Dimmed mic/camera/flip/chevron |
| `mic-on` | `#30D158` | Mic on + level ring |
| `cam-on` | `#0A84FF` | Camera on |
| `status-ok` | `#30D158` | Connected & streaming path live |
| `status-wait` | `#FFD60A` | Waiting for approval / authorization |
| `status-err` | `#FF453A` | No PC |
| `text-primary` | `#FFFFFF` | Readable text |
| `text-secondary` | `#8E8E93` | Sub-labels, version string |

Pressed state: 0.08 white alpha overlay. Nothing else.

## 8.3 Typography

- System default (Roboto on most Android).
- Three sizes only: `17sp` (reserved), `14sp` drawer items, `12sp` metadata.
- Regular 400 everywhere; Medium 500 for drawer section headers.
- Sentence case. Never all-caps.

## 8.4 Icons & sizes

- Mic/camera icons 56 dp; outline 2 dp stroke when off, filled when on.
- Flip button 40 dp touch target (24 dp glyph), 16 dp from top-left edges.
- Status dot 10 dp rounded square (3 dp radius), 16 dp from bottom-right edges.
- Chevron 24 dp glyph, 48 dp touch target, centered on the split line.
- All touch targets ≥ 48 dp.
