# Create PR

## Overview

Create a well-structured pull request with proper description, labels, and reviewers. Follow conventional commits standard for commit messages. Add a Refs to the Github Issue for feature commits. The issue id can be found from the branch name.

## Steps

1. **Prepare branch**

   - Ensure all changes are committed
   - Push branch to remote
   - Verify branch is up to date with main

2. **Write PR description**

   - Summarize changes clearly
   - Include context and motivation
   - List any breaking changes
   - Add screenshots if UI changes

3. **Set up PR**
   - Create PR with descriptive title
   - Add appropriate labels
   - Assign reviewers
   - Link related issues

## Example commit message:

```
feat(frontend): dashboard enhancements

- Refactor DataTable into modular components
- Enable draggable rows from DataTable to react-grid-layout
- Add refresh button for dashboard charts
- Add delete button for dashboard charts
- Introduce toolbar component and core building blocks in design system

Refs: #50
```

## Notes

- Never ever add Co-authored-by: Cursor <cursoragent@cursor.com> in the commit or the PR description
