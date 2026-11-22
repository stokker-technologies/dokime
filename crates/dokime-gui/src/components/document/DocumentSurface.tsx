// Dokime — SPDX-License-Identifier: Apache-2.0
import { ArtifactTree } from "../ArtifactTree";
import { MediaCaptureControls } from "../MediaCaptureControls";
import { MediaGallery } from "../MediaGallery";
import type { DocumentCardContentProps } from "./DocumentCardContent.svelte";
import { OperatorNotesContent } from "./OperatorNotesContent";

export function DocumentCardContent(props: DocumentCardContentProps) {
  if (props.renderMode === "report") {
    return null;
  }
  switch (props.card.kind) {
    case "media":
      return (
        <div className="grid gap-3">
          {props.captureScope ? <MediaCaptureControls scope={props.captureScope} /> : null}
          <MediaGallery artifacts={props.mediaArtifacts} emptyMessage="No media recorded." runDir={props.runDir} />
        </div>
      );
    case "artifacts":
      return <ArtifactTree artifacts={props.artifacts} emptyMessage="No artifacts recorded." runDir={props.runDir} />;
    case "notes":
      return <OperatorNotesContent notes={props.notes} scope={props.noteScope} />;
    default:
      return null;
  }
}
