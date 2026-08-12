import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
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
  const [editing, setEditing] = useState<Area | null>(null);
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
  const lifecycle = useMutation({
    mutationFn: ({
      id,
      revision,
      action,
    }: {
      id: string;
      revision: number;
      action: "archive" | "trash";
    }) =>
      unwrapCommand(
        action === "archive"
          ? commands.archiveArea({
              id,
              expectedRevision: revision,
              operationId: crypto.randomUUID(),
            })
          : commands.trashArea({
              id,
              expectedRevision: revision,
              operationId: crypto.randomUUID(),
            }),
      ),
    onSuccess: (receipt) => {
      setUndo(receipt.undoBatchId);
      void queryClient.invalidateQueries({ queryKey: ["areas"] });
    },
  });
  const update = useMutation({
    mutationFn: ({
      id,
      title,
      revision,
    }: {
      id: string;
      title: string;
      revision: number;
    }) =>
      unwrapCommand<ActionReceipt<Area>>(
        commands.updateArea({
          id,
          title,
          expectedRevision: revision,
          operationId: crypto.randomUUID(),
        }),
      ),
    onSuccess: (receipt) => {
      setEditing(null);
      setUndo(receipt.undoBatchId);
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
              <div className="area-actions">
                <Link to="/versions/$entityId" params={{ entityId: area.id }}>
                  {intl.formatMessage({ id: "area.history" })}
                </Link>
                <Button onPress={() => setEditing(area)}>
                  {intl.formatMessage({ id: "area.edit" })}
                </Button>
                <Button
                  onPress={() =>
                    lifecycle.mutate({
                      id: area.id,
                      revision: area.revision,
                      action: "archive",
                    })
                  }
                >
                  {intl.formatMessage({ id: "area.archive" })}
                </Button>
                <Button
                  onPress={() =>
                    lifecycle.mutate({
                      id: area.id,
                      revision: area.revision,
                      action: "trash",
                    })
                  }
                >
                  {intl.formatMessage({ id: "area.trash" })}
                </Button>
              </div>
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
      {editing ? (
        <EditAreaForm
          area={editing}
          busy={update.isPending}
          failed={update.isError}
          onCancel={() => setEditing(null)}
          onSave={(title) =>
            update.mutate({
              id: editing.id,
              title,
              revision: editing.revision,
            })
          }
        />
      ) : null}
    </section>
  );
}

function EditAreaForm({
  area,
  busy,
  failed,
  onCancel,
  onSave,
}: {
  area: Area;
  busy: boolean;
  failed: boolean;
  onCancel: () => void;
  onSave: (title: string) => void;
}) {
  const intl = useIntl();
  const form = useForm<FormValues>({ defaultValues: { title: area.title } });
  return (
    <form
      className="edit-card"
      aria-labelledby="edit-area-heading"
      onSubmit={form.handleSubmit(({ title }) => onSave(title))}
    >
      <h2 id="edit-area-heading">
        {intl.formatMessage({ id: "area.editTitle" })}
      </h2>
      <TextField>
        <Label>{intl.formatMessage({ id: "area.name" })}</Label>
        <Input
          {...form.register("title", { required: true, maxLength: 200 })}
        />
      </TextField>
      <div className="area-actions">
        <Button type="submit" isDisabled={busy}>
          {intl.formatMessage({ id: "common.save" })}
        </Button>
        <Button type="button" onPress={onCancel}>
          {intl.formatMessage({ id: "common.cancel" })}
        </Button>
      </div>
      {failed ? (
        <p className="form-error" role="alert">
          {intl.formatMessage({ id: "error.generic" })}
        </p>
      ) : null}
    </form>
  );
}
