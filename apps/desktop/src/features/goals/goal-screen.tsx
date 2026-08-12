import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Button, Input, Label, TextField } from "react-aria-components";
import { useForm } from "react-hook-form";
import { useIntl } from "react-intl";
import {
  commands,
  type ActionReceipt,
  type Goal,
  type GoalHorizon,
} from "@lifeos/contracts/bindings";

type GoalFormValues = {
  title: string;
  horizon: GoalHorizon;
  startDate: string;
  targetDate: string;
};
type CommandResult<T> =
  { status: "ok"; data: T } | { status: "error"; error: unknown };

async function unwrapCommand<T>(
  command: Promise<CommandResult<T>>,
): Promise<T> {
  const result = await command;
  if (result.status === "error") throw result.error;
  return result.data;
}

export function GoalScreen() {
  const intl = useIntl();
  const queryClient = useQueryClient();
  const [undo, setUndo] = useState<string | null>(null);
  const [editing, setEditing] = useState<Goal | null>(null);
  const goals = useQuery({
    queryKey: ["goals"],
    queryFn: () => unwrapCommand(commands.listGoals()),
  });
  const archivedGoals = useQuery({
    queryKey: ["archived-goals"],
    queryFn: () => unwrapCommand(commands.listArchivedGoals()),
  });
  const trashedGoals = useQuery({
    queryKey: ["trashed-goals"],
    queryFn: () => unwrapCommand(commands.listTrashedGoals()),
  });
  const form = useForm<GoalFormValues>({
    defaultValues: {
      title: "",
      horizon: "medium",
      startDate: "",
      targetDate: "",
    },
  });
  const create = useMutation({
    mutationFn: (values: GoalFormValues) =>
      unwrapCommand<ActionReceipt<Goal>>(
        commands.createGoal({
          title: values.title,
          horizon: values.horizon,
          startDate: values.startDate || null,
          targetDate: values.targetDate || null,
          operationId: crypto.randomUUID(),
        }),
      ),
    onSuccess: (receipt) => {
      form.reset();
      setUndo(receipt.undoBatchId);
      void queryClient.invalidateQueries({ queryKey: ["goals"] });
    },
  });
  const undoMutation = useMutation({
    mutationFn: (undoBatchId: string) =>
      unwrapCommand(
        commands.undoAction({ undoBatchId, operationId: crypto.randomUUID() }),
      ),
    onSuccess: () => {
      setUndo(null);
      void queryClient.invalidateQueries({ queryKey: ["goals"] });
    },
  });
  const update = useMutation({
    mutationFn: ({
      id,
      revision,
      values,
    }: {
      id: string;
      revision: number;
      values: GoalFormValues;
    }) =>
      unwrapCommand<ActionReceipt<Goal>>(
        commands.updateGoal({
          id,
          title: values.title,
          horizon: values.horizon,
          startDate: values.startDate || null,
          targetDate: values.targetDate || null,
          expectedRevision: revision,
          operationId: crypto.randomUUID(),
        }),
      ),
    onSuccess: (receipt) => {
      setEditing(null);
      setUndo(receipt.undoBatchId);
      queryClient.setQueryData<Goal[]>(["goals"], (current) =>
        current?.map((goal) =>
          goal.id === receipt.data.id ? receipt.data : goal,
        ),
      );
      void queryClient.invalidateQueries({ queryKey: ["goals"] });
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
      action: "archive" | "trash" | "restore";
    }) =>
      unwrapCommand(
        (action === "archive"
          ? commands.archiveGoal
          : action === "trash"
            ? commands.trashGoal
            : commands.restoreGoal)({
          id,
          expectedRevision: revision,
          operationId: crypto.randomUUID(),
        }),
      ),
    onSuccess: (receipt) => {
      setUndo(receipt.undoBatchId);
      void Promise.all([
        queryClient.invalidateQueries({ queryKey: ["goals"] }),
        queryClient.invalidateQueries({ queryKey: ["archived-goals"] }),
        queryClient.invalidateQueries({ queryKey: ["trashed-goals"] }),
      ]);
    },
  });

  return (
    <section className="canvas" aria-labelledby="goals-heading">
      <div className="eyebrow">
        {intl.formatMessage({ id: "goal.eyebrow" })}
      </div>
      <div className="page-heading">
        <div>
          <h1 id="goals-heading">{intl.formatMessage({ id: "goal.title" })}</h1>
          <p>{intl.formatMessage({ id: "goal.description" })}</p>
        </div>
        <span
          className="metric"
          aria-label={intl.formatMessage(
            { id: "goal.count" },
            { count: goals.data?.length ?? 0 },
          )}
        >
          {goals.data?.length ?? 0}
        </span>
      </div>
      <form
        className="create-card goal-create-card"
        onSubmit={form.handleSubmit((values) => create.mutate(values))}
      >
        <TextField>
          <Label>{intl.formatMessage({ id: "goal.name" })}</Label>
          <Input
            {...form.register("title", { required: true, maxLength: 200 })}
          />
        </TextField>
        <label className="select-field">
          {intl.formatMessage({ id: "goal.horizon" })}
          <select {...form.register("horizon")}>
            <option value="short">
              {intl.formatMessage({ id: "goal.horizon.short" })}
            </option>
            <option value="medium">
              {intl.formatMessage({ id: "goal.horizon.medium" })}
            </option>
            <option value="long">
              {intl.formatMessage({ id: "goal.horizon.long" })}
            </option>
            <option value="lifetime">
              {intl.formatMessage({ id: "goal.horizon.lifetime" })}
            </option>
          </select>
        </label>
        <TextField>
          <Label>{intl.formatMessage({ id: "goal.startDate" })}</Label>
          <Input type="date" {...form.register("startDate")} />
        </TextField>
        <TextField>
          <Label>{intl.formatMessage({ id: "goal.targetDate" })}</Label>
          <Input type="date" {...form.register("targetDate")} />
        </TextField>
        <Button type="submit" isDisabled={create.isPending}>
          {intl.formatMessage({ id: "goal.create" })}
        </Button>
      </form>
      {create.isError ? (
        <p className="form-error" role="alert">
          {intl.formatMessage({ id: "error.generic" })}
        </p>
      ) : null}
      {goals.isLoading ? (
        <p>{intl.formatMessage({ id: "common.loading" })}</p>
      ) : goals.isError ? (
        <p role="alert">{intl.formatMessage({ id: "error.generic" })}</p>
      ) : goals.data?.length === 0 ? (
        <div className="empty">
          <h2>{intl.formatMessage({ id: "empty.goals" })}</h2>
          <p>{intl.formatMessage({ id: "empty.goals.detail" })}</p>
        </div>
      ) : (
        <div className="area-grid">
          {goals.data?.map((goal) => (
            <article className="area-card" key={goal.id}>
              <span>{intl.formatMessage({ id: "entity.goal" })}</span>
              <h2 dir="auto">{goal.title}</h2>
              <small>
                {intl.formatMessage({ id: `goal.horizon.${goal.horizon}` })}
              </small>
              {goal.targetDate ? (
                <small>
                  {intl.formatMessage({ id: "goal.targetDate" })}:{" "}
                  <time dateTime={goal.targetDate}>{goal.targetDate}</time>
                </small>
              ) : null}
              <div className="area-actions">
                <Button onPress={() => setEditing(goal)}>
                  {intl.formatMessage({ id: "goal.edit" })}
                </Button>
                <Button
                  onPress={() =>
                    lifecycle.mutate({
                      id: goal.id,
                      revision: goal.revision,
                      action: "archive",
                    })
                  }
                >
                  {intl.formatMessage({ id: "goal.archive" })}
                </Button>
                <Button
                  onPress={() =>
                    lifecycle.mutate({
                      id: goal.id,
                      revision: goal.revision,
                      action: "trash",
                    })
                  }
                >
                  {intl.formatMessage({ id: "goal.trash" })}
                </Button>
              </div>
            </article>
          ))}
        </div>
      )}
      <GoalCollection
        title={intl.formatMessage({ id: "goal.archivedTitle" })}
        empty={intl.formatMessage({ id: "goal.archivedEmpty" })}
        goals={archivedGoals.data}
        loading={archivedGoals.isLoading}
        failed={archivedGoals.isError}
        onRestore={(goal) =>
          lifecycle.mutate({
            id: goal.id,
            revision: goal.revision,
            action: "restore",
          })
        }
      />
      <GoalCollection
        title={intl.formatMessage({ id: "goal.trashedTitle" })}
        empty={intl.formatMessage({ id: "goal.trashedEmpty" })}
        goals={trashedGoals.data}
        loading={trashedGoals.isLoading}
        failed={trashedGoals.isError}
        onRestore={(goal) =>
          lifecycle.mutate({
            id: goal.id,
            revision: goal.revision,
            action: "restore",
          })
        }
      />
      {undo ? (
        <aside className="undo-toast" role="status">
          {intl.formatMessage({ id: "receipt.goalCreated" })}
          <Button onPress={() => undoMutation.mutate(undo)}>
            {intl.formatMessage({ id: "common.undo" })}
          </Button>
        </aside>
      ) : null}
      {editing ? (
        <EditGoalForm
          goal={editing}
          busy={update.isPending}
          failed={update.isError}
          onCancel={() => setEditing(null)}
          onSave={(values) =>
            update.mutate({
              id: editing.id,
              revision: editing.revision,
              values,
            })
          }
        />
      ) : null}
    </section>
  );
}

function GoalCollection({
  title,
  empty,
  goals,
  loading,
  failed,
  onRestore,
}: {
  title: string;
  empty: string;
  goals: Goal[] | undefined;
  loading: boolean;
  failed: boolean;
  onRestore: (goal: Goal) => void;
}) {
  const intl = useIntl();
  return (
    <section className="goal-collection" aria-label={title}>
      <h2>{title}</h2>
      {loading ? <p>{intl.formatMessage({ id: "common.loading" })}</p> : null}
      {failed ? (
        <p role="alert">{intl.formatMessage({ id: "error.generic" })}</p>
      ) : null}
      {goals?.length === 0 ? <p>{empty}</p> : null}
      <div className="area-grid">
        {goals?.map((goal) => (
          <article className="area-card" key={goal.id}>
            <h3 dir="auto">{goal.title}</h3>
            <Button onPress={() => onRestore(goal)}>
              {intl.formatMessage({ id: "goal.restore" })}
            </Button>
          </article>
        ))}
      </div>
    </section>
  );
}

function EditGoalForm({
  goal,
  busy,
  failed,
  onCancel,
  onSave,
}: {
  goal: Goal;
  busy: boolean;
  failed: boolean;
  onCancel: () => void;
  onSave: (values: GoalFormValues) => void;
}) {
  const intl = useIntl();
  const form = useForm<GoalFormValues>({
    defaultValues: {
      title: goal.title,
      horizon: goal.horizon,
      startDate: goal.startDate ?? "",
      targetDate: goal.targetDate ?? "",
    },
  });
  return (
    <form
      className="edit-card"
      aria-labelledby="edit-goal-heading"
      onSubmit={form.handleSubmit(onSave)}
    >
      <h2 id="edit-goal-heading">
        {intl.formatMessage({ id: "goal.editTitle" })}
      </h2>
      <TextField>
        <Label>{intl.formatMessage({ id: "goal.name" })}</Label>
        <Input
          {...form.register("title", { required: true, maxLength: 200 })}
        />
      </TextField>
      <label className="select-field">
        {intl.formatMessage({ id: "goal.horizon" })}
        <select {...form.register("horizon")}>
          <option value="short">
            {intl.formatMessage({ id: "goal.horizon.short" })}
          </option>
          <option value="medium">
            {intl.formatMessage({ id: "goal.horizon.medium" })}
          </option>
          <option value="long">
            {intl.formatMessage({ id: "goal.horizon.long" })}
          </option>
          <option value="lifetime">
            {intl.formatMessage({ id: "goal.horizon.lifetime" })}
          </option>
        </select>
      </label>
      <TextField>
        <Label>{intl.formatMessage({ id: "goal.startDate" })}</Label>
        <Input type="date" {...form.register("startDate")} />
      </TextField>
      <TextField>
        <Label>{intl.formatMessage({ id: "goal.targetDate" })}</Label>
        <Input type="date" {...form.register("targetDate")} />
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
