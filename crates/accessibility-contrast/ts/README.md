# @harborgrid/accessibility-contrast

Enterprise-grade color contrast analyzer with WCAG 2.1 and APCA compliance. A comprehensive TypeScript/React library for ensuring color accessibility in your applications.

## Features

- **WCAG 2.1 Compliance**: Full support for WCAG 2.1 contrast requirements (AA & AAA)
- **APCA Support**: Advanced Perceptual Contrast Algorithm for more accurate contrast measurements
- **Color Blindness Simulation**: Simulate 8 types of color vision deficiencies
- **Automatic Suggestions**: Get accessible color alternatives automatically
- **Palette Analysis**: Analyze entire color palettes with contrast matrices
- **Scientific Accuracy**: Implements CIE color spaces (LAB, LCH, XYZ) for precise calculations
- **React Components**: Production-ready UI components
- **TypeScript**: Full type safety and IntelliSense support
- **Zero Dependencies**: Core algorithms have no external dependencies

## Installation

```bash
npm install @harborgrid/accessibility-contrast
```

## Quick Start

### Simple Contrast Check

```typescript
import { checkContrast, isAccessible } from '@harborgrid/accessibility-contrast';

// Check contrast ratio
const ratio = checkContrast('#000000', '#FFFFFF');
console.log(ratio); // 21

// Check WCAG AA compliance
const accessible = isAccessible('#3B82F6', '#FFFFFF');
console.log(accessible); // true
```

### Using React Components

```tsx
import { ContrastAnalyzer } from '@harborgrid/accessibility-contrast';

function App() {
  return (
    <ContrastAnalyzer
      initialForeground="#000000"
      initialBackground="#FFFFFF"
      showSuggestions={true}
    />
  );
}
```

### Using Hooks

```tsx
import { useContrast } from '@harborgrid/accessibility-contrast';

function MyComponent() {
  const {
    contrast,
    isAccessible,
    grade,
    suggestions
  } = useContrast({
    foreground: '#3B82F6',
    background: '#FFFFFF',
    autoSuggest: true
  });

  return (
    <div>
      <p>Contrast Ratio: {contrast?.ratio.toFixed(2)}:1</p>
      <p>Grade: {grade}</p>
      <p>WCAG AA: {isAccessible ? 'Pass' : 'Fail'}</p>
    </div>
  );
}
```

## API Reference

### Core Functions

#### `calculateContrast(foreground: RGB, background: RGB): ContrastResult`

Calculate complete contrast analysis including WCAG and APCA.

```typescript
import { calculateContrast } from '@harborgrid/accessibility-contrast';

const result = calculateContrast(
  { r: 0, g: 0, b: 0 },
  { r: 255, g: 255, b: 255 }
);

console.log(result.ratio); // 21
console.log(result.wcag.normalTextAA); // true
console.log(result.apca.score); // 106
```

#### `generateColorSuggestions(foreground: RGB, background: RGB, options: OptimizationOptions): ColorSuggestion[]`

Generate accessible color alternatives.

```typescript
import { generateColorSuggestions, WCAGConformance } from '@harborgrid/accessibility-contrast';

const suggestions = generateColorSuggestions(
  { r: 100, g: 100, b: 100 },
  { r: 255, g: 255, b: 255 },
  {
    target: WCAGConformance.NORMAL_TEXT_AA,
    preserveHue: true,
    suggestionCount: 10
  }
);
```

### Color Conversion

```typescript
import {
  rgbToHsl,
  rgbToLab,
  rgbToLch,
  hexToRgb,
  rgbToHex
} from '@harborgrid/accessibility-contrast';

// Convert colors
const hsl = rgbToHsl({ r: 59, g: 130, b: 246 });
const lab = rgbToLab({ r: 59, g: 130, b: 246 });
const rgb = hexToRgb('#3B82F6');
const hex = rgbToHex({ r: 59, g: 130, b: 246 });
```

### Color Blindness Simulation

```typescript
import {
  simulateColorBlindness,
  ColorBlindnessType
} from '@harborgrid/accessibility-contrast';

const simulated = simulateColorBlindness(
  { r: 255, g: 0, b: 0 },
  ColorBlindnessType.DEUTERANOPIA
);
```

## React Components

### ContrastAnalyzer

Complete contrast analysis UI with color pickers and suggestions.

```tsx
<ContrastAnalyzer
  initialForeground="#000000"
  initialBackground="#FFFFFF"
  showSuggestions={true}
  onChange={(fg, bg) => console.log(fg, bg)}
/>
```

### PaletteBuilder

Build and analyze accessible color palettes.

```tsx
<PaletteBuilder
  initialName="My Palette"
  background="#FFFFFF"
  onChange={(colors) => console.log(colors)}
/>
```

### ColorBlindSimulator

Simulate color blindness for testing.

```tsx
<ColorBlindSimulator
  initialColor="#3B82F6"
  showAll={true}
/>
```

### ContrastSuggestions

Display accessible color alternatives.

```tsx
<ContrastSuggestions
  foreground={{ r: 100, g: 100, b: 100 }}
  background={{ r: 255, g: 255, b: 255 }}
  maxSuggestions={10}
  onSelect={(color) => console.log(color)}
/>
```

## React Hooks

### useContrast

```tsx
const {
  contrast,           // Full contrast analysis
  foregroundRGB,      // Current foreground color
  backgroundRGB,      // Current background color
  isAccessible,       // WCAG AA compliance
  isExcellent,        // WCAG AAA compliance
  grade,              // Letter grade (A+ to F)
  suggestions,        // Accessible alternatives
  setForeground,      // Update foreground
  setBackground       // Update background
} = useContrast({
  foreground: '#000000',
  background: '#FFFFFF',
  autoSuggest: true
});
```

### useColorBlindness

```tsx
const {
  original,           // Original color
  simulated,          // Simulated color
  type,               // Current simulation type
  severity,           // Severity (0-1)
  allSimulations,     // All simulation results
  setType,            // Change type
  setSeverity         // Change severity
} = useColorBlindness({
  color: '#3B82F6',
  type: ColorBlindnessType.DEUTERANOPIA,
  simulateAll: true
});
```

### usePalette

```tsx
const {
  palette,            // Palette data with contrast matrix
  isCompliant,        // WCAG compliance status
  addColor,           // Add color to palette
  removeColor,        // Remove color from palette
  optimize,           // Auto-optimize for accessibility
  exportJSON          // Export palette as JSON
} = usePalette({
  name: 'My Palette',
  background: '#FFFFFF'
});
```

## Color Spaces

This library implements scientifically accurate color space conversions:

- **RGB**: Standard red-green-blue color space
- **HSL**: Hue-saturation-lightness (perceptual)
- **LAB**: CIE L\*a\*b\* (perceptually uniform)
- **LCH**: Cylindrical representation of LAB
- **XYZ**: CIE 1931 color space

## WCAG 2.1 Compliance

The library checks against all WCAG 2.1 contrast requirements:

| Level | Normal Text | Large Text | UI Components |
|-------|-------------|------------|---------------|
| AA    | 4.5:1       | 3:1        | 3:1           |
| AAA   | 7:1         | 4.5:1      | -             |

**Large text** is defined as:
- 18pt (24px) or larger
- 14pt (18.66px) or larger if bold

## APCA (Advanced Perceptual Contrast Algorithm)

APCA provides more accurate contrast measurements based on human perception:

- **Lc ≥ 75**: Body text
- **Lc ≥ 60**: Large text / subtext
- **Lc ≥ 45**: UI components / placeholders
- **Lc ≥ 30**: Disabled text (informational only)

## Color Blindness Types

Supports simulation of 8 types of color vision deficiency:

### Complete (Dichromacy)
- **Protanopia**: Red-blind (~1% of males)
- **Deuteranopia**: Green-blind (~1% of males)
- **Tritanopia**: Blue-blind (~0.001%)
- **Achromatopsia**: Total color blindness (~0.003%)

### Partial (Anomalous Trichromacy)
- **Protanomaly**: Red-weak (~1% of males)
- **Deuteranomaly**: Green-weak (~5% of males)
- **Tritanomaly**: Blue-weak (very rare)
- **Achromatomaly**: Blue cone monochromacy (very rare)

## Examples

### Find Accessible Alternative

```typescript
import { findBestAccessibleColor, WCAGConformance } from '@harborgrid/accessibility-contrast';

const accessible = findBestAccessibleColor(
  { r: 100, g: 100, b: 100 }, // Current color
  { r: 255, g: 255, b: 255 }, // Background
  {
    target: WCAGConformance.NORMAL_TEXT_AA,
    preserveHue: true
  }
);
```

### Optimize Entire Palette

```typescript
import { optimizePalette } from '@harborgrid/accessibility-contrast';

const colors = [
  { r: 59, g: 130, b: 246 },
  { r: 239, g: 68, b: 68 },
  { r: 34, g: 197, b: 94 }
];

const optimized = optimizePalette(
  colors,
  { r: 255, g: 255, b: 255 }, // Background
  4.5 // Minimum ratio
);
```

### Calculate Color Distance

```typescript
import { deltaE2000, rgbToLab } from '@harborgrid/accessibility-contrast';

const lab1 = rgbToLab({ r: 255, g: 0, b: 0 });
const lab2 = rgbToLab({ r: 200, g: 0, b: 0 });

const distance = deltaE2000(lab1, lab2);
console.log(distance); // CIE ΔE 2000 distance
```

## Browser Support

- Chrome/Edge: Latest 2 versions
- Firefox: Latest 2 versions
- Safari: Latest 2 versions
- iOS Safari: Latest 2 versions
- Chrome Android: Latest version

## TypeScript

This library is written in TypeScript and provides comprehensive type definitions.

```typescript
import type {
  RGB,
  HSL,
  LAB,
  LCH,
  ContrastResult,
  ColorSuggestion,
  ColorBlindnessType,
  WCAGConformance
} from '@harborgrid/accessibility-contrast';
```

## Performance

All algorithms are optimized for performance:

- Contrast calculation: < 1ms
- Color conversion: < 1ms
- Suggestion generation: < 50ms (10 suggestions)
- Color blindness simulation: < 1ms per color

## License

MIT License - see LICENSE file for details

## Contributing

Contributions are welcome! Please see CONTRIBUTING.md for guidelines.

## Credits

- WCAG 2.1 algorithms based on W3C specifications
- APCA implementation based on research by Andrew Somers
- Color blindness simulation based on research by Brettel, Viénot, and Mollon
- CIE color space conversions based on International Commission on Illumination standards

## Links

- [WCAG 2.1 Guidelines](https://www.w3.org/WAI/WCAG21/quickref/)
- [APCA Documentation](https://git.apcacontrast.com/)
- [CIE Color Spaces](https://en.wikipedia.org/wiki/CIELAB_color_space)
- [Color Blindness Research](https://www.color-blindness.com/)

---

Built with ❤️ by HarborGrid for the Meridian GIS Platform
