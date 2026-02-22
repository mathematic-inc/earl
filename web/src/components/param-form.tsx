import { useCallback, useEffect, useId, useRef } from "react";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { Textarea } from "@/components/ui/textarea";
import type { ParamSpec } from "@/lib/types";
import { cn } from "@/lib/utils";

export interface ParamFormProps {
  autoFocus?: boolean;
  errors: Record<string, string>;
  onChange: (values: Record<string, unknown>) => void;
  onErrorsChange: (errors: Record<string, string>) => void;
  params: ParamSpec[];
  values: Record<string, unknown>;
}

/**
 * Sort params so required fields come first, preserving relative order within
 * each group.
 */
function sortedParams(params: ParamSpec[]): ParamSpec[] {
  const required: ParamSpec[] = [];
  const optional: ParamSpec[] = [];
  for (const p of params) {
    if (p.required) {
      required.push(p);
    } else {
      optional.push(p);
    }
  }
  return [...required, ...optional];
}

/** Whether a param type needs a full-width row (textarea-based fields). */
function isWideField(spec: ParamSpec): boolean {
  return spec.type === "array" || spec.type === "object";
}

function validateField(spec: ParamSpec, value: unknown): string | undefined {
  // Check required
  if (
    spec.required &&
    (value === undefined || value === null || value === "")
  ) {
    return "Required";
  }

  // Number validation
  if (
    (spec.type === "integer" || spec.type === "number") &&
    typeof value === "string" &&
    value !== ""
  ) {
    const n = Number(value);
    if (Number.isNaN(n)) {
      return "Must be a number";
    }
    if (spec.type === "integer" && !Number.isInteger(n)) {
      return "Must be a number";
    }
  }

  // JSON validation for array/object
  if (
    (spec.type === "array" || spec.type === "object") &&
    typeof value === "string" &&
    value !== ""
  ) {
    try {
      JSON.parse(value);
    } catch (e: unknown) {
      if (e instanceof SyntaxError) {
        return e.message;
      }
      return "Invalid JSON";
    }
  }

  return undefined;
}

function StringField({
  autoFocus,
  spec,
  value,
  error,
  onValueChange,
  onErrorChange,
  fieldId,
  errorId,
}: {
  autoFocus?: boolean;
  spec: ParamSpec;
  value: unknown;
  error: string | undefined;
  onValueChange: (name: string, value: unknown) => void;
  onErrorChange: (name: string, error: string | undefined) => void;
  fieldId: string;
  errorId: string;
}) {
  const inputRef = useRef<HTMLInputElement>(null);
  useEffect(() => {
    if (autoFocus) {
      inputRef.current?.focus();
    }
  }, [autoFocus]);

  const handleChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const v = e.target.value;
      onValueChange(spec.name, v);
      const err = validateField(spec, v);
      onErrorChange(spec.name, err);
    },
    [spec, onValueChange, onErrorChange]
  );

  return (
    <Input
      aria-describedby={error !== undefined ? errorId : undefined}
      aria-invalid={error !== undefined ? true : undefined}
      aria-required={spec.required ? true : undefined}
      id={fieldId}
      onChange={handleChange}
      placeholder={spec.description ?? ""}
      ref={inputRef}
      value={typeof value === "string" ? value : ""}
    />
  );
}

function NumberField({
  autoFocus,
  spec,
  value,
  error,
  onValueChange,
  onErrorChange,
  fieldId,
  errorId,
}: {
  autoFocus?: boolean;
  spec: ParamSpec;
  value: unknown;
  error: string | undefined;
  onValueChange: (name: string, value: unknown) => void;
  onErrorChange: (name: string, error: string | undefined) => void;
  fieldId: string;
  errorId: string;
}) {
  const inputRef = useRef<HTMLInputElement>(null);
  useEffect(() => {
    if (autoFocus) {
      inputRef.current?.focus();
    }
  }, [autoFocus]);

  const handleChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const v = e.target.value;
      onValueChange(spec.name, v);
      const err = validateField(spec, v);
      onErrorChange(spec.name, err);
    },
    [spec, onValueChange, onErrorChange]
  );

  return (
    <Input
      aria-describedby={error !== undefined ? errorId : undefined}
      aria-invalid={error !== undefined ? true : undefined}
      aria-required={spec.required ? true : undefined}
      id={fieldId}
      onChange={handleChange}
      placeholder={spec.description ?? ""}
      ref={inputRef}
      type="number"
      value={
        typeof value === "string" || typeof value === "number"
          ? String(value)
          : ""
      }
    />
  );
}

function BooleanField({
  spec,
  value,
  onValueChange,
  fieldId,
}: {
  spec: ParamSpec;
  value: unknown;
  onValueChange: (name: string, value: unknown) => void;
  fieldId: string;
}) {
  const handleChange = useCallback(
    (checked: boolean) => {
      onValueChange(spec.name, checked);
    },
    [spec.name, onValueChange]
  );

  return (
    <Switch
      checked={Boolean(value)}
      id={fieldId}
      onCheckedChange={handleChange}
    />
  );
}

function JsonField({
  spec,
  value,
  error,
  onValueChange,
  onErrorChange,
  fieldId,
  errorId,
}: {
  spec: ParamSpec;
  value: unknown;
  error: string | undefined;
  onValueChange: (name: string, value: unknown) => void;
  onErrorChange: (name: string, error: string | undefined) => void;
  fieldId: string;
  errorId: string;
}) {
  const handleChange = useCallback(
    (e: React.ChangeEvent<HTMLTextAreaElement>) => {
      const v = e.target.value;
      onValueChange(spec.name, v);
      const err = validateField(spec, v);
      onErrorChange(spec.name, err);
    },
    [spec, onValueChange, onErrorChange]
  );

  const handleFormat = useCallback(() => {
    if (typeof value !== "string" || value === "") {
      return;
    }
    try {
      const parsed: unknown = JSON.parse(value);
      const formatted = JSON.stringify(parsed, null, 2);
      onValueChange(spec.name, formatted);
      onErrorChange(spec.name, undefined);
    } catch {
      // If invalid JSON, validation error is already shown
    }
  }, [value, spec.name, onValueChange, onErrorChange]);

  return (
    <div className="space-y-1.5">
      <Textarea
        aria-describedby={error !== undefined ? errorId : undefined}
        aria-invalid={error !== undefined ? true : undefined}
        aria-required={spec.required ? true : undefined}
        className="min-h-[80px] font-mono text-xs"
        id={fieldId}
        onChange={handleChange}
        placeholder={spec.description ?? ""}
        value={typeof value === "string" ? value : ""}
      />
      <Button onClick={handleFormat} size="xs" type="button" variant="ghost">
        Format JSON
      </Button>
    </div>
  );
}

function ParamField({
  autoFocus,
  spec,
  value,
  error,
  onValueChange,
  onErrorChange,
}: {
  autoFocus?: boolean;
  spec: ParamSpec;
  value: unknown;
  error: string | undefined;
  onValueChange: (name: string, value: unknown) => void;
  onErrorChange: (name: string, error: string | undefined) => void;
}) {
  const reactId = useId();
  const fieldId = `param-${spec.name}-${reactId}`;
  const errorId = `${spec.name}-error`;

  const renderInput = () => {
    switch (spec.type) {
      case "string":
        return (
          <StringField
            autoFocus={autoFocus}
            error={error}
            errorId={errorId}
            fieldId={fieldId}
            onErrorChange={onErrorChange}
            onValueChange={onValueChange}
            spec={spec}
            value={value}
          />
        );
      case "integer":
      case "number":
        return (
          <NumberField
            autoFocus={autoFocus}
            error={error}
            errorId={errorId}
            fieldId={fieldId}
            onErrorChange={onErrorChange}
            onValueChange={onValueChange}
            spec={spec}
            value={value}
          />
        );
      case "boolean":
        return (
          <BooleanField
            fieldId={fieldId}
            onValueChange={onValueChange}
            spec={spec}
            value={value}
          />
        );
      case "array":
      case "object":
        return (
          <JsonField
            error={error}
            errorId={errorId}
            fieldId={fieldId}
            onErrorChange={onErrorChange}
            onValueChange={onValueChange}
            spec={spec}
            value={value}
          />
        );
      case "null":
        return (
          <Input
            disabled
            id={fieldId}
            placeholder={spec.description ?? ""}
            value="null"
          />
        );
      default:
        return null;
    }
  };

  return (
    <div className={cn("space-y-1", isWideField(spec) && "col-span-full")}>
      <Label className="text-[0.65rem] text-muted-foreground" htmlFor={fieldId}>
        {spec.name}
        {spec.required && (
          <span
            aria-hidden="true"
            className="ml-1 inline-block size-1.5 rounded-full bg-destructive align-middle"
          />
        )}
      </Label>
      {renderInput()}
      {error !== undefined && (
        <span className="text-destructive text-xs" id={errorId} role="alert">
          {error}
        </span>
      )}
    </div>
  );
}

export function ParamForm({
  autoFocus,
  params,
  values,
  onChange,
  errors,
  onErrorsChange,
}: ParamFormProps) {
  const sorted = sortedParams(params);

  const handleValueChange = useCallback(
    (name: string, value: unknown) => {
      onChange({ ...values, [name]: value });
    },
    [values, onChange]
  );

  const handleErrorChange = useCallback(
    (name: string, error: string | undefined) => {
      if (error === undefined) {
        const next = { ...errors };
        // eslint-disable-next-line @typescript-eslint/no-dynamic-delete
        delete next[name];
        onErrorsChange(next);
      } else {
        onErrorsChange({ ...errors, [name]: error });
      }
    },
    [errors, onErrorsChange]
  );

  if (sorted.length === 0) {
    return (
      <p className="text-muted-foreground text-xs">
        This command has no parameters.
      </p>
    );
  }

  const autoFocusIndex = autoFocus
    ? sorted.findIndex(
        (spec) =>
          spec.required &&
          (values[spec.name] === undefined ||
            values[spec.name] === null ||
            values[spec.name] === "")
      )
    : -1;

  return (
    <div className="grid grid-cols-[repeat(auto-fill,minmax(140px,1fr))] gap-x-3 gap-y-2">
      {sorted.map((spec, i) => (
        <ParamField
          autoFocus={i === autoFocusIndex}
          error={errors[spec.name]}
          key={spec.name}
          onErrorChange={handleErrorChange}
          onValueChange={handleValueChange}
          spec={spec}
          value={values[spec.name]}
        />
      ))}
    </div>
  );
}
