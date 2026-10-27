/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

import { RotateCcw } from "lucide-react";
import { type PointerEvent, useEffect, useMemo, useState } from "react";
import { DocumentMetricGroup } from "./DocumentMetricGroup";
import {
  moveMetricGroup,
  type OverviewMetricGroup,
  orderMetricGroups,
  readMetricFlowOrder,
  removeMetricFlowOrder,
  writeMetricFlowOrder,
} from "./metric-model";

export function DocumentMetricFlow({ groups, storageKey }: { groups: OverviewMetricGroup[]; storageKey: string }) {
  const [savedOrder, setSavedOrder] = useState<string[]>(() => readMetricFlowOrder(storageKey));
  const [draggedGroupId, setDraggedGroupId] = useState<string | null>(null);
  const [targetGroupId, setTargetGroupId] = useState<string | null>(null);
  const orderedGroups = useMemo(() => orderMetricGroups(groups, savedOrder), [groups, savedOrder]);
  const hasCustomOrder = savedOrder.length > 0;

  useEffect(() => {
    setSavedOrder(readMetricFlowOrder(storageKey));
  }, [storageKey]);

  function saveOrder(groupIds: string[]) {
    writeMetricFlowOrder(storageKey, groupIds);
    setSavedOrder(groupIds);
  }

  // Pointer events rather than HTML5 drag and drop: the shell takes native
  // file drops, which keeps HTML5 drag events from reaching the page.
  function startDrag(event: PointerEvent<HTMLButtonElement>, sourceGroupId: string) {
    if (event.button !== 0) return;
    event.preventDefault();
    const grip = event.currentTarget;
    grip.setPointerCapture(event.pointerId);
    setDraggedGroupId(sourceGroupId);
    const order = orderedGroups.map((group) => group.groupId);
    const move = (moved: globalThis.PointerEvent) => setTargetGroupId(dropTarget(moved, sourceGroupId)?.groupId ?? null);
    const finish = (ended: globalThis.PointerEvent) => {
      grip.removeEventListener("pointermove", move);
      grip.removeEventListener("pointerup", finish);
      grip.removeEventListener("pointercancel", finish);
      const target = ended.type === "pointerup" ? dropTarget(ended, sourceGroupId) : null;
      if (target) saveOrder(moveMetricGroup(order, sourceGroupId, target.groupId, target.insertAfter));
      setDraggedGroupId(null);
      setTargetGroupId(null);
    };
    grip.addEventListener("pointermove", move);
    grip.addEventListener("pointerup", finish);
    grip.addEventListener("pointercancel", finish);
  }

  function resetOrder() {
    removeMetricFlowOrder(storageKey);
    setSavedOrder([]);
  }

  return (
    <div className="grid gap-3" data-testid="metric-flow">
      {hasCustomOrder ? (
        <div className="flex justify-end">
          <button
            aria-label="Reset Metric Group Order"
            className="inline-flex size-8 items-center justify-center rounded-lg border text-muted-foreground transition-colors hover:bg-muted hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring/40"
            title="Reset Metric Group Order"
            type="button"
            onClick={resetOrder}
          >
            <RotateCcw size={14} />
          </button>
        </div>
      ) : null}
      <div className="flex flex-wrap items-start gap-3">
        {orderedGroups.map((group) => (
          <DocumentMetricGroup
            groupId={group.groupId}
            isDragging={draggedGroupId === group.groupId}
            key={group.groupId}
            metrics={group.metrics}
            title={group.title}
            isTarget={targetGroupId === group.groupId}
            onGripPointerDown={(event) => startDrag(event, group.groupId)}
          />
        ))}
      </div>
    </div>
  );
}

/** The group under the pointer, other than the dragged one, and which half of it the pointer is over. */
function dropTarget(event: globalThis.PointerEvent, sourceGroupId: string): { groupId: string; insertAfter: boolean } | null {
  const element = document.elementFromPoint(event.clientX, event.clientY)?.closest<HTMLElement>("[data-metric-group-id]");
  const groupId = element?.dataset.metricGroupId;
  if (!element || !groupId || groupId === sourceGroupId) return null;
  const bounds = element.getBoundingClientRect();
  return { groupId, insertAfter: event.clientX > bounds.left + bounds.width / 2 };
}
