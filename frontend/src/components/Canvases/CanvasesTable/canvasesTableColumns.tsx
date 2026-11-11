import { ColumnConfig } from "src/components/Table/Table";
import { CanvasDto } from "src/components/Canvases/dto";
import { formatDate, formatRelativeTime } from "src/utils/timestamp_utils";
import { Avatar, Flex, Text } from "src/components/design-system";
import { ActionButtons } from "src/components/ActionButtons";
import { describeActor } from "src/components/Attribution/describeActor";
import { CREATED_BY_LABEL } from "src/components/Attribution/constants";

const CANVAS_NAME_COLUMN_WIDTH = 260;
const CANVAS_CREATED_BY_COLUMN_WIDTH = 180;
const CANVAS_DATE_COLUMN_WIDTH = 150;
const CANVAS_ACTIONS_COLUMN_WIDTH = 56;

type CreateColumnsParams = {
  onViewClick: (id: string) => void;
  onEditClick: (id: string) => void;
  onDeleteClick: (id: string) => void;
};

export function createCanvasesTableColumns({
  onViewClick,
  onEditClick,
  onDeleteClick,
}: CreateColumnsParams): ColumnConfig<CanvasDto>[] {
  return [
    {
      key: "name",
      header: "Name",
      type: "accessor",
      accessor: "name",
      size: CANVAS_NAME_COLUMN_WIDTH,
      cell: (props) => (
        <Text fontSize="sm" truncate>
          {props.getValue() as string | null}
        </Text>
      ),
    },
    {
      key: "description",
      header: "Description",
      type: "accessor",
      accessor: "description",
      fill: true,
      cell: (props) => (
        <Text fontSize="sm" color="darkGrey" truncate>
          {props.getValue() as string | null}
        </Text>
      ),
    },
    {
      key: "created_by",
      header: CREATED_BY_LABEL,
      type: "display",
      size: CANVAS_CREATED_BY_COLUMN_WIDTH,
      cell: (props) => (
        <Flex direction="row" alignItems="center" gap="xs" overflow="hidden">
          <Avatar
            name={describeActor(props.row.original.created_by)}
            size="sm"
          />
          <Text fontSize="sm" color="darkGrey" truncate>
            {describeActor(props.row.original.created_by)}
          </Text>
        </Flex>
      ),
    },
    {
      key: "created_at",
      header: "Created",
      type: "accessor",
      accessor: "created_at",
      size: CANVAS_DATE_COLUMN_WIDTH,
      cell: (props) => (
        <Text fontSize="sm" color="darkGrey" truncate>
          {formatDate(props.getValue() as string | null)}
        </Text>
      ),
    },
    {
      key: "updated_at",
      header: "Last modified",
      type: "accessor",
      accessor: "updated_at",
      size: CANVAS_DATE_COLUMN_WIDTH,
      cell: (props) => (
        <Text fontSize="sm" color="darkGrey" truncate>
          {formatRelativeTime(props.getValue() as string | null)}
        </Text>
      ),
    },
    {
      key: "actions",
      header: "",
      type: "display",
      size: CANVAS_ACTIONS_COLUMN_WIDTH,
      cell: (props) => (
        <ActionButtons
          connectionId={props.row.original.id ?? ""}
          capabilities={props.row.original.capabilities}
          onViewClick={onViewClick}
          onEditClick={onEditClick}
          onDeleteClick={onDeleteClick}
        />
      ),
    },
  ];
}
