"use client";

import { Menu as MenuPrimitive } from "@base-ui/react/menu";
import { Menubar as MenubarPrimitive } from "@base-ui/react/menubar";
import { CheckIcon } from "lucide-react";
import type { ComponentProps } from "react";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuPortal,
  DropdownMenuRadioGroup,
  DropdownMenuSeparator,
  DropdownMenuShortcut,
  DropdownMenuSub,
  DropdownMenuSubContent,
  DropdownMenuSubTrigger,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { cn } from "@/lib/utils";

function Menubar({ className, ...props }: MenubarPrimitive.Props) {
  return (
    <MenubarPrimitive
      className={cn(
        "flex h-9 items-center rounded-lg border bg-background p-1",
        className
      )}
      data-slot="menubar"
      {...props}
    />
  );
}

function MenubarMenu({ ...props }: ComponentProps<typeof DropdownMenu>) {
  return <DropdownMenu data-slot="menubar-menu" {...props} />;
}

function MenubarGroup({ ...props }: ComponentProps<typeof DropdownMenuGroup>) {
  return <DropdownMenuGroup data-slot="menubar-group" {...props} />;
}

function MenubarPortal({
  ...props
}: ComponentProps<typeof DropdownMenuPortal>) {
  return <DropdownMenuPortal data-slot="menubar-portal" {...props} />;
}

function MenubarTrigger({
  className,
  ...props
}: ComponentProps<typeof DropdownMenuTrigger>) {
  return (
    <DropdownMenuTrigger
      className={cn(
        "flex select-none items-center rounded-[calc(var(--radius-md)-2px)] px-2 py-[calc(--spacing(0.85))] font-medium text-xs/relaxed outline-hidden hover:bg-muted aria-expanded:bg-muted",
        className
      )}
      data-slot="menubar-trigger"
      {...props}
    />
  );
}

function MenubarContent({
  className,
  align = "start",
  alignOffset = -4,
  sideOffset = 8,
  ...props
}: ComponentProps<typeof DropdownMenuContent>) {
  return (
    <DropdownMenuContent
      align={align}
      alignOffset={alignOffset}
      className={cn(
        "data-open:fade-in-0 data-open:zoom-in-95 data-[side=bottom]:slide-in-from-top-2 data-[side=left]:slide-in-from-right-2 data-[side=right]:slide-in-from-left-2 data-[side=top]:slide-in-from-bottom-2 data-[side=inline-start]:slide-in-from-end-2 data-[side=inline-end]:slide-in-from-start-2 min-w-32 rounded-lg bg-popover p-1 text-popover-foreground shadow-md ring-1 ring-foreground/10 duration-100 data-open:animate-in",
        className
      )}
      data-slot="menubar-content"
      sideOffset={sideOffset}
      {...props}
    />
  );
}

function MenubarItem({
  className,
  inset,
  variant = "default",
  ...props
}: ComponentProps<typeof DropdownMenuItem>) {
  return (
    <DropdownMenuItem
      className={cn(
        "group/menubar-item min-h-7 gap-2 rounded-md px-2 py-1 text-xs/relaxed focus:bg-accent focus:text-accent-foreground not-data-[variant=destructive]:focus:**:text-accent-foreground data-inset:ps-7.5 data-[variant=destructive]:text-destructive data-disabled:opacity-50 data-[variant=destructive]:focus:bg-destructive/10 data-[variant=destructive]:focus:text-destructive dark:data-[variant=destructive]:focus:bg-destructive/20 [&_svg:not([class*='size-'])]:size-3.5 data-[variant=destructive]:*:[svg]:text-destructive!",
        className
      )}
      data-inset={inset}
      data-slot="menubar-item"
      data-variant={variant}
      {...props}
    />
  );
}

function MenubarCheckboxItem({
  className,
  children,
  checked,
  inset,
  ...props
}: MenuPrimitive.CheckboxItem.Props & {
  inset?: boolean;
}) {
  return (
    <MenuPrimitive.CheckboxItem
      checked={checked}
      className={cn(
        "relative flex min-h-7 cursor-default select-none items-center gap-2 rounded-md py-1.5 ps-7.5 pe-2 text-xs outline-hidden focus:bg-accent focus:text-accent-foreground focus:**:text-accent-foreground data-disabled:pointer-events-none data-inset:ps-7.5 data-disabled:opacity-50 [&_svg]:pointer-events-none [&_svg]:shrink-0",
        className
      )}
      data-inset={inset}
      data-slot="menubar-checkbox-item"
      {...props}
    >
      <span className="pointer-events-none absolute start-2 flex size-4 items-center justify-center [&_svg:not([class*='size-'])]:size-4">
        <MenuPrimitive.CheckboxItemIndicator>
          <CheckIcon />
        </MenuPrimitive.CheckboxItemIndicator>
      </span>
      {children}
    </MenuPrimitive.CheckboxItem>
  );
}

function MenubarRadioGroup({
  ...props
}: ComponentProps<typeof DropdownMenuRadioGroup>) {
  return <DropdownMenuRadioGroup data-slot="menubar-radio-group" {...props} />;
}

function MenubarRadioItem({
  className,
  children,
  inset,
  ...props
}: MenuPrimitive.RadioItem.Props & {
  inset?: boolean;
}) {
  return (
    <MenuPrimitive.RadioItem
      className={cn(
        "relative flex min-h-7 cursor-default select-none items-center gap-2 rounded-md py-1.5 ps-7.5 pe-2 text-xs outline-hidden focus:bg-accent focus:text-accent-foreground focus:**:text-accent-foreground data-disabled:pointer-events-none data-inset:ps-7.5 data-disabled:opacity-50 [&_svg:not([class*='size-'])]:size-3.5 [&_svg]:pointer-events-none [&_svg]:shrink-0",
        className
      )}
      data-inset={inset}
      data-slot="menubar-radio-item"
      {...props}
    >
      <span className="pointer-events-none absolute start-2 flex size-4 items-center justify-center [&_svg:not([class*='size-'])]:size-4">
        <MenuPrimitive.RadioItemIndicator>
          <CheckIcon />
        </MenuPrimitive.RadioItemIndicator>
      </span>
      {children}
    </MenuPrimitive.RadioItem>
  );
}

function MenubarLabel({
  className,
  inset,
  ...props
}: ComponentProps<typeof DropdownMenuLabel> & {
  inset?: boolean;
}) {
  return (
    <DropdownMenuLabel
      className={cn(
        "px-2 py-1.5 text-muted-foreground text-xs data-inset:ps-7.5",
        className
      )}
      data-inset={inset}
      data-slot="menubar-label"
      {...props}
    />
  );
}

function MenubarSeparator({
  className,
  ...props
}: ComponentProps<typeof DropdownMenuSeparator>) {
  return (
    <DropdownMenuSeparator
      className={cn("-mx-1 my-1 h-px bg-border/50", className)}
      data-slot="menubar-separator"
      {...props}
    />
  );
}

function MenubarShortcut({
  className,
  ...props
}: ComponentProps<typeof DropdownMenuShortcut>) {
  return (
    <DropdownMenuShortcut
      className={cn(
        "ms-auto text-[0.625rem] text-muted-foreground tracking-widest group-focus/menubar-item:text-accent-foreground",
        className
      )}
      data-slot="menubar-shortcut"
      {...props}
    />
  );
}

function MenubarSub({ ...props }: ComponentProps<typeof DropdownMenuSub>) {
  return <DropdownMenuSub data-slot="menubar-sub" {...props} />;
}

function MenubarSubTrigger({
  className,
  inset,
  ...props
}: ComponentProps<typeof DropdownMenuSubTrigger> & {
  inset?: boolean;
}) {
  return (
    <DropdownMenuSubTrigger
      className={cn(
        "min-h-7 gap-2 rounded-md px-2 py-1 text-xs focus:bg-accent focus:text-accent-foreground not-data-[variant=destructive]:focus:**:text-accent-foreground data-open:bg-accent data-inset:ps-7.5 data-open:text-accent-foreground [&_svg:not([class*='size-'])]:size-3.5",
        className
      )}
      data-inset={inset}
      data-slot="menubar-sub-trigger"
      {...props}
    />
  );
}

function MenubarSubContent({
  className,
  ...props
}: ComponentProps<typeof DropdownMenuSubContent>) {
  return (
    <DropdownMenuSubContent
      className={cn(
        "data-closed:fade-out-0 data-open:fade-in-0 data-closed:zoom-out-95 data-open:zoom-in-95 data-[side=bottom]:slide-in-from-top-2 data-[side=left]:slide-in-from-right-2 data-[side=right]:slide-in-from-left-2 data-[side=top]:slide-in-from-bottom-2 min-w-32 rounded-lg bg-popover p-1 text-popover-foreground shadow-md ring-1 ring-foreground/10 duration-100 data-closed:animate-out data-open:animate-in",
        className
      )}
      data-slot="menubar-sub-content"
      {...props}
    />
  );
}

export {
  Menubar,
  MenubarPortal,
  MenubarMenu,
  MenubarTrigger,
  MenubarContent,
  MenubarGroup,
  MenubarSeparator,
  MenubarLabel,
  MenubarItem,
  MenubarShortcut,
  MenubarCheckboxItem,
  MenubarRadioGroup,
  MenubarRadioItem,
  MenubarSub,
  MenubarSubTrigger,
  MenubarSubContent,
};
