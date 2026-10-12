/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { useNavigate } from "@tanstack/react-router";
import { MoreHorizontal } from "lucide-react";

/* === Local Imports === */

import { useNativeSessionController } from "@/app/native-session/NativeSessionProvider";
import { buttonVariants } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { cn } from "@/lib/utils";

/* === Components === */

/**
 * The global commands that used to live in the menu bar. Only commands with
 * real behavior appear here.
 */
export function ApplicationMenu() {
  const navigate = useNavigate();
  const session = useNativeSessionController();

  return (
    <DropdownMenu>
      <DropdownMenuTrigger
        aria-label="Application"
        className={cn(
          buttonVariants({ size: "sm", variant: "ghost" }),
          "h-[var(--control-height)] gap-1.5",
        )}
        type="button"
      >
        <MoreHorizontal aria-hidden />
        Application
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end" className="w-56">
        <DropdownMenuItem onSelect={() => void navigate({ to: "/settings" })}>
          Settings
        </DropdownMenuItem>
        <DropdownMenuItem
          onSelect={() =>
            void navigate({ params: { section: "diagnostics" }, to: "/settings/$section" })
          }
        >
          Diagnostics
        </DropdownMenuItem>
        <DropdownMenuItem
          onSelect={() => void navigate({ params: { section: "about" }, to: "/settings/$section" })}
        >
          About
        </DropdownMenuItem>
        <DropdownMenuSeparator />
        <DropdownMenuItem onSelect={() => void session.disconnect()}>Sign Out</DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
