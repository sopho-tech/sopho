import { ColumnConfig } from "src/components/SophoTable/SophoTable";
import { CanvasDto } from "src/components/Canvases/dto";
import { formatTimestamp } from "src/utils/timestamp_utils";
import { ActionButtons } from "src/components/ActionButtons";

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
      key: "created_at",
      header: "Created On",
      type: "accessor",
      accessor: "created_at",
      cell: (props) => formatTimestamp(props.getValue()),
    },
    {
      key: "updated_at",
      header: "Last Modified",
      type: "accessor",
      accessor: "updated_at",
      cell: (props) => formatTimestamp(props.getValue()),
    },
    {
      key: "actions",
      header: "Actions",
      type: "display",
      cell: (props) => (
        <ActionButtons
          connectionId={props.row.original.id}
          onViewClick={onViewClick}
          onEditClick={onEditClick}
          onDeleteClick={onDeleteClick}
        />
      ),
    },
  ];
}
