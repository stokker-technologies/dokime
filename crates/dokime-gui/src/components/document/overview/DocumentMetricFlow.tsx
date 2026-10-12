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
import { type DragEvent, useEffect, useMemo, useState } from "react";
import { DocumentMetricGroup } from "./DocumentMetricGroup";
import {
  moveMetricGroup,
  type OverviewMetricGroup,
  orderMetricGroups,
  readMetricFlowOrder,
  removeMetricFlowOrder,
  writeMetricFlowOrder,
} from "./metric-model";

export function DocumentMetricFlow({
  groups,
  storageKey,
}: {
  groups: OverviewMetricGroup[];
  storageKey: string;
}) {
  const [savedOrder, setSavedOrder] = useState<string[]>(() => readMetricFlowOrder(storageKey));
  const [draggedGroupId, setDraggedGroupId] = useState<string | null>(null);
  const orderedGroups = useMemo(() => orderMetricGroups(groups, savedOrder), [groups, savedOrder]);
  const hasCustomOrder = savedOrder.length > 0;

  useEffect(() => {
    setSavedOrder(readMetricFlowOrder(storageKey));
  }, [storageKey]);

  function saveOrder(groupIds: string[]) {
    writeMetricFlowOrder(storageKey, groupIds);
    setSavedOrder(groupIds);
  }

  function handleDrop(event: DragEvent<HTMLElement>, targetGroupId: string) {
    const sourceGroupId = draggedGroupId || event.dataTransfer.getData("text/plain");
    if (!sourceGroupId || sourceGroupId === targetGroupId) {
      setDraggedGroupId(null);
      return;
    }

    event.preventDefault();
    const rect = event.currentTarget.getBoundingClientRect();
    const insertAfter = event.clientX > rect.left + rect.width / 2;
    saveOrder(
      moveMetricGroup(
        orderedGroups.map((group) => group.groupId),
        sourceGroupId,
        targetGroupId,
        insertAfter,
      ),
    );
    setDraggedGroupId(null);
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
            className="inline-flex size-8 items-center justify-center rounded-md border text-muted-foreground transition-colors hover:bg-muted hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring/40"
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
            onDragEnd={() => setDraggedGroupId(null)}
            onDragOver={(event) => {
              if (draggedGroupId && draggedGroupId !== group.groupId) {
                event.preventDefault();
                event.dataTransfer.dropEffect = "move";
              }
            }}
            onDragStart={(event) => {
              setDraggedGroupId(group.groupId);
              event.dataTransfer.effectAllowed = "move";
              event.dataTransfer.setData("text/plain", group.groupId);
            }}
            onDrop={(event) => handleDrop(event, group.groupId)}
          />
        ))}
      </div>
    </div>
  );
}
