# Enterprise SaaS Platform v0.4.0 - Error Fixes

**Timestamp**: 2026-01-01 00:34:00 UTC
**Agent**: BUILD ERROR AGENT
**Session**: TypeScript/React error resolution for v0.4.0
**Platform**: Enterprise Web-Accessibility SaaS Platform

---

## Fix Summary

| Error # | File | Status | Priority | Assigned |
|---------|------|--------|----------|----------|
| 1 | `/home/user/esxi/crates/meridian-ui-components/ts/package.json` | 🔴 Open | Critical | Error Agent |
| 2 | `/home/user/esxi/crates/enterprise-notifications/ts/package.json` | 🔴 Open | Critical | Error Agent |

---

## Error #1: Rollup Version Conflict

### Error Details
**File**: `/home/user/esxi/crates/meridian-ui-components/ts/package.json`
**Line**: Unknown (package.json dependency section)
**Priority**: **CRITICAL** (P0)
**Status**: 🔴 OPEN

### Error Message
```
ERESOLVE unable to resolve dependency tree

While resolving: @meridian/ui-components@0.2.5
Found: rollup@4.54.0
node_modules/rollup
  dev rollup@"^4.9.6" from @meridian/ui-components@0.2.5

Could not resolve dependency:
peer rollup@"^2.0.0" from rollup-plugin-terser@7.0.2
node_modules/rollup-plugin-terser
  dev rollup-plugin-terser@"^7.0.2" from @meridian/ui-components@0.2.5
```

### Root Cause Analysis

**Problem**:
1. Package uses Rollup v4.9.6 (modern version)
2. Also uses rollup-plugin-terser v7.0.2
3. rollup-plugin-terser v7.0.2 has peer dependency: `rollup@^2.0.0`
4. Rollup v2.x and v4.x are incompatible
5. npm cannot resolve this peer dependency conflict

**Why This Happened**:
- rollup-plugin-terser was last updated in 2021
- It only supports Rollup v2.x
- Rollup v4.x was released in 2023
- The plugin is now deprecated/unmaintained

### Recommended Fix

**Option 1: Replace rollup-plugin-terser with @rollup/plugin-terser (RECOMMENDED)**

This is the official, maintained replacement that supports Rollup v4.x.

**Action**: Update `/home/user/esxi/crates/meridian-ui-components/ts/package.json`

**Remove**:
```json
"rollup-plugin-terser": "^7.0.2"
```

**Add**:
```json
"@rollup/plugin-terser": "^0.4.4"
```

**Impact**:
- ✅ Compatible with Rollup v4.x
- ✅ Actively maintained
- ✅ Drop-in replacement
- ⚠️ May require minor import statement updates in rollup.config.js

**Code Changes Required**:
If there's a `rollup.config.js` or similar, update:

```javascript
// Old import
import { terser } from 'rollup-plugin-terser';

// New import
import terser from '@rollup/plugin-terser';
```

**Confidence**: 🟢 High - This is the official migration path

---

**Option 2: Downgrade Rollup to v2.x (NOT RECOMMENDED)**

**Action**: Update `/home/user/esxi/crates/meridian-ui-components/ts/package.json`

**Change**:
```json
"rollup": "^4.9.6"
```

**To**:
```json
"rollup": "^2.79.1"
```

**Impact**:
- ✅ Resolves peer dependency conflict
- ❌ Uses outdated Rollup version (v2.x from 2022)
- ❌ Missing Rollup v4.x performance improvements
- ❌ Missing Rollup v4.x features
- ❌ Security updates only in v4.x

**Confidence**: 🟡 Medium - Works but uses outdated tooling

---

**Option 3: Remove rollup-plugin-terser entirely**

If terser minification is not critical for development builds:

**Action**: Remove from package.json and rollup config

**Impact**:
- ✅ Removes dependency conflict
- ✅ Faster builds (no minification)
- ❌ Larger bundle sizes
- ❌ Not suitable for production builds

**Confidence**: 🟡 Medium - Only for development

---

### Implementation Steps

**Step 1**: Read current package.json
```bash
cat /home/user/esxi/crates/meridian-ui-components/ts/package.json
```

**Step 2**: Update dependency (Option 1 - RECOMMENDED)
- Remove: `"rollup-plugin-terser": "^7.0.2"`
- Add: `"@rollup/plugin-terser": "^0.4.4"`

**Step 3**: Check for rollup config files
```bash
find /home/user/esxi/crates/meridian-ui-components/ts -name "rollup.config.*" -o -name "vite.config.*"
```

**Step 4**: Update import statements if needed
- Old: `import { terser } from 'rollup-plugin-terser';`
- New: `import terser from '@rollup/plugin-terser';`

**Step 5**: Retry npm install
```bash
npm install
```

**Step 6**: Verify build works
```bash
npm run build --workspace=crates/meridian-ui-components/ts
```

---

### Validation Checklist

After applying fix:
- [ ] npm install completes successfully
- [ ] No peer dependency warnings
- [ ] Package builds successfully
- [ ] Bundle size is reasonable (minified)
- [ ] Other workspaces can depend on this package
- [ ] All workspaces install successfully

---

### Additional Investigation Needed

**Check for similar issues**:
```bash
# Search for other uses of rollup-plugin-terser
grep -r "rollup-plugin-terser" /home/user/esxi/crates/*/ts/package.json

# Check rollup versions across all packages
grep -r '"rollup"' /home/user/esxi/crates/*/ts/package.json
```

**Expected findings**:
- May find rollup-plugin-terser in other packages
- May find inconsistent rollup versions
- Should standardize across all packages

---

## Error #2: Missing @types/juice Package

### Error Details
**File**: `/home/user/esxi/crates/enterprise-notifications/ts/package.json`
**Line**: 105
**Priority**: **CRITICAL** (P0)
**Status**: 🔴 OPEN

### Error Message
```
npm error code E404
npm error 404 Not Found - GET https://registry.npmjs.org/@types%2fjuice - Not found
npm error 404  '@types/juice@^0.0.36' is not in this registry.
```

### Root Cause Analysis

**Problem**:
1. Package specifies `@types/juice@^0.0.36` as a dev dependency
2. This @types package does not exist in npm registry
3. Version 0.0.36 has never existed on DefinitelyTyped
4. The `juice` library itself is at v10.0.0

**Why This Happened**:
- The juice library may have built-in TypeScript types (checking required)
- @types/juice may have been removed or never existed
- Incorrect assumption that all npm packages have @types packages
- Package.json may have been auto-generated with incorrect types

### Recommended Fix

**Option 1: Check if juice has built-in types and remove @types/juice (RECOMMENDED)**

Modern npm packages often include TypeScript types directly. The juice package v10.0.0 may have built-in types.

**Action**: Update `/home/user/esxi/crates/enterprise-notifications/ts/package.json`

**Remove**:
```json
"@types/juice": "^0.0.36",
```

**Impact**:
- ✅ Removes dependency on non-existent package
- ✅ If juice has built-in types, TypeScript will work automatically
- ⚠️ If juice doesn't have built-in types, will need custom type declarations

**How to verify juice has built-in types**:
```bash
# After installing juice, check for type definitions
npm install juice@^10.0.0
cat node_modules/juice/package.json | grep -A2 "types\|typings"
# Or check for index.d.ts
ls node_modules/juice/*.d.ts
```

**Confidence**: 🟢 High - Modern packages usually include types

---

**Option 2: Create custom type declarations**

If juice doesn't have built-in types:

**Action**: Create `/home/user/esxi/crates/enterprise-notifications/ts/src/types/juice.d.ts`

```typescript
declare module 'juice' {
  interface JuiceOptions {
    extraCss?: string;
    applyStyleTags?: boolean;
    removeStyleTags?: boolean;
    preserveMediaQueries?: boolean;
    preserveFontFaces?: boolean;
    preserveKeyFrames?: boolean;
    preservePseudos?: boolean;
    insertPreservedExtraCss?: boolean;
    applyWidthAttributes?: boolean;
    applyHeightAttributes?: boolean;
    applyAttributesTableElements?: boolean;
    webResources?: {
      images?: boolean | number;
      scripts?: boolean;
      links?: boolean;
      relativeTo?: string;
      rebaseRelativeTo?: string;
    };
  }

  function juice(html: string, options?: JuiceOptions): string;

  namespace juice {
    function juiceResources(
      html: string,
      options: JuiceOptions,
      callback: (err: Error | null, html: string) => void
    ): void;

    function inlineContent(
      html: string,
      css: string,
      options?: JuiceOptions
    ): string;
  }

  export = juice;
}
```

**Update package.json**: Remove the @types/juice line

**Impact**:
- ✅ Provides TypeScript types for juice
- ✅ Full control over type definitions
- ✅ Can be updated as needed
- ⚠️ Requires manual maintenance
- ⚠️ May not be 100% accurate

**Confidence**: 🟡 Medium - Requires manual type creation

---

**Option 3: Add type suppression (NOT RECOMMENDED)**

Only as last resort:

**Action**: Update tsconfig.json to skip type checking for juice

```json
{
  "compilerOptions": {
    "skipLibCheck": true
  }
}
```

Or use `// @ts-ignore` where juice is imported.

**Impact**:
- ✅ Allows compilation
- ❌ Loses type safety
- ❌ No autocomplete/IntelliSense
- ❌ Potential runtime errors

**Confidence**: 🔴 Low - Defeats purpose of TypeScript

---

### Implementation Steps

**Step 1**: Check if juice has built-in types
```bash
# Install just the juice package to check
cd /tmp
npm init -y
npm install juice@^10.0.0
cat node_modules/juice/package.json | grep "types\|typings"
ls node_modules/juice/*.d.ts 2>/dev/null
```

**Step 2a**: If juice HAS built-in types
- Remove `"@types/juice": "^0.0.36"` from enterprise-notifications/package.json line 105
- Save file
- Retry npm install

**Step 2b**: If juice DOES NOT have built-in types
- Remove `"@types/juice": "^0.0.36"` from package.json
- Create custom type declaration (Option 2 above)
- Add to tsconfig.json files array if needed
- Retry npm install

**Step 3**: Verify TypeScript compilation
```bash
npm run type-check --workspace=crates/enterprise-notifications/ts
```

**Step 4**: Verify juice can be imported
```bash
# Check that juice is imported correctly in source files
grep -r "from 'juice'" crates/enterprise-notifications/ts/src
# Ensure no TypeScript errors at import sites
```

---

### Validation Checklist

After applying fix:
- [ ] npm install completes successfully
- [ ] No 404 errors for @types packages
- [ ] TypeScript compilation works
- [ ] Juice imports have proper types (or no errors)
- [ ] Code completion works for juice API
- [ ] Build succeeds for enterprise-notifications

---

### Additional Investigation

**Check for similar issues with other @types packages**:
```bash
# List all @types dependencies across workspace
grep -h "@types/" crates/*/ts/package.json web/package.json | sort | uniq

# Verify each @types package exists
for pkg in $(grep -h "@types/" crates/*/ts/package.json | grep -o '@types/[^"]*' | cut -d: -f1 | sort -u); do
  echo "Checking $pkg..."
  npm view $pkg version 2>/dev/null || echo "  ❌ NOT FOUND: $pkg"
done
```

**Expected findings**:
- May find other missing @types packages
- Should verify all @types dependencies exist
- May need to remove or replace several

---

## Future Error Predictions

Based on the monorepo structure, likely upcoming errors:

### Predicted Error #3: Additional Missing Type Definitions
**Likelihood**: 🔴 High
**Location**: Various packages importing from other packages
**Fix**: Add proper type exports in package.json or verify @types packages exist

### Predicted Error #4: React Version Conflicts
**Likelihood**: 🟡 Medium
**Location**: Multiple packages may use different React versions
**Fix**: Standardize React version across all workspaces

### Predicted Error #5: Accessibility Package Cross-Dependencies
**Likelihood**: 🟡 Medium
**Location**: 13 accessibility packages importing each other
**Fix**: Ensure proper build order in turbo.json

### Predicted Error #6: Missing Dependencies
**Likelihood**: 🟢 Low-Medium
**Location**: Individual packages missing peer dependencies
**Fix**: Add missing dependencies to package.json files

---

## Error Pattern Analysis

### Common TypeScript Build Errors Expected:

1. **Dependency Conflicts** (like Error #1)
   - Peer dependency mismatches
   - Version incompatibilities
   - Transitive dependency issues

2. **Missing Type Definitions**
   - Missing @types/* packages
   - Missing type exports
   - Incorrect type paths in tsconfig

3. **Import/Export Issues**
   - Incorrect module paths
   - Missing exports from index.ts
   - Circular dependencies

4. **React Component Errors**
   - Incorrect prop types
   - Missing component exports
   - Hook usage violations

5. **Interface Implementation Issues**
   - Missing interface properties
   - Type incompatibilities
   - Generic type errors

---

## Dependency Standards (To Be Established)

### Recommended Standard Versions:
```json
{
  "react": "^18.2.0",
  "react-dom": "^18.2.0",
  "typescript": "^5.3.3",
  "rollup": "^4.9.6",
  "@rollup/plugin-terser": "^0.4.4",
  "vite": "^5.0.0"
}
```

### Build Tools:
```json
{
  "turbo": "^1.11.3",
  "@types/node": "^20.10.7",
  "@typescript-eslint/eslint-plugin": "^6.18.1",
  "@typescript-eslint/parser": "^6.18.1"
}
```

---

## Testing Strategy

### Post-Fix Testing:
1. ✅ Run `npm install` - verify no errors
2. ✅ Run `npm run typecheck` - verify TypeScript compilation
3. ✅ Run `npm run build` - verify all packages build
4. ✅ Run `npm run lint` - verify code quality
5. ✅ Run `npm test` - verify tests pass (if available)

### Regression Testing:
- Verify ui-components package builds
- Verify packages depending on ui-components
- Verify production bundle size
- Verify minification works

---

## Documentation Updates Needed

After fixes are applied:
1. Document standard dependency versions
2. Create dependency upgrade guide
3. Document build troubleshooting steps
4. Create onboarding guide for new developers

---

*This document is automatically maintained by the BUILD ERROR AGENT.*
*Last updated: 2026-01-01 00:38:22 UTC*
*Status: 2 errors documented, 0 errors fixed*
