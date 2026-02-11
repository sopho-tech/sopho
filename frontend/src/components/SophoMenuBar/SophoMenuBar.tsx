import * as Menubar from "@radix-ui/react-menubar";
import SophoMenuBarStyles from "src/components/SophoMenuBar/SophoMenuBar.module.css";

export type MenuItem = {
  label: string;
  shortcut?: string;
  onClick?: () => void;
  disabled?: boolean;
};

type SophoMenuBarRootProps = {
  children: React.ReactNode;
  loop?: boolean;
};

const SophoMenuBarRoot = ({ children, loop = true }: SophoMenuBarRootProps) => {
  return (
    <Menubar.Root className={SophoMenuBarStyles.root} loop={loop}>
      {children}
    </Menubar.Root>
  );
};

SophoMenuBarRoot.displayName = "SophoMenuBar";

type SophoMenuBarMenuProps = {
  value: string;
  icon: React.ReactNode;
  children: React.ReactNode;
};

const SophoMenuBarMenu = ({ value, icon, children }: SophoMenuBarMenuProps) => {
  return (
    <Menubar.Menu value={value}>
      <Menubar.Trigger className={SophoMenuBarStyles.trigger}>
        {icon}
      </Menubar.Trigger>
      <Menubar.Portal>
        <Menubar.Content
          className={SophoMenuBarStyles.content}
          align="start"
          sideOffset={5}
          alignOffset={-3}
        >
          {children}
        </Menubar.Content>
      </Menubar.Portal>
    </Menubar.Menu>
  );
};

SophoMenuBarMenu.displayName = "SophoMenuBar.Menu";

type SophoMenuBarItemProps = MenuItem;

const SophoMenuBarItem = ({
  label,
  shortcut,
  onClick,
  disabled,
}: SophoMenuBarItemProps) => {
  return (
    <Menubar.Item
      className={SophoMenuBarStyles.item}
      onClick={onClick}
      disabled={disabled}
    >
      {label}
      {shortcut && (
        <div className={SophoMenuBarStyles.rightSlot}>{shortcut}</div>
      )}
    </Menubar.Item>
  );
};

SophoMenuBarItem.displayName = "SophoMenuBar.Item";

export const SophoMenuBar = Object.assign(SophoMenuBarRoot, {
  Menu: SophoMenuBarMenu,
  Item: SophoMenuBarItem,
});
