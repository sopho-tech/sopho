import { ColumnDataType } from "src/constants/database_types";
import { IconType } from "src/components/design-system/datatypes";

export function getIconForDataType(dataType: ColumnDataType): IconType {
  switch (dataType) {
    case ColumnDataType.INT4:
    case ColumnDataType.INT8:
      return "hash";
    case ColumnDataType.TEXT:
    case ColumnDataType.VARCHAR:
      return "type";
    case ColumnDataType.TIMESTAMP:
    case ColumnDataType.TIMESTAMPTZ:
      return "calendar";
    case ColumnDataType.BOOL:
      return "check_square";
    case ColumnDataType.UUID:
      return "key";
    default:
      return "book";
  }
}

