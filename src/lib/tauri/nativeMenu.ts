import { LogicalPosition } from "@tauri-apps/api/dpi";
import { Menu, type MenuOptions } from "@tauri-apps/api/menu";

export type NativeMenuItem = {
  label: string;
  enabled?: boolean;
  danger?: boolean;
  dividerAfter?: boolean;
  action: () => void | Promise<void>;
};

export type NativeMenuPosition = {
  x: number;
  y: number;
};

export function canShowNativeMenu(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

export async function showNativeMenu(
  items: NativeMenuItem[],
  position?: NativeMenuPosition,
): Promise<void> {
  const menuItems: NonNullable<MenuOptions["items"]> = items.flatMap((item, index) => {
    const built: NonNullable<MenuOptions["items"]> = [{
      id: `native-menu-${index}`,
      text: item.label,
      enabled: item.enabled ?? true,
      action: () => {
        void item.action();
      },
    }];

    if (item.dividerAfter) {
      built.push({ item: "Separator" });
    }

    return built;
  });

  const menu = await Menu.new({
    items: menuItems,
  });

  try {
    await menu.popup(position ? new LogicalPosition(position.x, position.y) : undefined);
  } finally {
    await menu.close();
  }
}