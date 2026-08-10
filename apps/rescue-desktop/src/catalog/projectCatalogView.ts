import type {
  ProjectCatalogListResult,
  ProjectListGroup,
  ProjectListItem,
} from "./contracts";

export type VisibleProjectGroup = {
  group: ProjectListGroup;
  items: ProjectListItem[];
};

export function visibleProjectGroups(
  catalog: ProjectCatalogListResult,
  query: string,
): VisibleProjectGroup[] {
  const normalizedQuery = query.trim().toLocaleLowerCase();
  const itemsById = new Map(catalog.items.map((item) => [item.item_id, item]));
  return catalog.groups
    .map((group) => ({
      group,
      items: group.item_ids
        .map((id) => itemsById.get(id))
        .filter((item): item is ProjectListItem => Boolean(item))
        .filter((item) => matchesQuery(group, item, normalizedQuery)),
    }))
    .filter(({ items }) => items.length > 0);
}

export function selectedCatalogIds(
  catalog: ProjectCatalogListResult,
  selectedIds: ReadonlySet<string>,
): string[] {
  const itemsById = new Map(catalog.items.map((item) => [item.item_id, item]));
  return catalog.groups.flatMap((group) =>
    group.item_ids
      .map((id) => itemsById.get(id))
      .filter((item): item is ProjectListItem => Boolean(item))
      .filter((item) => selectedIds.has(item.live_set_id))
      .map((item) => item.live_set_id),
  );
}

export function formatProjectDate(timestamp: number | null): string {
  if (timestamp === null) return "Data niedostępna";
  return new Intl.DateTimeFormat("pl-PL", {
    dateStyle: "medium",
    timeStyle: "short",
  }).format(new Date(timestamp));
}

export function formatFileSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

function matchesQuery(group: ProjectListGroup, item: ProjectListItem, query: string): boolean {
  if (!query) return true;
  return [group.display_name, item.display_name, item.project_display_name ?? ""]
    .some((value) => value.toLocaleLowerCase().includes(query));
}
