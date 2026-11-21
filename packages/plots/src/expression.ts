// Dokime — SPDX-License-Identifier: Apache-2.0
import { Aggregate, type DataExpression } from "@host/model";
import type { Wire } from "./lib/rpc/canonical";

const MAX_TRANSFORMS = 32;

/** Validate Canonical Expressions Without A Second Serialized Type Family. */
export function validatedExpression(value: Wire<DataExpression>, depth = 0): Wire<DataExpression> {
  if (depth > MAX_TRANSFORMS) throw new Error("Read Plot Expression: Reduce Its Transform Depth to 32");
  const kind = value.kind;
  switch (kind.case) {
    case "stream":
      if (!kind.value.identifier || !kind.value.field) throw new Error("Read Plot Expression: Supply Its Stream and Field");
      break;
    case "parameter":
    case "dimension":
    case "artifact":
      if (!kind.value) throw new Error("Read Plot Expression: Supply Its Source Identity");
      break;
    case "record":
      if (!kind.value.identifier) throw new Error("Read Plot Expression: Supply Its Record Identity");
      break;
    case "literal":
      break;
    case "aggregate": {
      const aggregate = kind.value;
      if (!aggregate.input) throw new Error("Read Plot Expression: Supply Its Aggregate Input");
      if (!aggregate.kind || !(aggregate.kind in Aggregate)) throw new Error("Read Plot Expression: Supply a Declared Aggregate");
      if (
        aggregate.kind === Aggregate.PERCENTILE &&
        (!Number.isInteger(aggregate.percentile) || aggregate.percentile < 0 || aggregate.percentile > 100)
      )
        throw new Error("Read Plot Expression: Use a Percentile From 0 to 100");
      validatedExpression(aggregate.input, depth + 1);
      break;
    }
    case "transform": {
      const transform = kind.value;
      if (!transform.input) throw new Error("Read Plot Expression: Supply Its Transform Input");
      const operation = transform.kind;
      if (!operation.case) throw new Error("Read Plot Expression: Supply Its Transform");
      if ((operation.case === "scale" || operation.case === "offset") && !Number.isFinite(operation.value))
        throw new Error("Read Plot Expression: Supply a Finite Transform Value");
      if ((operation.case === "detail" || operation.case === "timeBetween") && !operation.value)
        throw new Error("Read Plot Expression: Supply Its Transform Field");
      validatedExpression(transform.input, depth + 1);
      break;
    }
    default:
      throw new Error("Read Plot Expression: Supply Its Source");
  }
  return value;
}

export function expressionSource(value: Wire<DataExpression>): Wire<DataExpression> {
  let source = value;
  while (source.kind.case === "transform" || source.kind.case === "aggregate") {
    const input = source.kind.value.input;
    if (!input) break;
    source = input;
  }
  return source;
}
