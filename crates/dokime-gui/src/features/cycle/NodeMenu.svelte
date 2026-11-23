<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { DropdownMenu } from "bits-ui";
  import { Archive, ArchiveRestore, Check, Ellipsis, LockKeyhole, Play, Plus, Repeat, RotateCcw, Settings2, Trash2 } from "lucide-svelte";
  import type { Snippet } from "svelte";
  import { buttonVariants } from "../../components/ui/button-variants";
  import { cn } from "../../lib/utils";
  import { nodeMenuGroups, type NodeMenuItem } from "./cycle-workspace-model";
  import type { CycleMenu, ExecutionMenu } from "./node-menu";

  let { menu, children }: { menu: CycleMenu | ExecutionMenu; children?: Snippet } = $props();
  const groups = $derived(nodeMenuGroups(menu.kind, menu.archive?.archived ?? false));
  const itemClass =
    "flex cursor-default items-center gap-2 rounded-lg px-2 py-1 text-sm outline-none data-highlighted:bg-accent data-disabled:pointer-events-none data-disabled:opacity-50 [&>svg]:size-4";

  function entry(item: NodeMenuItem, menu: CycleMenu | ExecutionMenu) {
    if (item === "Archive" || item === "Unarchive") {
      return {
        Icon: item === "Archive" ? Archive : ArchiveRestore,
        label: item,
        action: menu.archive?.onToggle,
        refusal: menu.archive?.refusal,
      };
    }
    if (item === "Settings") {
      return { Icon: Settings2, label: item, action: menu.onSettings };
    }
    if (menu.kind === "execution") {
      switch (item) {
        case "Start Execution":
          return { Icon: Play, label: item, action: menu.onStart, title: menu.startRefusal };
        case "Delete Case":
          return { Icon: Trash2, label: item, action: menu.onDeleteCase, title: menu.deleteRefusal, destructive: true };
        case "Re-Run":
          return { Icon: RotateCcw, label: item, action: menu.onRerun, refusal: menu.rerunRefusal };
      }
    } else {
      switch (item) {
        case "Seal Cycle":
          return { Icon: LockKeyhole, label: menu.sealFailed ? "Retry Seal" : item, action: menu.onSeal };
        case "Rerun Cycle":
          return { Icon: Repeat, label: item, action: menu.onRerunCycle };
        case "Add Cases":
          return { Icon: Plus, label: item, action: menu.onAddCases };
      }
    }
  }
</script>

<DropdownMenu.Root>
  <DropdownMenu.Trigger
    class={cn(buttonVariants({ variant: "outline", size: "icon" }), "size-[var(--bar-control-height)]")}
    data-slot="button"
    data-size="icon"
    data-variant="outline"
    aria-label={menu.kind === "cycle" ? "Cycle Actions" : "Execution Actions"}
    ><Ellipsis /></DropdownMenu.Trigger
  >
  <DropdownMenu.Portal>
    <DropdownMenu.Content align="end" class="z-50 w-52 rounded-lg bg-popover p-1 text-popover-foreground ring-1 ring-border">
      {@render children?.()}
      {#each groups as group, index (group[0])}
        {#if index > 0}
          <DropdownMenu.Separator class="-mx-1 my-1 h-px bg-border" />
        {/if}
        {#each group as item (item)}
          {@const command = entry(item, menu)}
          {#if item === "Follow Active" && menu.kind === "cycle"}
            <DropdownMenu.CheckboxItem
              class={itemClass}
              checked={menu.followActive}
              closeOnSelect={false}
              onCheckedChange={menu.onToggleFollowActive}
            >
              {#snippet children({
                checked,
              })}
                <span class="grid size-4 place-items-center"
                  >{#if checked}
                    <Check class="size-4" />
                  {/if}</span
                >
                Follow Active
              {/snippet}
            </DropdownMenu.CheckboxItem>
          {:else if command}
            <DropdownMenu.Item
              class={[itemClass, command.destructive && "text-destructive data-highlighted:bg-destructive/10"]}
              disabled={!command.action || Boolean(command.refusal)}
              onSelect={() => command.action?.()}
              title={command.refusal ?? command.title}
            >
              <command.Icon />
              {command.label}
            </DropdownMenu.Item>
          {/if}
        {/each}
      {/each}
    </DropdownMenu.Content>
  </DropdownMenu.Portal>
</DropdownMenu.Root>
