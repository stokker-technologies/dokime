// Dokime — SPDX-License-Identifier: Apache-2.0
import { createElement } from "react";
import { createRoot } from "react-dom/client";
import { CartesianPlot } from "@dokime/plots/CartesianPlot";
import { DocumentViewRenderer } from "@dokime/document/components/document/DocumentViewRenderer";
import { documentCard, documentView } from "@dokime/document/lib/document-presentation";

const metadata = { identifier: "fixture", label: "Shared Presentation" };
const card = documentCard({ identifier: "fixture", label: "Throughput" }, "plots");
const document = documentView("default", metadata, [card], { nodes: [{ type: "card", card_id: card.card_id, layout: card.layout }] });

function PresentationExamples() {
  return (
    <DocumentViewRenderer
      document={document}
      renderCard={() => (
        <CartesianPlot
          title="Throughput"
          unit="Mbps"
          series={[
            {
              key: "throughput",
              label: "Mean",
              raw_points: 4,
              downsampled: false,
              points: [
                { x: 0, y: 10 },
                { x: 1, y: 12 },
                { x: 2, y: 11 },
                { x: 3, y: 15 },
              ],
            },
          ]}
        />
      )}
    />
  );
}

export function mountPresentationExamples(element: HTMLElement) {
  createRoot(element).render(createElement(PresentationExamples));
}
