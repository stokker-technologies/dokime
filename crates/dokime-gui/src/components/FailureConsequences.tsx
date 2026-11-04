/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Expandable Unavailable Operation Details
 */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

type FailureConsequencesProps = { items: readonly string[] };

/* === Components === */

export function FailureConsequences({ items }: FailureConsequencesProps) {
  if (!items.length) return null;
  return (
    <details
      className="text-sm [overflow-wrap:anywhere]"
      onClick={(event) => event.stopPropagation()}
      onKeyDown={(event) => event.stopPropagation()}
    >
      <summary className="cursor-pointer">
        {items.length} {items.length === 1 ? "operation" : "operations"} unavailable because startup failed
      </summary>
      <ul className="list-disc space-y-1 pl-5">
        {items.map((item) => (
          <li key={item}>{item}</li>
        ))}
      </ul>
    </details>
  );
}

/* === Hooks === */

/* === Public Functions === */

/* === Private Functions === */
