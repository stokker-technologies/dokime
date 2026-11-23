// Dokime — SPDX-License-Identifier: Apache-2.0

export type SecretSetting = {
  id: string;
  title: string;
  description: string | null;
  required?: boolean;
};

export type Value = "Null" | boolean | number | null | string | Value[] | { [key in string]: Value };
