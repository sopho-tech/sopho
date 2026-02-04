import { Command } from "cmdk";
import { EntityType } from "src/api/search";
import { formatTimestamp } from "src/utils/timestamp_utils";
import { Badge } from "src/components/design-system";
import styles from "./CommandMenu.module.css";
import { CommandMenuSearchResultsProps } from "./CommandMenu.types";

const EntityNameMap: Record<EntityType, string> = {
  [EntityType.Canvas]: "Canvas",
  [EntityType.SqlCell]: "SQL Cell",
  [EntityType.ChartCell]: "Chart Cell",
};

const EntityBadgeVariantMap: Record<EntityType, "blue" | "green" | "yellow"> = {
  [EntityType.Canvas]: "blue",
  [EntityType.SqlCell]: "green",
  [EntityType.ChartCell]: "yellow",
};

export const CommandMenuSearchResults = ({
  data,
  onItemSelect,
}: CommandMenuSearchResultsProps) => {
  if (!data || data.length === 0) {
    return null;
  }

  return (
    <Command.Group heading="Search Results">
      {data.map((item) => {
        const entityName = EntityNameMap[item.entity_type];
        const badgeVariant = EntityBadgeVariantMap[item.entity_type];
        const formattedTimestamp = formatTimestamp(item.updated_at);

        return (
          <Command.Item
            key={item.id}
            value={item.id}
            onSelect={() => onItemSelect(item)}
          >
            <Badge variant={badgeVariant}>{entityName}</Badge>
            <span>{item.name}</span>
            <span className={styles.timestamp}>{formattedTimestamp}</span>
          </Command.Item>
        );
      })}
    </Command.Group>
  );
};
