import * as Menubar from "@radix-ui/react-menubar";
import SophoMenuBarStyles from "src/components/SophoMenuBar/SophoMenuBar.module.css";
import { ReactNode } from "react";

export type MenuItem = {
  label: string;
  shortcut?: string;
  onClick?: () => void;
  disabled?: boolean;
};

export type MenuConfig = {
  value: string;
  icon: ReactNode;
  items: MenuItem[];
};

type SophoMenuBarProps = {
  menus: MenuConfig[];
  loop?: boolean;
};

export function SophoMenuBar({ menus, loop = true }: SophoMenuBarProps) {
  return (
    <Menubar.Root className={SophoMenuBarStyles.root} loop={loop}>
      {menus.map((menu) => (
        <Menubar.Menu key={menu.value} value={menu.value}>
          <Menubar.Trigger className={SophoMenuBarStyles.trigger}>
            {menu.icon}
          </Menubar.Trigger>
          <Menubar.Portal>
            <Menubar.Content
              className={SophoMenuBarStyles.content}
              align="start"
              sideOffset={5}
              alignOffset={-3}
            >
              {menu.items.map((item, index) => (
                <Menubar.Item
                  key={index}
                  className={SophoMenuBarStyles.item}
                  onClick={item.onClick}
                  disabled={item.disabled}
                >
                  {item.label}
                  {item.shortcut && (
                    <div className={SophoMenuBarStyles.rightSlot}>
                      {item.shortcut}
                    </div>
                  )}
                </Menubar.Item>
              ))}
            </Menubar.Content>
          </Menubar.Portal>
        </Menubar.Menu>
      ))}
    </Menubar.Root>
  );
}
