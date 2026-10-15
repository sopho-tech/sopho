import { ColumnConfig } from "src/components/Table/Table";
import { CanvasDto } from "src/components/Canvases/dto";
import { formatTimestamp } from "src/utils/timestamp_utils";
import { ActionButtons } from "src/components/ActionButtons";
import { describeActor } from "src/components/Attribution/describeActor";
import { CREATED_BY_LABEL } from "src/components/Attribution/constants";

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
    },
    {
      key: "description",
      header: "Description",
      type: "accessor",
      accessor: "description",
    },
    {
      key: "created_by",
      header: CREATED_BY_LABEL,
      type: "display",
      cell: (props) => describeActor(props.row.original.created_by),
    },
    {
      key: "created_at",
      header: "Created On",
      type: "accessor",
      accessor: "created_at",
      cell: (props) => formatTimestamp(props.getValue() as string | null),
    },
    {
      key: "updated_at",
      header: "Last Modified",
      type: "accessor",
      accessor: "updated_at",
      cell: (props) => formatTimestamp(props.getValue() as string | null),
    },
    {
      key: "actions",
      header: "Actions",
      type: "display",
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
