import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Button, Input, Label, TextField } from "react-aria-components";
import { useForm } from "react-hook-form";
import { useIntl } from "react-intl";
import {
  commands,
  type ActionReceipt,
  type Area,
} from "@lifeos/contracts/bindings";

type FormValues = { title: string };
type CommandResult<T> =
  { status: "ok"; data: T } | { status: "error"; error: unknown };

async function unwrapCommand<T>(
  command: Promise<CommandResult<T>>,
): Promise<T> {
  const result = await command;
  if (result.status === "error") throw result.error;
  return result.data;
}

export function AreaScreen() {
  const intl = useIntl();
  const queryClient = useQueryClient();
  const [undo, setUndo] = useState<string | null>(null);
  const areas = useQuery({
    queryKey: ["areas"],
    queryFn: () => unwrapCommand(commands.listAreas()),
  });
  const form = useForm<FormValues>({ defaultValues: { title: "" } });
  const create = useMutation({
    mutationFn: (title: string) =>
      unwrapCommand<ActionReceipt<Area>>(
        commands.createArea({ title, operationId: crypto.randomUUID() }),
      ),
    onSuccess: (receipt) => {
      form.reset();
      setUndo(receipt.undoBatchId);
      void queryClient.invalidateQueries({ queryKey: ["areas"] });
    },
  });
  const undoMutation = useMutation({
    mutationFn: (undoBatchId: string) =>
      unwrapCommand(
        commands.undoAction({ undoBatchId, operationId: crypto.randomUUID() }),
      ),
    onSuccess: () => {
      setUndo(null);
      void queryClient.invalidateQueries({ queryKey: ["areas"] });
    },
  });

  return (
    <section className="canvas" aria-labelledby="areas-heading">
      <div className="eyebrow">
        {intl.formatMessage({ id: "area.eyebrow" })}
      </div>
      <div className="page-heading">
        <div>
          <h1 id="areas-heading">{intl.formatMessage({ id: "area.title" })}</h1>
          <p>{intl.formatMessage({ id: "area.description" })}</p>
        </div>
        <span
          className="metric"
          aria-label={intl.formatMessage(
            { id: "area.count" },
            { count: areas.data?.length ?? 0 },
          )}
        >
          {areas.data?.length ?? 0}
        </span>
      </div>
      <form
        className="create-card"
        onSubmit={form.handleSubmit(({ title }) => create.mutate(title))}
      >
        <TextField>
          <Label>{intl.formatMessage({ id: "area.name" })}</Label>
          <Input
            {...form.register("title", { required: true, maxLength: 200 })}
          />
        </TextField>
        <Button type="submit" isDisabled={create.isPending}>
          {intl.formatMessage({ id: "area.create" })}
        </Button>
      </form>
      {create.isError ? (
        <p className="form-error" role="alert">
          {intl.formatMessage({ id: "error.generic" })}
        </p>
      ) : null}
      {areas.isLoading ? (
        <p>{intl.formatMessage({ id: "common.loading" })}</p>
      ) : areas.isError ? (
        <p role="alert">{intl.formatMessage({ id: "error.generic" })}</p>
      ) : areas.data?.length === 0 ? (
        <div className="empty">
          <h2>{intl.formatMessage({ id: "empty.areas" })}</h2>
          <p>{intl.formatMessage({ id: "empty.areas.detail" })}</p>
        </div>
      ) : (
        <div className="area-grid">
          {areas.data?.map((area) => (
            <article className="area-card" key={area.id}>
              <span>{intl.formatMessage({ id: "entity.area" })}</span>
              <h2 dir="auto">{area.title}</h2>
              <small>
                {intl.formatMessage(
                  { id: "entity.revision" },
                  { revision: area.revision },
                )}
              </small>
            </article>
          ))}
        </div>
      )}
      {undo ? (
        <aside className="undo-toast" role="status">
          {intl.formatMessage({ id: "receipt.areaCreated" })}
          <Button onPress={() => undoMutation.mutate(undo)}>
            {intl.formatMessage({ id: "common.undo" })}
          </Button>
        </aside>
      ) : null}
    </section>
  );
}
