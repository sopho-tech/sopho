import NotebookMenuBarStyles from "src/components/Notebooks/Notebook/NotebookMenuBar/NotebookMenuBar.module.css";
import * as Menubar from "@radix-ui/react-menubar";
import {
  CellType,
  CreateCellDto,
} from "src/components/Notebooks/Notebook/Cell";
import { useNotebookStore } from "src/components/Notebooks/store";
import { useCreateCell } from "src/api/cell";

export function NotebookMenuBar() {
  const { currentNotebookId } = useNotebookStore();
  const createCellMutation = useCreateCell();

  async function onClickNewMarkdownCell() {
    console.log("onClickNewMarkdownCell");
  }

  async function onClickNewSqlCell() {
    const newCell: CreateCellDto = {
      notebook_id: currentNotebookId,
      name: null,
      content: null,
      cell_type: CellType.SQL,
    };
    createCellMutation.mutate(newCell);
  }

  return (
    <Menubar.Root className={NotebookMenuBarStyles.root} loop>
      <Menubar.Menu value="new">
        <Menubar.Trigger className={NotebookMenuBarStyles.trigger}>
          <span
            className={`material-symbols-outlined ${NotebookMenuBarStyles["addButton"]}`}
          >
            add
          </span>
        </Menubar.Trigger>
        <Menubar.Portal>
          <Menubar.Content
            className={NotebookMenuBarStyles.content}
            align="start"
            sideOffset={5}
            alignOffset={-3}
          >
            <Menubar.Item
              className={NotebookMenuBarStyles.item}
              onClick={onClickNewMarkdownCell}
            >
              Markdown Cell{" "}
              <div className={NotebookMenuBarStyles.rightSlot}>⌘ M</div>
            </Menubar.Item>
            <Menubar.Item
              className={NotebookMenuBarStyles.item}
              onClick={onClickNewSqlCell}
            >
              SQL Cell{" "}
              <div className={NotebookMenuBarStyles.rightSlot}>⌘ Q</div>
            </Menubar.Item>
            <Menubar.Item className={NotebookMenuBarStyles.item}>
              Chart Cell{" "}
              <div className={NotebookMenuBarStyles.rightSlot}>⌘ C</div>
            </Menubar.Item>
          </Menubar.Content>
        </Menubar.Portal>
      </Menubar.Menu>
      <Menubar.Menu value="delete">
        <Menubar.Trigger className={NotebookMenuBarStyles.trigger}>
          <span
            className={`material-symbols-outlined ${NotebookMenuBarStyles["addButton"]}`}
          >
            remove
          </span>
        </Menubar.Trigger>
        <Menubar.Portal>
          <Menubar.Content
            className={NotebookMenuBarStyles.content}
            align="start"
            sideOffset={5}
            alignOffset={-3}
          >
            <Menubar.Item className={NotebookMenuBarStyles.item}>
              Delete Active Cell{" "}
              <div className={NotebookMenuBarStyles.rightSlot}>⌘ D</div>
            </Menubar.Item>
            <Menubar.Item className={NotebookMenuBarStyles.item}>
              Delete All Cells{" "}
              <div className={NotebookMenuBarStyles.rightSlot}>⇧ ⌘ D</div>
            </Menubar.Item>
          </Menubar.Content>
        </Menubar.Portal>
      </Menubar.Menu>
      <Menubar.Menu value="move_up">
        <Menubar.Trigger className={NotebookMenuBarStyles.trigger}>
          <span
            className={`material-symbols-outlined ${NotebookMenuBarStyles["addButton"]}`}
          >
            arrow_upward
          </span>
        </Menubar.Trigger>
        <Menubar.Portal>
          <Menubar.Content
            className={NotebookMenuBarStyles.content}
            align="start"
            sideOffset={5}
            alignOffset={-3}
          >
            <Menubar.Item className={NotebookMenuBarStyles.item}>
              Move Cell Up{" "}
              <div className={NotebookMenuBarStyles.rightSlot}>⌘ M</div>
            </Menubar.Item>
            <Menubar.Item className={NotebookMenuBarStyles.item}>
              Move Cell Top{" "}
              <div className={NotebookMenuBarStyles.rightSlot}>⇧ ⌘ M</div>
            </Menubar.Item>
          </Menubar.Content>
        </Menubar.Portal>
      </Menubar.Menu>
      <Menubar.Menu value="move_down">
        <Menubar.Trigger className={NotebookMenuBarStyles.trigger}>
          <span
            className={`material-symbols-outlined ${NotebookMenuBarStyles["addButton"]}`}
          >
            arrow_downward
          </span>
        </Menubar.Trigger>
        <Menubar.Portal>
          <Menubar.Content
            className={NotebookMenuBarStyles.content}
            align="start"
            sideOffset={5}
            alignOffset={-3}
          >
            <Menubar.Item className={NotebookMenuBarStyles.item}>
              Move Cell Down{" "}
              <div className={NotebookMenuBarStyles.rightSlot}>⌘ T</div>
            </Menubar.Item>
            <Menubar.Item className={NotebookMenuBarStyles.item}>
              Move Cell Bottom{" "}
              <div className={NotebookMenuBarStyles.rightSlot}>⇧ ⌘ T</div>
            </Menubar.Item>
          </Menubar.Content>
        </Menubar.Portal>
      </Menubar.Menu>
    </Menubar.Root>
  );
}
