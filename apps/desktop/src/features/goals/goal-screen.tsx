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
  const goals = useQuery({
    queryKey: ["goals"],
    queryFn: () => unwrapCommand(commands.listGoals()),
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
            </article>
          ))}
        </div>
      )}
      {undo ? (
        <aside className="undo-toast" role="status">
          {intl.formatMessage({ id: "receipt.goalCreated" })}
          <Button onPress={() => undoMutation.mutate(undo)}>
            {intl.formatMessage({ id: "common.undo" })}
          </Button>
        </aside>
      ) : null}
    </section>
  );
}
