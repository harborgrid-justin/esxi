# Meridian GIS Platform v0.4.0 - Warning Fixes Guide

## Overview
This document provides detailed fix instructions for all build warnings found in v0.4.0.
Each fix includes:
- File path
- Line number
- Warning message
- Recommended fix
- Impact level
- Code examples

**Generated**: 2026-01-01 UTC
**Agent**: BUILD WARNING AGENT v0.4.0

---

## Quick Fix Summary

| Category | Count | Auto-Fix | Command | Time Estimate |
|----------|-------|----------|---------|---------------|
| Missing Dependencies | 200+ | ✅ Yes | `npm install` | 5-10 min |
| Implicit any Types | 150+ | ❌ No | Manual typing | 4-6 hours |
| Unused Variables | 50-100 | ✅ Yes | `npm run lint:fix` | 2-5 min |
| React Hooks Deps | 15-30 | ⚠️ Partial | Semi-auto + review | 1-2 hours |
| Accessibility Issues | 10-30 | ❌ No | Manual fixes | 2-4 hours |
| Console Usage | 20-40 | ❌ No | Replace with logger | 1-2 hours |
| Missing Return Types | 30-60 | ❌ No | Manual typing | 2-3 hours |
| Async/Promise Issues | 10-20 | ❌ No | Manual fixes | 1-2 hours |

**Total Estimated Fix Time**: 12-20 hours for manual fixes

---

## Priority 0: Critical Fixes (Do First)

### FIX-001: Install Missing Dependencies

**Impact**: CRITICAL - Blocks all other fixes
**Auto-Fix**: ✅ Yes
**Time**: 5-10 minutes

#### Problem
```
error TS2307: Cannot find module '@tanstack/react-query'
error TS2307: Cannot find module 'lucide-react'
error TS2307: Cannot find module 'react'
```

#### Solution
```bash
cd /home/user/esxi
npm install
```

#### Expected Packages Installed
```json
{
  "dependencies": {
    "react": "^18.2.0",
    "react-dom": "^18.2.0",
    "@tanstack/react-query": "^5.x",
    "lucide-react": "^0.x",
    "@radix-ui/react-dialog": "^1.x",
    "@radix-ui/react-select": "^1.x",
    "@radix-ui/react-slider": "^1.x",
    "@radix-ui/react-tabs": "^1.x",
    "maplibre-gl": "^4.x"
  },
  "devDependencies": {
    "@types/react": "^18.2.0",
    "@types/react-dom": "^18.2.0",
    "vite": "^5.x"
  }
}
```

#### Verification
```bash
# Should pass after install
npm run typecheck
```

---

### FIX-002: Fix Cargo.toml Workspace Configuration

**Impact**: CRITICAL - Blocks Rust builds
**Auto-Fix**: ❌ No
**Time**: 5 minutes

#### Problem
```
error: failed to load manifest for workspace member `/home/user/esxi/crates/accessibility-dashboard`
Caused by: No such file or directory (os error 2)
```

#### Solution

**File**: `/home/user/esxi/Cargo.toml`

**Current** (Lines 44-55):
```toml
# Enterprise Web Accessibility SaaS (v0.3.0)
"crates/accessibility-scanner",
"crates/accessibility-dashboard",
"crates/accessibility-realtime",
```

**Fixed**:
```toml
# Enterprise Web Accessibility SaaS (v0.4.0)
# Note: These crates are now TypeScript-only (in */ts/ subdirectories)
# Rust implementations removed in v0.4.0
# "crates/accessibility-scanner",
# "crates/accessibility-dashboard",
# "crates/accessibility-realtime",
# ... (comment out all accessibility-* crates that don't have Cargo.toml)
```

**Or** create Rust stubs if needed:
```bash
# Only if Rust bindings are needed
cd /home/user/esxi/crates/accessibility-dashboard
cargo init --lib
```

#### Verification
```bash
cargo build --workspace
```

---

### FIX-003: Accessibility Errors (jsx-a11y)

**Impact**: CRITICAL - Required for accessibility SaaS platform
**Auto-Fix**: ❌ No
**Time**: 2-4 hours
**Count**: 10-30 errors (estimated)

#### Common Patterns & Fixes

##### Pattern 1: Missing alt text
**Error**: `jsx-a11y/alt-text`
**Files**: All components with `<img>` tags

**Before**:
```typescript
<img src={logo} />
```

**After**:
```typescript
<img src={logo} alt="Company logo" />
```

---

##### Pattern 2: Click without keyboard handler
**Error**: `jsx-a11y/click-events-have-key-events`
**Files**: Interactive elements

**Before**:
```typescript
<div onClick={handleClick}>Click me</div>
```

**After**:
```typescript
<div
  onClick={handleClick}
  onKeyDown={(e) => {
    if (e.key === 'Enter' || e.key === ' ') {
      handleClick(e);
    }
  }}
  role="button"
  tabIndex={0}
>
  Click me
</div>
```

**Better**: Use semantic HTML
```typescript
<button onClick={handleClick}>Click me</button>
```

---

##### Pattern 3: Missing form labels
**Error**: `jsx-a11y/label-has-associated-control`
**Files**: Form components

**Before**:
```typescript
<label>Name</label>
<input type="text" />
```

**After**:
```typescript
<label htmlFor="name-input">Name</label>
<input type="text" id="name-input" />
```

**Or**:
```typescript
<label>
  Name
  <input type="text" />
</label>
```

---

##### Pattern 4: Autofocus usage
**Error**: `jsx-a11y/no-autofocus` (warning)
**Files**: Form/modal components

**Before**:
```typescript
<input type="text" autoFocus />
```

**After**:
```typescript
// Remove autoFocus or document why it's needed
<input
  type="text"
  // eslint-disable-next-line jsx-a11y/no-autofocus -- Modal dialog needs focus trap
  autoFocus
/>
```

---

##### Pattern 5: Missing ARIA labels
**Error**: Various `jsx-a11y/aria-*` rules

**Before**:
```typescript
<button>
  <Icon />
</button>
```

**After**:
```typescript
<button aria-label="Delete item">
  <Icon />
</button>
```

---

### FIX-004: Async/Promise Handling Errors

**Impact**: HIGH - Potential runtime errors
**Auto-Fix**: ❌ No
**Time**: 1-2 hours
**Count**: 10-20 errors (estimated)

#### Error Types

##### Type 1: Floating promises
**Error**: `@typescript-eslint/no-floating-promises`

**Before**:
```typescript
function handleSubmit() {
  fetchData(); // Warning: Promise not handled
}
```

**After**:
```typescript
async function handleSubmit() {
  await fetchData();
}

// Or with error handling
function handleSubmit() {
  fetchData().catch((error) => {
    console.error('Failed to fetch:', error);
  });
}

// Or explicitly void
function handleSubmit() {
  void fetchData();
}
```

---

##### Type 2: Misused promises
**Error**: `@typescript-eslint/no-misused-promises`

**Before**:
```typescript
<button onClick={async () => await fetchData()}>
  Submit
</button>
```

**After**:
```typescript
<button onClick={() => {
  void fetchData();
}}>
  Submit
</button>

// Or better:
const handleSubmit = async () => {
  try {
    await fetchData();
  } catch (error) {
    console.error(error);
  }
};

<button onClick={handleSubmit}>Submit</button>
```

---

##### Type 3: Await non-promise
**Error**: `@typescript-eslint/await-thenable`

**Before**:
```typescript
await synchronousFunction();
```

**After**:
```typescript
synchronousFunction();
```

---

##### Type 4: Async without await
**Error**: `@typescript-eslint/require-await`

**Before**:
```typescript
async function doSomething() {
  return 42; // No await used
}
```

**After**:
```typescript
function doSomething() {
  return 42;
}
```

---

## Priority 1: High Priority Fixes

### FIX-005: Implicit 'any' Type Parameters

**Impact**: HIGH - Type safety
**Auto-Fix**: ❌ No
**Time**: 4-6 hours
**Count**: 150+ instances

#### Common Patterns & Fixes

##### Pattern 1: Event handlers
**Error**: `Parameter 'e' implicitly has an 'any' type`
**Files**: `/home/user/esxi/web/src/components/Analysis/AnalysisPanel.tsx` (Lines 102, 156)

**Before**:
```typescript
const handleChange = (e) => {
  setValue(e.target.value);
};
```

**After**:
```typescript
const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
  setValue(e.target.value);
};
```

**Common Event Types**:
```typescript
// Click events
onClick={(e: React.MouseEvent<HTMLButtonElement>) => {}}

// Change events
onChange={(e: React.ChangeEvent<HTMLInputElement>) => {}}

// Form submit
onSubmit={(e: React.FormEvent<HTMLFormElement>) => {}}

// Keyboard events
onKeyDown={(e: React.KeyboardEvent<HTMLDivElement>) => {}}

// Focus events
onFocus={(e: React.FocusEvent<HTMLInputElement>) => {}}
```

---

##### Pattern 2: State updater functions
**Error**: `Parameter 'prev' implicitly has an 'any' type`
**Files**:
- `/home/user/esxi/web/src/components/Analysis/AnalysisPanel.tsx` (Lines 42, 47, 67, 103)
- `/home/user/esxi/web/src/components/Map/DrawingTools.tsx` (Lines 34, 73, 118)

**Before**:
```typescript
setCount((prev) => prev + 1);
```

**After**:
```typescript
// Option 1: Infer from state type
const [count, setCount] = useState(0);
setCount((prev) => prev + 1); // Type inferred as number

// Option 2: Explicit typing
setCount((prev: number) => prev + 1);

// Option 3: Complex state
interface FormState {
  name: string;
  email: string;
}
setFormData((prev: FormState) => ({
  ...prev,
  name: newName
}));
```

---

##### Pattern 3: Array callbacks
**Error**: `Parameter 'item' implicitly has an 'any' type`
**Files**: `/home/user/esxi/web/src/components/Analysis/AnalysisPanel.tsx` (Line 48)

**Before**:
```typescript
items.map((item) => item.name);
items.filter((item) => item.active);
```

**After**:
```typescript
// Option 1: Type the array
const items: Item[] = [...];
items.map((item) => item.name); // Type inferred

// Option 2: Inline typing
items.map((item: Item) => item.name);

// Option 3: Generic type assertion
items.map((item: { name: string; active: boolean }) => item.name);
```

---

##### Pattern 4: Generic callbacks
**Error**: `Parameter 'value' implicitly has an 'any' type`
**Files**:
- `/home/user/esxi/web/src/components/Analysis/AnalysisPanel.tsx` (Lines 66, 117)
- `/home/user/esxi/web/src/components/Map/LayerPanel.tsx` (Line 83)

**Before**:
```typescript
onValueChange={(value) => {
  setSelected(value);
}}
```

**After**:
```typescript
// For Radix UI components - check their docs
import * as Select from '@radix-ui/react-select';

<Select.Root
  onValueChange={(value: string) => {
    setSelected(value);
  }}
>
```

---

##### Pattern 5: Object parameters
**Error**: `Parameter 'layer' implicitly has an 'any' type`
**Files**: `/home/user/esxi/web/src/components/Map/LayerPanel.tsx` (Line 39)

**Before**:
```typescript
layers.map((layer) => (
  <div key={layer.id}>{layer.name}</div>
))
```

**After**:
```typescript
interface Layer {
  id: string;
  name: string;
  visible: boolean;
  opacity: number;
}

layers.map((layer: Layer) => (
  <div key={layer.id}>{layer.name}</div>
))
```

---

### FIX-006: Missing Return Types

**Impact**: MEDIUM-HIGH - Type safety
**Auto-Fix**: ❌ No
**Time**: 2-3 hours
**Count**: 30-60 functions (estimated)

#### Common Patterns & Fixes

##### Pattern 1: Component functions
**Before**:
```typescript
function MyComponent({ title }) {
  return <div>{title}</div>;
}
```

**After**:
```typescript
interface MyComponentProps {
  title: string;
}

function MyComponent({ title }: MyComponentProps): JSX.Element {
  return <div>{title}</div>;
}

// Or using React.FC (less preferred)
const MyComponent: React.FC<MyComponentProps> = ({ title }) => {
  return <div>{title}</div>;
};
```

---

##### Pattern 2: Event handlers
**Before**:
```typescript
function handleClick(e: React.MouseEvent) {
  console.log('Clicked');
}
```

**After**:
```typescript
function handleClick(e: React.MouseEvent): void {
  console.log('Clicked');
}
```

---

##### Pattern 3: Async functions
**Before**:
```typescript
async function fetchData() {
  const response = await fetch('/api/data');
  return response.json();
}
```

**After**:
```typescript
interface DataResponse {
  items: Item[];
  total: number;
}

async function fetchData(): Promise<DataResponse> {
  const response = await fetch('/api/data');
  return response.json();
}
```

---

##### Pattern 4: Utility functions
**Before**:
```typescript
function formatDate(date) {
  return new Intl.DateTimeFormat('en-US').format(date);
}
```

**After**:
```typescript
function formatDate(date: Date): string {
  return new Intl.DateTimeFormat('en-US').format(date);
}
```

---

### FIX-007: React Hooks Dependencies

**Impact**: MEDIUM-HIGH - Correctness
**Auto-Fix**: ⚠️ Partial
**Time**: 1-2 hours
**Count**: 15-30 warnings (estimated)

#### Error Pattern
```
Warning: React Hook useEffect has missing dependencies: 'prop1', 'prop2'
```

#### Fixes

##### Fix 1: Add missing dependencies
**Before**:
```typescript
useEffect(() => {
  fetchData(userId, filter);
}, []); // Missing: userId, filter
```

**After**:
```typescript
useEffect(() => {
  fetchData(userId, filter);
}, [userId, filter]);
```

---

##### Fix 2: Use callback refs for functions
**Before**:
```typescript
const handleData = () => {
  console.log(value);
};

useEffect(() => {
  handleData();
}, []); // Missing: handleData (which depends on value)
```

**After**:
```typescript
const handleData = useCallback(() => {
  console.log(value);
}, [value]);

useEffect(() => {
  handleData();
}, [handleData]);
```

---

##### Fix 3: Move function inside effect
**Before**:
```typescript
const processData = (data) => {
  setResult(data.map(item => item.value));
};

useEffect(() => {
  fetchData().then(processData);
}, []); // Missing: processData
```

**After**:
```typescript
useEffect(() => {
  const processData = (data) => {
    setResult(data.map(item => item.value));
  };

  fetchData().then(processData);
}, []);
```

---

##### Fix 4: Intentionally omit (rare)
**Before**:
```typescript
useEffect(() => {
  // This should only run on mount
  initializeApp(config);
}, []); // config is stable but linter warns
```

**After**:
```typescript
useEffect(() => {
  // This should only run on mount
  initializeApp(config);
  // eslint-disable-next-line react-hooks/exhaustive-deps
}, []); // config is from context and stable
```

**Note**: Only use this escape hatch when you're certain it's correct!

---

## Priority 2: Medium Priority Fixes

### FIX-008: Unused Variables and Imports

**Impact**: LOW-MEDIUM - Code cleanliness
**Auto-Fix**: ✅ Yes
**Time**: 2-5 minutes
**Count**: 50-100 instances

#### Auto-Fix Command
```bash
npm run lint:fix
```

#### Manual Fixes

##### Example 1: Unused import
**File**: `/home/user/esxi/web/src/components/Map/DrawingTools.tsx` (Line 3)

**Before**:
```typescript
import { useFeatures } from '@/hooks/useFeatures';
import { useLayers } from '@/hooks/useLayers';

export function DrawingTools() {
  const { layers } = useLayers();
  // useFeatures never used
}
```

**After**:
```typescript
import { useLayers } from '@/hooks/useLayers';

export function DrawingTools() {
  const { layers } = useLayers();
}
```

---

##### Example 2: Unused variable
**File**: `/home/user/esxi/web/src/components/Map/LayerPanel.tsx` (Line 12)

**Before**:
```typescript
const [selectedLayer, setSelectedLayer] = useState(null);
const [opacity, setOpacity] = useState(100);
// selectedLayer never used
```

**After**:
```typescript
const [opacity, setOpacity] = useState(100);
```

**Or** prefix with underscore if intentionally unused:
```typescript
const [_selectedLayer, setSelectedLayer] = useState(null);
```

---

##### Example 3: Unused icon import
**File**: `/home/user/esxi/web/src/components/Map/ToolBar.tsx` (Line 2)

**Before**:
```typescript
import { MousePointer2, Pencil, Square } from 'lucide-react';

// Only Pencil and Square used
```

**After**:
```typescript
import { Pencil, Square } from 'lucide-react';
```

---

### FIX-009: Console Usage

**Impact**: LOW - Development experience
**Auto-Fix**: ❌ No
**Time**: 1-2 hours
**Count**: 20-40 instances

#### Rule
```javascript
'no-console': ['warn', { allow: ['warn', 'error'] }]
```

#### Allowed vs Not Allowed

**Not Allowed**:
```typescript
console.log('Debug info');
console.info('Info message');
console.debug('Debug message');
```

**Allowed**:
```typescript
console.warn('Warning message');
console.error('Error message');
```

#### Fix Options

##### Option 1: Remove (preferred for production)
**Before**:
```typescript
function fetchData() {
  console.log('Fetching data...');
  return api.get('/data');
}
```

**After**:
```typescript
function fetchData() {
  return api.get('/data');
}
```

---

##### Option 2: Use logger utility
**Before**:
```typescript
console.log('User logged in:', user);
```

**After**:
Create `/home/user/esxi/web/src/lib/logger.ts`:
```typescript
export const logger = {
  debug: (message: string, ...args: unknown[]) => {
    if (import.meta.env.DEV) {
      console.log(`[DEBUG] ${message}`, ...args);
    }
  },
  info: (message: string, ...args: unknown[]) => {
    if (import.meta.env.DEV) {
      console.log(`[INFO] ${message}`, ...args);
    }
  },
  warn: (message: string, ...args: unknown[]) => {
    console.warn(`[WARN] ${message}`, ...args);
  },
  error: (message: string, ...args: unknown[]) => {
    console.error(`[ERROR] ${message}`, ...args);
  }
};
```

Use it:
```typescript
import { logger } from '@/lib/logger';

logger.debug('User logged in:', user);
```

---

##### Option 3: Suppress warning (rare)
**Before**:
```typescript
function debugFunction() {
  console.log('Debug info');
}
```

**After**:
```typescript
function debugFunction() {
  // eslint-disable-next-line no-console
  console.log('Debug info');
}
```

---

### FIX-010: Missing Namespace Declarations

**Impact**: MEDIUM - Type safety
**Auto-Fix**: ❌ No
**Time**: 30 minutes
**Count**: 10+ instances

#### Pattern: MapLibre GL types

**File**: `/home/user/esxi/web/src/components/Map/DrawingTools.tsx`
**Error**: `Cannot find namespace 'maplibregl'`

**Before**:
```typescript
const handleClick = (e: maplibregl.MapMouseEvent) => {
  // ...
};
```

**After**:

1. Install types:
```bash
npm install --save-dev @types/maplibre-gl
```

2. Import types:
```typescript
import type { MapMouseEvent, Map } from 'maplibre-gl';

const handleClick = (e: MapMouseEvent) => {
  // ...
};
```

---

### FIX-011: Environment Variable Types

**Impact**: MEDIUM - Type safety
**Auto-Fix**: ❌ No
**Time**: 15 minutes
**Count**: 5+ instances

#### Pattern: Vite env variables

**File**: `/home/user/esxi/web/src/api/client.ts`
**Error**: `Property 'env' does not exist on type 'ImportMeta'`

**Before**:
```typescript
const baseURL = import.meta.env.VITE_API_URL;
```

**After**:

Create `/home/user/esxi/web/src/vite-env.d.ts`:
```typescript
/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_API_URL: string;
  readonly VITE_MAP_TOKEN: string;
  // Add all env variables here
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
```

Then:
```typescript
const baseURL = import.meta.env.VITE_API_URL; // Now typed!
```

---

## Priority 3: Low Priority Fixes

### FIX-012: Circular Dependencies

**Impact**: MEDIUM - Architecture
**Auto-Fix**: ❌ No
**Time**: 2-3 hours
**Count**: 5-10 instances (estimated)

#### Detection
```
Warning: Dependency cycle detected: A → B → C → A
```

#### Fixes

##### Pattern 1: Extract shared code
**Before**:
```
// fileA.ts imports fileB
// fileB.ts imports fileA
```

**After**:
```
// Create fileShared.ts with shared code
// fileA.ts imports fileShared
// fileB.ts imports fileShared
```

---

##### Pattern 2: Use dependency injection
**Before**:
```typescript
// userService.ts
import { authService } from './authService';

// authService.ts
import { userService } from './userService';
```

**After**:
```typescript
// userService.ts
export function createUserService(auth: AuthService) {
  // ...
}

// authService.ts
export function createAuthService(user: UserService) {
  // ...
}

// main.ts
const authService = createAuthService();
const userService = createUserService(authService);
```

---

### FIX-013: Deprecated API Usage

**Impact**: LOW-MEDIUM - Future compatibility
**Auto-Fix**: ⚠️ Partial
**Time**: 1-2 hours
**Count**: 5-15 instances (estimated)

#### Common Patterns

##### Pattern 1: React.FC with children
**Before**:
```typescript
const Component: React.FC = ({ children }) => {
  return <div>{children}</div>;
};
```

**After**:
```typescript
const Component: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  return <div>{children}</div>;
};

// Or better (TypeScript 5.1+):
function Component({ children }: { children: React.ReactNode }) {
  return <div>{children}</div>;
}
```

---

##### Pattern 2: Old MapLibre methods
Check MapLibre GL documentation for deprecated methods and migrate to new APIs.

---

## Fix Workflow Recommendations

### Step-by-Step Process

#### Phase 1: Setup (10 minutes)
```bash
# 1. Install dependencies
cd /home/user/esxi
npm install

# 2. Verify installation
npm run typecheck

# 3. Create feature branch
git checkout -b fix/warnings-v0.4
```

#### Phase 2: Critical Fixes (2-4 hours)
```bash
# 1. Fix Cargo.toml
# Edit /home/user/esxi/Cargo.toml

# 2. Fix accessibility issues
# Go through each jsx-a11y error manually

# 3. Fix async/promise issues
# Review all async code

# 4. Test
npm run typecheck
npm run lint:ts
```

#### Phase 3: High Priority Fixes (4-6 hours)
```bash
# 1. Fix implicit any types
# Add type annotations systematically

# 2. Fix missing return types
# Add return types to all functions

# 3. Fix React hooks dependencies
# Review and fix useEffect dependencies

# 4. Test
npm run test
```

#### Phase 4: Medium Priority Fixes (2-3 hours)
```bash
# 1. Auto-fix unused code
npm run lint:fix

# 2. Fix console usage
# Replace with logger

# 3. Fix namespace issues
# Install missing type packages

# 4. Test
npm run build:ts
```

#### Phase 5: Verification
```bash
# 1. Full build
npm run build:all

# 2. Full lint
npm run lint

# 3. Full test
npm run test

# 4. Verify zero warnings
npm run lint:ts 2>&1 | grep -i warning
```

---

## Code Templates

### TypeScript Component Template
```typescript
import React from 'react';

/**
 * Props for MyComponent
 */
interface MyComponentProps {
  /**
   * The title to display
   */
  title: string;
  /**
   * Optional callback when clicked
   */
  onClick?: (id: string) => void;
  /**
   * Child elements
   */
  children?: React.ReactNode;
}

/**
 * MyComponent description
 */
export function MyComponent({
  title,
  onClick,
  children
}: MyComponentProps): JSX.Element {
  const handleClick = (): void => {
    if (onClick) {
      onClick('example-id');
    }
  };

  return (
    <div>
      <h1>{title}</h1>
      <button onClick={handleClick}>Click me</button>
      {children}
    </div>
  );
}
```

### Async Function Template
```typescript
/**
 * Fetches data from the API
 */
export async function fetchData(): Promise<DataResponse> {
  try {
    const response = await fetch('/api/data');

    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }

    const data = await response.json() as DataResponse;
    return data;
  } catch (error) {
    console.error('Failed to fetch data:', error);
    throw error;
  }
}
```

### Event Handler Template
```typescript
const handleChange = (e: React.ChangeEvent<HTMLInputElement>): void => {
  const value = e.target.value;
  setValue(value);
};

const handleSubmit = async (e: React.FormEvent<HTMLFormElement>): Promise<void> => {
  e.preventDefault();

  try {
    await submitForm(formData);
  } catch (error) {
    console.error('Submit failed:', error);
  }
};
```

### Custom Hook Template
```typescript
import { useState, useEffect, useCallback } from 'react';

interface UseDataOptions {
  autoFetch?: boolean;
}

interface UseDataReturn<T> {
  data: T | null;
  loading: boolean;
  error: Error | null;
  refetch: () => Promise<void>;
}

export function useData<T>(
  url: string,
  options: UseDataOptions = {}
): UseDataReturn<T> {
  const [data, setData] = useState<T | null>(null);
  const [loading, setLoading] = useState<boolean>(false);
  const [error, setError] = useState<Error | null>(null);

  const fetchData = useCallback(async (): Promise<void> => {
    setLoading(true);
    setError(null);

    try {
      const response = await fetch(url);
      const result = await response.json() as T;
      setData(result);
    } catch (err) {
      setError(err as Error);
    } finally {
      setLoading(false);
    }
  }, [url]);

  useEffect(() => {
    if (options.autoFetch) {
      void fetchData();
    }
  }, [options.autoFetch, fetchData]);

  return { data, loading, error, refetch: fetchData };
}
```

---

## Testing After Fixes

### Automated Tests
```bash
# Type checking
npm run typecheck

# Linting
npm run lint:ts

# Build
npm run build:ts

# Unit tests
npm run test

# E2E tests (if available)
npm run test:e2e
```

### Manual Verification Checklist

- [ ] All TypeScript files compile without errors
- [ ] All ESLint rules pass
- [ ] No console.log in production code
- [ ] All accessibility errors fixed
- [ ] All async code properly handled
- [ ] All event handlers typed
- [ ] All React hooks have correct dependencies
- [ ] No unused imports/variables
- [ ] Application builds successfully
- [ ] Application runs in development mode
- [ ] Application runs in production mode

---

## Tracking Progress

### Create Progress Log
Create `/home/user/esxi/WARNING_FIXES_PROGRESS.md`:
```markdown
# Warning Fixes Progress - v0.4.0

## Phase 1: Critical Fixes
- [x] FIX-001: Install dependencies (5 min)
- [x] FIX-002: Fix Cargo.toml (5 min)
- [ ] FIX-003: Accessibility errors (2-4 hours)
  - [ ] Missing alt text (20 instances)
  - [ ] Click handlers (15 instances)
  - [ ] Form labels (10 instances)
- [ ] FIX-004: Async/promise issues (1-2 hours)

## Phase 2: High Priority
- [ ] FIX-005: Implicit any types (4-6 hours)
  - [ ] Event handlers (50 instances)
  - [ ] State updaters (40 instances)
  - [ ] Callbacks (60 instances)

...
```

---

## Common Mistakes to Avoid

### Mistake 1: Suppressing Warnings Without Understanding
**Bad**:
```typescript
// eslint-disable-next-line @typescript-eslint/no-explicit-any
const data: any = fetchData();
```

**Good**:
```typescript
interface DataResponse {
  items: Item[];
}
const data: DataResponse = await fetchData();
```

### Mistake 2: Using `any` as a Quick Fix
**Bad**:
```typescript
function process(data: any) {
  return data.items.map((item: any) => item.name);
}
```

**Good**:
```typescript
interface DataItem {
  name: string;
  id: string;
}

interface ProcessData {
  items: DataItem[];
}

function process(data: ProcessData): string[] {
  return data.items.map((item) => item.name);
}
```

### Mistake 3: Ignoring React Hooks Warnings
**Bad**:
```typescript
useEffect(() => {
  fetchData(userId);
  // eslint-disable-next-line react-hooks/exhaustive-deps
}, []);
```

**Good**:
```typescript
useEffect(() => {
  fetchData(userId);
}, [userId]);
```

---

## Resources

### Documentation
- [TypeScript Handbook](https://www.typescriptlang.org/docs/handbook/intro.html)
- [React TypeScript Cheatsheet](https://react-typescript-cheatsheet.netlify.app/)
- [ESLint Rules](https://eslint.org/docs/latest/rules/)
- [jsx-a11y Rules](https://github.com/jsx-eslint/eslint-plugin-jsx-a11y#supported-rules)
- [TypeScript ESLint Rules](https://typescript-eslint.io/rules/)

### Tools
- [TypeScript Playground](https://www.typescriptlang.org/play)
- [ESLint Playground](https://eslint.org/play/)
- [WAVE Accessibility Tool](https://wave.webaim.org/)

---

## Next Steps

1. ✅ Review this guide
2. ⏳ Start with Phase 1 (Critical Fixes)
3. ⏳ Create progress tracking document
4. ⏳ Fix warnings systematically
5. ⏳ Update WARNING_LOG_v0.4.md with actual counts
6. ⏳ Create pull request with fixes
7. ⏳ Set up CI/CD to prevent regressions

---

**Generated**: 2026-01-01 UTC
**Agent**: BUILD WARNING AGENT v0.4.0
**Status**: Ready for implementation
**Estimated Total Fix Time**: 12-20 hours
