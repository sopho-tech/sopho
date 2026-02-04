# Refactor React Component

## Overview

You are a React, JS/TS, frontend, and Vite expert. Convert the existing React component into modular components, hooks, and functions. Follow design principles of React and standard ones like DRY, KISS, YAGNI, and SOLID.

## Steps

### 1. Analysis Phase

- **Review the component structure** - Identify responsibilities, side effects, and data flow
- **Check for logic optimization opportunities**
  - Simplify conditional logic (if-else, ternaries, switch statements)
  - Remove redundant state variables
  - Eliminate unnecessary re-renders
  - Identify derived state that can be computed
  - Look for memo/useMemo/useCallback opportunities
- **Identify code smells**
  - Large component files (>200-300 lines)
  - Multiple responsibilities in one component
  - Prop drilling beyond 2-3 levels
  - Duplicate logic across components
  - Inline complex logic in JSX
  - High cyclomatic complexity

### 2. Component Breakdown

- **Extract presentational components** - Pure UI components that receive props
- **Extract container components** - Components that handle business logic and state
- **Create compound components** - For related UI elements that work together
- **Identify reusable components** - Button, Input, Card, Modal patterns
- **Separate concerns** - UI vs logic vs data fetching

### 3. Custom Hooks Extraction

- **Extract stateful logic** - useState, useReducer patterns
- **Extract side effects** - API calls, subscriptions, event listeners
- **Create domain-specific hooks** - useAuth, useForm, useModal, etc.
- **Ensure hooks are composable** - Can be combined with other hooks

### 4. Utility Functions

- **Extract pure functions** - Data transformations, calculations, validations
- **Create helper functions** - Formatting, parsing, type guards
- **Move constants to separate files** - Magic numbers, configuration
- **Type utilities** - Shared TypeScript types and interfaces

### 5. State Management

- **Minimize state** - Remove unnecessary useState calls
- **Lift state appropriately** - Not too high, not too low
- **Use context wisely** - For truly global state, avoid prop drilling
- **Consider state managers** - For complex state (Zustand)
- **Prefer composition over props** - Use children pattern

### 6. Performance Optimization

- **Memoization**
  - `React.memo` for expensive components
  - `useMemo` for expensive computations
  - `useCallback` for stable function references
- **Code splitting** - React.lazy and Suspense for route-based splitting
- **Virtualization** - For long lists (react-window, react-virtuoso)
- **Debouncing/Throttling** - For event handlers

### 7. Code Quality Standards

- **Readability**
  - Descriptive variable and function names
  - Max function length: 20-30 lines
  - Max component length: 150-200 lines
- **Type Safety** (TypeScript)
  - Explicit prop types/interfaces
  - Avoid `any` types
  - Use discriminated unions for complex state

### 8. Best Practices Checklist

- ✅ Single Responsibility Principle - One component, one job
- ✅ Props are immutable - Never mutate props
- ✅ Key prop for lists - Stable, unique keys
- ✅ Avoid inline object/array creation in JSX - Causes re-renders
- ✅ Use destructuring - For props and state
- ✅ Early returns - For conditional rendering
- ✅ Controlled components - Prefer this over uncontrolled ones
- ✅ Accessibility - ARIA labels, semantic HTML, keyboard navigation
- ✅ Testing considerations - Components should be testable

### 9. Common Patterns to Apply

- **Render Props** - For sharing logic with flexible UI
- **Higher-Order Components** - For cross-cutting concerns (sparingly)
- **Custom Hooks** - Primary method for logic reuse
- **Composition** - Prefer over inheritance
- **Container/Presenter** - Separate logic from UI
- **Controlled vs Uncontrolled** - Choose appropriately for forms

### 10. Things to Avoid

- ❌ Prop drilling - Use context or state management
- ❌ Massive components - Break down into smaller pieces
- ❌ Logic in JSX - Extract to functions/hooks
- ❌ Inline styles - Use CSS modules or styled-components
- ❌ Index as key - Use stable unique identifiers
- ❌ Overuse of useEffect - Often a sign of poor design
- ❌ Premature optimization - Profile before optimizing
- ❌ Comments for obvious code - Code should be self-documenting

## Deliverables

- Refactored component files with clear separation of concerns
- Extracted custom hooks in dedicated files
- Utility functions in separate modules
- Updated imports and exports
- Consistent naming conventions throughout
