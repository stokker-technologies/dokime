// Dokime — SPDX-License-Identifier: Apache-2.0

export type Boolean = {
  id: string;
  title: string;
  description: string | null;
  default: boolean;
};

export type Choice = {
  value: string;
  title: string;
};

export type MultipleChoice = {
  id: string;
  title: string;
  description: string | null;
  default: string;
  choices: Choice[];
  /** The id of the plugin route that fills this dropdown at run time. */
  provider?: string | null;
};

export type Number = {
  id: string;
  title: string;
  description: string | null;
  default: number | null;
  min: number | null;
  max: number | null;
  step: number | null;
};

export type Path = {
  id: string;
  title: string;
  description: string | null;
  default: string;
  required?: boolean;
};

export type RuntimePluginSettings = {
  settings: Settings;
  values: { [key in string]: Value };
  secrets: SecretAvailability[];
};

export type SecretAvailability = { identifier: string; available: boolean };

export type SecretSetting = {
  id: string;
  title: string;
  description: string | null;
  required?: boolean;
};

export type Settings = {
  contributions?: SettingsEntry[];
};

export type SettingsEntry =
  | ({
      kind: "boolean";
    } & Boolean)
  | ({
      kind: "text_area";
    } & TextArea)
  | ({
      kind: "path";
    } & Path)
  | ({
      kind: "number";
    } & Number)
  | ({
      kind: "multiple_choice";
    } & MultipleChoice)
  | ({
      kind: "secret";
    } & SecretSetting);

export type TextArea = {
  id: string;
  title: string;
  description: string | null;
  default: string;
  required?: boolean;
};

export type Value = "Null" | boolean | number | null | string | Value[] | { [key in string]: Value };
