# LifeOS domain glossary

## Area

A durable, broad life direction. Areas are independent canonical entities and
are not structural parents of Goals.

## Goal

A directional outcome with a horizon, status, and optional local start and
target dates. A Goal can semantically relate to an Area and to many projects,
skills, hobbies, habits, milestones, tasks, notes, and people; none of those
relationships is containment. A Goal's progress is a later deterministic
projection, not an authoritative mutable percentage in the first Goal slice.

## Project

A focused unit of work with optional local start/target dates, priority, and
one optional structural parent Project. Only active Projects can be selected
as a parent. Project nesting is intentionally distinct from Areas, Goals, and
semantic relationships; moving an existing Project and generic cycle-safe
hierarchy operations belong to a later M3 slice.

## Structural hierarchy

An ownership/parent-child relationship used only where the domain explicitly
permits nesting (for example, projects and tasks). It is distinct from semantic
relations and must remain acyclic.

## Semantic relation

A typed non-containment link between two existing entities. RelationGraph will
own validation and traversal when M3 introduces the generic relation spine.
