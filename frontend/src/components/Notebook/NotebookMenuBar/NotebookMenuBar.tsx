import { SophoMenuBar, MenuConfig } from "src/components/SophoMenuBar";
import NotebookMenuBarStyles from "src/components/Notebook/NotebookMenuBar/NotebookMenuBar.module.css";
import {
  CellType,
  CreateCellDto,
} from "src/components/Notebook/Cell";
import { useCreateCell } from "src/api/cell";
import { Icon } from "src/components/design-system/Icon";
import { useCanvasStore } from "src/components/Canvases/store";

type NotebookMenuBarProps = {
  style?: React.CSSProperties;
};

export function NotebookMenuBar({ style }: NotebookMenuBarProps) {
  const { activeNotebookId } = useCanvasStore();
  const createCellMutation = useCreateCell();

  function handleCreateNewCell(cellType: CellType) {
    const newCell: CreateCellDto = {
      notebook_id: activeNotebookId,
      name: null,
      content: null,
      cell_type: cellType,
    };
    createCellMutation.mutate(newCell);
  }

  const menus: MenuConfig[] = [
    {
      value: "new",
      icon: (
        <div className={NotebookMenuBarStyles.icon}>
          <Icon type="add" color="default" />
        </div>
      ),
      items: [
        {
          label: "Markdown Cell",
          shortcut: "⌘ M",
          onClick: () => handleCreateNewCell(CellType.MARKDOWN),
        },
        {
          label: "SQL Cell",
          shortcut: "⌘ Q",
          onClick: () => handleCreateNewCell(CellType.SQL),
        },
        {
          label: "Chart Cell",
          shortcut: "⌘ C",
          onClick: () => handleCreateNewCell(CellType.CHART),
        },
      ],
    },
    {
      value: "delete",
      icon: (
        <div className={NotebookMenuBarStyles.icon}>
          <Icon type="remove" color="default" />
        </div>
      ),
      items: [
        {
          label: "Delete Active Cell",
          shortcut: "⌘ D",
        },
        {
          label: "Delete All Cells",
          shortcut: "⇧ ⌘ D",
        },
      ],
    },
    {
      value: "move_up",
      icon: (
        <div className={NotebookMenuBarStyles.icon}>
          <Icon type="arrow_up" color="default" />
        </div>
      ),
      items: [
        {
          label: "Move Cell Up",
          shortcut: "⌘ M",
        },
        {
          label: "Move Cell Top",
          shortcut: "⇧ ⌘ M",
        },
      ],
    },
    {
      value: "move_down",
      icon: (
        <div className={NotebookMenuBarStyles.icon}>
          <Icon type="arrow_down" color="default" />
        </div>
      ),
      items: [
        {
          label: "Move Cell Down",
          shortcut: "⌘ T",
        },
        {
          label: "Move Cell Bottom",
          shortcut: "⇧ ⌘ T",
        },
      ],
    },
  ];

  return <SophoMenuBar menus={menus} style={style} />;
}
