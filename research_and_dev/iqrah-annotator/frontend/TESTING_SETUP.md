# Testing Setup Instructions

## Prerequisites

Install testing dependencies:

```bash
npm install -D vitest @vitest/ui @testing-library/react @testing-library/jest-dom @testing-library/user-event jsdom
npm install -D @playwright/test
npm install -D @vitest/coverage-v8
```

## Package.json Scripts

Add these scripts to your `package.json`:

```json
{
  "scripts": {
    "test": "vitest",
    "test:ui": "vitest --ui",
    "test:run": "vitest run",
    "test:coverage": "vitest run --coverage",
    "test:watch": "vitest watch",
    "test:e2e": "playwright test",
    "test:e2e:ui": "playwright test --ui",
    "test:e2e:headed": "playwright test --headed",
    "test:e2e:debug": "playwright test --debug",
    "test:all": "npm run test:run && npm run test:e2e"
  }
}
```

## Running Tests

### Unit Tests (Vitest)

```bash
# Run all unit tests
npm run test

# Run with UI
npm run test:ui

# Run once (CI mode)
npm run test:run

# Run with coverage
npm run test:coverage

# Watch mode
npm run test:watch
```

### E2E Tests (Playwright)

```bash
# Run all E2E tests
npm run test:e2e

# Run with UI
npm run test:e2e:ui

# Run in headed mode (see browser)
npm run test:e2e:headed

# Debug mode
npm run test:e2e:debug
```

### Run Everything

```bash
npm run test:all
```

## Test Coverage

Coverage reports are generated in `./coverage` directory:
- `coverage/index.html` - HTML report (open in browser)
- `coverage/lcov.info` - LCOV format (for CI integration)
- `coverage/coverage-final.json` - JSON format

### Coverage Thresholds

Current thresholds (in `vitest.config.ts`):
- Lines: 70%
- Functions: 70%
- Branches: 60%
- Statements: 70%

## Test File Locations

```
frontend/
├── src/
│   ├── components/
│   │   └── LoadingOverlay.test.tsx
│   ├── constants/
│   │   └── errorMessages.test.ts ✅
│   ├── hooks/
│   │   ├── useAutoSave.test.ts ✅
│   │   └── useKeyboardShortcuts.test.ts ✅
│   ├── lib/
│   │   └── utils.test.ts ✅
│   ├── utils/
│   │   └── exportValidation.test.ts ✅
│   ├── store/
│   │   └── wizardStore.test.ts
│   ├── annotation/
│   │   └── manager.test.ts
│   └── test/
│       └── setup.ts ✅
├── e2e/
│   ├── wizard-flow.spec.ts
│   ├── annotation.spec.ts
│   └── session-recovery.spec.ts
├── vitest.config.ts ✅
└── playwright.config.ts
```

## Writing Tests

### Unit Test Example

```typescript
import { describe, it, expect } from 'vitest';
import { myFunction } from './myModule';

describe('myFunction', () => {
  it('should do something', () => {
    const result = myFunction(input);
    expect(result).toBe(expected);
  });
});
```

### Component Test Example

```typescript
import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import MyComponent from './MyComponent';

describe('MyComponent', () => {
  it('should render correctly', () => {
    render(<MyComponent />);
    expect(screen.getByText('Hello')).toBeInTheDocument();
  });
});
```

### E2E Test Example

```typescript
import { test, expect } from '@playwright/test';

test('complete wizard flow', async ({ page }) => {
  await page.goto('http://localhost:5173');
  await page.click('text=Start Wizard');
  // ... test steps
});
```

## CI Integration

### GitHub Actions Example

```yaml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions/setup-node@v3
        with:
          node-version: '18'
      - run: npm ci
      - run: npm run test:run
      - run: npm run test:e2e
      - uses: codecov/codecov-action@v3
        with:
          files: ./coverage/lcov.info
```

## Debugging Tests

### Vitest

```bash
# Run specific test file
npm run test -- errorMessages.test.ts

# Run tests matching pattern
npm run test -- --grep "validation"

# Run with debug output
DEBUG=* npm run test
```

### Playwright

```bash
# Run specific test
npm run test:e2e -- wizard-flow.spec.ts

# Debug mode (opens inspector)
npm run test:e2e:debug

# Generate test code
npx playwright codegen http://localhost:5173
```

## Best Practices

1. **Unit Tests**
   - Test pure functions and utilities first
   - Mock external dependencies
   - Use descriptive test names
   - Test edge cases and error handling

2. **Component Tests**
   - Test user interactions
   - Test rendering logic
   - Mock hooks and context
   - Use Testing Library queries

3. **E2E Tests**
   - Test critical user journeys
   - Use data-testid for stable selectors
   - Keep tests independent
   - Clean up after tests

4. **Coverage**
   - Aim for 70%+ coverage
   - Focus on critical paths
   - Don't obsess over 100%
   - Use coverage gaps to find missing tests

## Troubleshooting

### "Cannot find module" errors
- Check import paths
- Ensure test setup is correct
- Verify vitest.config.ts aliases

### Tests timeout
- Increase timeout in vitest.config.ts
- Check for infinite loops
- Use `waitFor` for async operations

### E2E tests fail locally
- Ensure dev server is running
- Check port numbers match
- Install Playwright browsers: `npx playwright install`

### Coverage not updating
- Clear coverage cache: `rm -rf coverage`
- Run with --coverage flag
- Check .gitignore doesn't exclude coverage

---

**Testing infrastructure is ready!** 🎉

Start with:
```bash
npm run test:coverage
```
