import { Command } from "cmdk";
import React from "react";
import { CommandMenuItem } from "./CommandMenuItem";
import { CommandMenuHomePageProps } from "./CommandMenu.types";

export const CommandMenuHomePage = ({
  onSelect,
  onCreateCanvas,
}: CommandMenuHomePageProps) => {
  return (
    <React.Fragment>
      <Command.Group heading="Canvases">
        <CommandMenuItem
          value="Search Canvases"
          label="Search Canvases"
          iconType="search"
          onSelect={() => onSelect("canvases")}
        />
        <CommandMenuItem
          value="Create Canvas"
          label="Create Canvas"
          iconType="add"
          onSelect={onCreateCanvas}
        />
      </Command.Group>
      <Command.Separator />
      <Command.Group heading="Chart Cells">
        <CommandMenuItem
          value="Search Chart Cells"
          label="Search Chart Cells"
          iconType="search"
          onSelect={() => onSelect("chart_cells")}
        />
      </Command.Group>
      <Command.Group heading="SQL Cells">
        <CommandMenuItem
          value="Search SQL Cells"
          label="Search SQL Cells"
          iconType="search"
          onSelect={() => onSelect("sql_cells")}
        />
      </Command.Group>
    </React.Fragment>
  );
};
