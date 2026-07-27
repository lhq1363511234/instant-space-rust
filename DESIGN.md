# inspace Design System

## Direction: 宋式空间 / Song Spatial Editorial

This is not an “ancient Chinese skin.” It translates Song-dynasty aesthetic principles into a modern location product:

- **疏** — whitespace groups content before borders do.
- **静** — moon-white paper, ink, and celadon form the environment; cinnabar is reserved for seals, critical state, and focus.
- **雅** — serif display typography carries editorial voice; operational text remains plain and readable.
- **序** — shared baselines and strong spacing steps establish hierarchy; each first viewport has one dominant focus.
- **生** — motion resembles a scroll unrolling or ink settling, never perpetual floating or decorative bounce.

## Product truth

inspace builds the meso layer of the physical world. Maps help people arrive; inspace helps them enter a real place, understand it, participate, remember, and leave something behind.

## Visual tokens

- Ground: `#fcfbf7` / `#f6f3ea`
- Ink: `#211f1a` / `#38352e`
- Celadon: `#667568` / `#edf1ec`
- Cinnabar seal: `#a43b2d`, used sparingly
- Rules: warm paper lines, normally below full opacity
- Radius: 1–4px; pills only where the control semantics require one
- Shadow: none by default; elevation is communicated by spacing and overlap

## Typography

- Display: `Noto Serif SC`, Songti fallbacks
- UI/body: `Noto Sans SC`, system fallbacks
- Major headings: weight 500–600, balanced, restrained tracking
- Body copy: line-height 1.75–2.0, comfortable measure under 720px

## Layout

- Stable left application shell remains.
- Page grouping relies on 64–120px section gaps.
- Cards are reserved for independently actionable objects; directories and admin data use rows, ledgers, and dividers.
- Desktop editorial pages use asymmetric columns and generous negative space.
- Mobile recomposes into one deliberate reading column; fixed navigation always receives reserved safe space.

## Motion

- One authored homepage entrance: scroll unroll + ink settle.
- Operational pages use only state and hierarchy feedback.
- Real-time interactions 140–180ms; system outcomes 420–700ms.
- `prefers-reduced-motion` preserves every state without movement.

## Anti-patterns

- No card-inside-card layouts.
- No generic SaaS gradients, colored glows, large soft shadows, or excessive rounded rectangles.
- No numbering or red marks unless sequence/state is meaningful.
- No translucent gray body copy.
- No decorative motion that asks for attention after the task is clear.
