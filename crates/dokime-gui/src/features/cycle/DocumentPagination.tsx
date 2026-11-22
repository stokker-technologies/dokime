/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Document Page Navigation
 */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
import { Button } from "@/components/ui/button";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

type Navigation = {
  hasPrevious: boolean;
  hasNext: boolean;
  fixed: boolean;
  busy: boolean;
  previous: () => void;
  next: () => void;
  latest: () => void;
};

/* === Components === */

export function DocumentPagination({ navigation }: { navigation: Navigation }) {
  if (!navigation.hasPrevious && !navigation.hasNext && !navigation.fixed) return null;
  return (
    <nav aria-label="Document Pages" className="flex items-center gap-2">
      <Button variant="outline" size="sm" disabled={!navigation.hasPrevious || navigation.busy} onClick={navigation.previous}>
        Previous Page
      </Button>
      <Button variant="outline" size="sm" disabled={!navigation.hasNext || navigation.busy} onClick={navigation.next}>
        Next Page
      </Button>
      {navigation.fixed ? (
        <Button variant="ghost" size="sm" onClick={navigation.latest}>
          Latest
        </Button>
      ) : null}
    </nav>
  );
}

/* === Hooks === */

/* === Public Functions === */

/* === Private Functions === */
