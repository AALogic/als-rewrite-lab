# Fixture Contract 004: ProjectDiscovery

Status: ready
Date: 2026-07-27

All fixtures are temporary synthetic directory trees.

Required shapes:

```text
Project/Ableton Project Info + Project/Set.als
Project/Ableton Project Info + Project/Sets/Nested.als
Project/Ableton Project Info + Project/Backup/Backup Set.als
Standalone/Set.als without marker
Outer/Ableton Project Info/Inner/Ableton Project Info/Set.als
Project/Ableton Project Info as symlink
selected ALS as symlink
case variant: ableton project info
```

The ALS fixture may contain arbitrary bytes because this module never opens
or parses it. Snapshot filesystem metadata before and after each observation
where safety is under test.
