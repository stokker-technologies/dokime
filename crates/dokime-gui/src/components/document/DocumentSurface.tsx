/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

import { ArtifactTree } from "@/components/ArtifactTree";
import { MediaCaptureControls } from "@/components/MediaCaptureControls";
import { MediaGallery } from "@/components/MediaGallery";
import { type DocumentCardContentProps, DocumentCardContentView } from "./DocumentSurfaceView";
import { OperatorNotesContent } from "./OperatorNotesContent";

export function DocumentCardContent(props: DocumentCardContentProps) {
  if (props.renderMode === "report") return <DocumentCardContentView {...props} />;
  return (
    <DocumentCardContentView
      {...props}
      captureControls={props.captureScope ? <MediaCaptureControls scope={props.captureScope} /> : null}
      mediaContent={<MediaGallery artifacts={props.mediaArtifacts} emptyMessage="No media recorded." runDir={props.runDir} />}
      artifactsContent={<ArtifactTree artifacts={props.artifacts} emptyMessage="No artifacts recorded." runDir={props.runDir} />}
      notesContent={<OperatorNotesContent notes={props.notes} scope={props.noteScope} />}
    />
  );
}
