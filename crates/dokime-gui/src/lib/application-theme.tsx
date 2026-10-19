/* ***
 * Dokime
 * SPDX-License-Identifier: Apache-2.0
 * ***
 */
/* === External Imports === */
import { useTheme } from "next-themes";
import { createContext, type ReactNode, useContext } from "react";

/* === Types === */
export type ApplicationTheme = {
  saving?: boolean;
  theme?: string;
  resolvedTheme?: string;
  forcedTheme?: string;
  setTheme: (value: string) => void;
};
const NativeTheme = createContext<ApplicationTheme | null>(null);

/** Desktop settings own preferences; next-themes remains the browser adapter. */
export function useApplicationTheme(): ApplicationTheme {
  const native = useContext(NativeTheme);
  const browser = useTheme();
  return native ?? browser;
}

/* === Components === */
export function ApplicationThemeProvider({ children, value }: { children: ReactNode; value: ApplicationTheme }) {
  return <NativeTheme.Provider value={value}>{children}</NativeTheme.Provider>;
}
