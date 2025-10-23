import { SophoMenuBar, MenuConfig } from "src/components/SophoMenuBar";
import NotebookMenuBarStyles from "src/components/Notebooks/Notebook/NotebookMenuBar/NotebookMenuBar.module.css";
import {
  CellType,
  CreateCellDto,
} from "src/components/Notebooks/Notebook/Cell";
import { useNotebookStore } from "src/components/Notebooks/store";
import { useCreateCell } from "src/api/cell";

export function NotebookMenuBar() {
  const { currentNotebookId } = useNotebookStore();
  const createCellMutation = useCreateCell();

  function onClickNewMarkdownCell() {
    console.log("onClickNewMarkdownCell");
  }

  function onClickNewSqlCell() {
    const newCell: CreateCellDto = {
      notebook_id: currentNotebookId,
      name: null,
      content: null,
      cell_type: CellType.SQL,
    };
    createCellMutation.mutate(newCell);
  }

  const menus: MenuConfig[] = [
    {
      value: "new",
      icon: (
        <span
          className={`material-symbols-outlined ${NotebookMenuBarStyles.icon}`}
        >
          add
        </span>
      ),
      items: [
        {
          label: "Markdown Cell",
          shortcut: "⌘ M",
          onClick: onClickNewMarkdownCell,
        },
        {
          label: "SQL Cell",
          shortcut: "⌘ Q",
          onClick: onClickNewSqlCell,
        },
        {
          label: "Chart Cell",
          shortcut: "⌘ C",
        },
      ],
    },
    {
      value: "delete",
      icon: (
        <span
          className={`material-symbols-outlined ${NotebookMenuBarStyles.icon}`}
        >
          remove
        </span>
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
        <span
          className={`material-symbols-outlined ${NotebookMenuBarStyles.icon}`}
        >
          arrow_upward
        </span>
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
        <span
          className={`material-symbols-outlined ${NotebookMenuBarStyles.icon}`}
        >
          arrow_downward
        </span>
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

  return <SophoMenuBar menus={menus} />;
}
