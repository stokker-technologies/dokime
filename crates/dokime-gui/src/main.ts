// Dokime — SPDX-License-Identifier: Apache-2.0
import { mount, unmount } from "svelte";
import DokimeToaster from "./components/DokimeToaster.svelte";
import ErrorCard from "./components/ErrorCard.svelte";
import { defaultBrand } from "./lib/brand";
import { failureOf } from "./lib/notification-details";
import "./desktop.css";

const target = document.getElementById("root");
if (!target) {
  throw new Error(`Missing ${defaultBrand.label} root element.`);
}

let active = true;
let dispose: (() => void) | undefined;
void import("./app/AppRoot.svelte")
  .then(({ default: AppRoot }) => {
    if (active) {
      const app = mount(AppRoot, { target });
      dispose = () => void unmount(app);
    }
  })
  .catch((error: unknown) => {
    if (active) {
      const toaster = mount(DokimeToaster, { target });
      const card = mount(ErrorCard, {
        target,
        props: { notificationContext: "background", failure: failureOf(error, "Application Could Not Start") },
      });
      dispose = () => {
        void unmount(card);
        void unmount(toaster);
      };
    }
  });

if (import.meta.hot) {
  import.meta.hot.dispose(() => {
    active = false;
    dispose?.();
  });
}
