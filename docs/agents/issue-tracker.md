# Issue tracker: Linear

Linear is the canonical source for milestones, issues, dependencies, and work
status. The repository's `docs/` directory is the behavioural source of truth.

Use the existing Linear integration for all Linear operations. Do not substitute
GitHub Issues merely because this repository has a GitHub remote.

## Reading work

- Resolve milestones and issues by their exact Linear name or identifier.
- Before implementing a milestone, fetch its description and every included
  issue.
- Read each issue's description, comments, acceptance criteria, status, and
  blocking relationships.
- Implement issues in dependency order.
- If no dependencies are recorded, use their Linear ordering.
- When Linear and `docs/` disagree about expected behaviour, stop and report
  the conflict instead of silently choosing one.

## Updating work

Linear mutations are authorized only when the user explicitly requests them.
A request to implement a milestone and update Linear authorizes updates only to
that milestone and its included issues.

During an authorized implementation:

- Move an issue to In Progress immediately before beginning its implementation.
- Move an issue to Done only after:
  - its acceptance criteria are satisfied;
  - relevant tests pass;
  - formatting and static checks pass;
  - its implementation has been committed.
- Add a completion comment containing:
  - a concise implementation summary;
  - tests and checks executed;
  - the resulting commit SHA;
  - any intentional deviations or follow-up work.
- If genuinely blocked, leave the issue open and comment with:
  - the blocker;
  - what was attempted;
  - the decision or access required to continue.
- Do not mark a milestone complete until every included issue is complete.

## Publishing work

When a skill says "publish to the issue tracker", create Linear issues only when
the user has explicitly authorized publication.

Preserve Linear identifiers in repository documentation, commit messages, and
pull requests.

## Safety

- Do not modify issues outside the requested milestone.
- Do not delete existing issues, milestones, projects, or comments.
- Do not rewrite issue descriptions unless explicitly requested.
- Do not begin a blocked issue.
- Do not create duplicate issues for work that already exists in Linear.
